use std::{collections::HashSet, sync::Arc, time::Duration};

use agnostic_orderbook::state::{
    critbit::{LeafNode, Slab},
    market_state::MarketState as OrderBookMarketState,
    orderbook::OrderBookState,
    AccountTag, OrderSummary,
};
use anchor_lang::AccountDeserialize;
use anchor_spl::token::TokenAccount;
use anyhow::{bail, Result};
use async_trait::async_trait;

use jet_fixed_term::{
    control::state::Market,
    margin::state::{AutoRollConfig, MarginUser, TermLoan},
    orderbook::state::{
        event_queue_len, orderbook_slab_len, CallbackInfo, MarketSide, OrderParams,
    },
    tickets::state::TermDeposit,
};
use jet_instructions::{
    fixed_term::{derive, InitializeMarketParams},
    margin::{derive_adapter_config, MarginConfigIxBuilder},
};
use jet_margin::{TokenAdmin, TokenConfigUpdate, TokenKind};
use jet_margin_sdk::{
    fixed_term::{
        event_consumer::{download_markets, EventConsumer},
        settler::{settle_margin_users_loop, SettleMarginUsersConfig},
        FixedTermIxBuilder, OrderbookAddresses, OwnedEventQueue,
    },
    ix_builder::{get_control_authority_address, MarginIxBuilder},
    margin_integrator::{NoProxy, Proxy, RefreshingProxy},
    solana::{
        keypair::clone,
        keypair::KeypairExt,
        transaction::{
            InverseSendTransactionBuilder, SendTransactionBuilder, TransactionBuilder,
            TransactionBuilderExt, WithSigner,
        },
    },
    tx_builder::global_initialize_instructions,
    util::no_dupe_queue::AsyncNoDupeQueue,
};
use jet_program_common::Fp32;
use jet_simulation::{send_and_confirm, solana_rpc_api::SolanaRpcClient};
use solana_sdk::{
    hash::Hash,
    instruction::Instruction,
    message::Message,
    native_token::LAMPORTS_PER_SOL,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::{instruction::initialize_mint, state::Mint};

use crate::{
    context::MarginTestContext,
    runtime::{Keygen, SolanaTestContext},
    setup_helper::{register_deposit, setup_user},
    tokens::TokenManager,
};

pub const LOCALNET_URL: &str = "http://127.0.0.1:8899";
pub const DEVNET_URL: &str = "https://api.devnet.solana.com/";

pub const STARTING_TOKENS: u64 = 1_000_000_000;
pub const MARKET_SEED: [u8; 32] = *b"verygoodlongseedfrombytewemakeit";
pub const MARKET_TAG: u64 = u64::from_le_bytes(*b"zachzach");
pub const FEEDER_FUND_SEED: u64 = u64::from_le_bytes(*b"feedingf");
pub const ORDERBOOK_CAPACITY: usize = 1_000;
pub const EVENT_QUEUE_CAPACITY: usize = 1_000;
pub const BORROW_TENOR: u64 = 3;
pub const LEND_TENOR: u64 = 5; // in seconds
pub const ORIGINATION_FEE: u64 = 10;
pub const MIN_ORDER_SIZE: u64 = 10;
pub const ORDERBOOK_PARAMS: InitializeMarketParams = InitializeMarketParams {
    version_tag: MARKET_TAG,
    seed: MARKET_SEED,
    borrow_tenor: BORROW_TENOR,
    lend_tenor: LEND_TENOR,
    origination_fee: ORIGINATION_FEE,
};

pub struct TestManager {
    pub client: Arc<dyn SolanaRpcClient>,
    pub keygen: Arc<dyn Keygen>,
    pub ix_builder: FixedTermIxBuilder,
    pub event_consumer: Arc<EventConsumer>,
    pub margin_accounts_to_settle: AsyncNoDupeQueue<Pubkey>,
    pub orderbook: OrderbookKeypairs,
    pub mint_authority: Keypair,
    airspace: Pubkey,
}

impl Clone for TestManager {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            ix_builder: self.ix_builder.clone(),
            event_consumer: self.event_consumer.clone(),
            keygen: self.keygen.clone(),
            orderbook: self.orderbook.clone(),
            mint_authority: clone(&self.mint_authority),
            margin_accounts_to_settle: AsyncNoDupeQueue::new(),
            airspace: self.airspace,
        }
    }
}

impl TestManager {
    pub async fn full(client: &MarginTestContext) -> Result<Self> {
        let (mint, mint_authority) = generate_test_mint(&client.solana).await?;
        let oracle = TokenManager::new(client.solana.clone())
            .create_oracle(&mint)
            .await?;
        let ticket_mint = derive::fixed_term_address(&[
            jet_fixed_term::seeds::TICKET_MINT,
            derive::market(&client.airspace, &mint, MARKET_SEED).as_ref(),
        ]);
        let ticket_oracle = TokenManager::new(client.solana.clone())
            .create_oracle(&ticket_mint)
            .await?;

        Self::new(
            client.solana.clone(),
            client.airspace,
            &mint,
            mint_authority,
            OrderbookKeypairs::generate(&client.solana.keygen),
            oracle.price,
            ticket_oracle.price,
        )
        .with_market()
        .await?
        .with_crank()
        .await?
        .with_margin(&client.airspace_authority)
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        client: SolanaTestContext,
        airspace: Pubkey,
        mint: &Pubkey,
        mint_authority: Keypair,
        orderbook: OrderbookKeypairs,
        underlying_oracle: Pubkey,
        ticket_oracle: Pubkey,
    ) -> Self {
        let SolanaTestContext {
            rpc: client,
            keygen,
        } = client;
        let payer = client.payer();

        let ix_builder = FixedTermIxBuilder::new_from_seed(
            payer.pubkey(),
            &airspace,
            mint,
            MARKET_SEED,
            payer.pubkey(),
            underlying_oracle,
            ticket_oracle,
            None,
            (&orderbook).into(),
        );
        Self {
            client: client.clone(),
            event_consumer: Arc::new(EventConsumer::new(client.clone())),
            keygen,
            ix_builder,
            orderbook,
            mint_authority,
            margin_accounts_to_settle: Default::default(),
            airspace,
        }
    }

    pub async fn with_market(self) -> Result<Self> {
        self.init_market().await?;
        Ok(self)
    }

    pub async fn init_market(&self) -> Result<()> {
        init_market(
            &self.client,
            &self.ix_builder,
            self.orderbook.clone(),
            ORDERBOOK_PARAMS,
        )
        .await
    }

    pub async fn with_crank(self) -> Result<Self> {
        let auth_crank = self
            .ix_builder
            .authorize_crank(self.client.payer().pubkey());

        self.sign_send_transaction(&[auth_crank], &[]).await?;
        Ok(self)
    }

    /// set up metadata authorization for margin to invoke fixed term and
    /// register relevant positions.
    pub async fn with_margin(self, airspace_authority: &Keypair) -> Result<Self> {
        self.create_authority_if_missing().await?;
        self.register_adapter_if_unregistered(&jet_fixed_term::ID, airspace_authority)
            .await?;
        self.register_adapter_if_unregistered(&jet_margin_pool::ID, airspace_authority)
            .await?;
        self.register_adapter_if_unregistered(&jet_margin_swap::ID, airspace_authority)
            .await?;
        self.register_tickets_position_metadatata(airspace_authority)
            .await?;
        register_deposit(
            &self.client,
            self.airspace,
            airspace_authority,
            self.ix_builder.token_mint(),
            None,
        )
        .await?;
        register_deposit(
            &self.client,
            self.airspace,
            airspace_authority,
            self.ix_builder.ticket_mint(),
            None,
        )
        .await?;

        Ok(self)
    }

    pub async fn sign_send_transaction(
        &self,
        instructions: &[Instruction],
        extra_signers: &[&Keypair],
    ) -> Result<Signature> {
        let mut signers = Vec::<&Keypair>::new();
        let mut keypairs = vec![];
        keypairs.extend_from_slice(extra_signers);
        keypairs.push(self.client.payer());
        keypairs.push(&self.mint_authority); // needed to fund users

        let msg = Message::new(instructions, Some(&self.client.payer().pubkey()));
        for signer in msg.signer_keys() {
            for kp in keypairs.clone() {
                if &kp.pubkey() == signer {
                    signers.push(kp);
                }
            }
        }
        let mut tx = Transaction::new_unsigned(msg);
        tx.sign(&signers, self.client.get_latest_blockhash().await?);

        self.client.send_and_confirm_transaction(&tx).await
    }
}

impl TestManager {
    pub async fn consume_events(&self) -> Result<()> {
        let market = self.ix_builder.market();

        loop {
            let market_struct = download_markets(self.client.as_ref(), &[market]).await?[0];
            self.event_consumer
                .insert_market(market_struct, Some(self.margin_accounts_to_settle.clone()));
            self.event_consumer.sync_queues().await?;
            self.event_consumer.sync_users().await?;

            let pending = self.event_consumer.pending_events(&market)?;
            if pending == 0 {
                break;
            }
            self.event_consumer.consume().await?;
            self.event_consumer.sync_queues().await?;
        }
        Ok(())
    }

    /// Two jobs:
    /// - Verifies that the event consumer has notified us that the expected
    ///   account needs to be settled. panic on failure.
    /// - settles those accounts. return error on failure.
    pub async fn expect_and_execute_settlement<P: Proxy>(
        &self,
        expected: &[&FixedTermUser<P>],
    ) -> Result<()> {
        self.expect_settlement(expected).await;
        self.settle(expected).await?;

        Ok(())
    }

    pub async fn expect_settlement<P: Proxy>(&self, expected: &[&FixedTermUser<P>]) {
        let to_settle = self.margin_accounts_to_settle.pop_many(usize::MAX).await;
        let expected_number_to_settle = expected.len();
        assert_eq!(expected_number_to_settle, to_settle.len());
        assert_eq!(
            expected
                .iter()
                .map(|u| u.proxy.pubkey())
                .collect::<HashSet<Pubkey>>(),
            to_settle.clone().into_iter().collect()
        );
        self.margin_accounts_to_settle.push_many(to_settle).await;
    }

    pub async fn settle<P: Proxy>(&self, users: &[&FixedTermUser<P>]) -> Result<()> {
        settle_margin_users_loop(
            self.client.clone(),
            self.ix_builder.clone(),
            self.margin_accounts_to_settle.clone(),
            SettleMarginUsersConfig {
                batch_size: std::cmp::max(1, users.len()),
                batch_delay: Duration::from_secs(0),
                wait_for_more_delay: Duration::from_secs(0),
                exit_when_done: true,
            },
        )
        .await;
        if self.margin_accounts_to_settle.is_empty().await {
            Ok(())
        } else {
            bail!("some settle transactions must have failed")
        }
    }

    pub async fn auto_roll_term_deposits(&self, margin_account: &Pubkey) -> Result<()> {
        let mut mature_deposits = self.load_mature_deposits(margin_account).await?;
        mature_deposits.sort_by(|a, b| a.1.sequence_number.cmp(&b.1.sequence_number));

        let mut seq_no = self
            .load_margin_user(margin_account)
            .await?
            .assets
            .next_new_deposit_seqno();
        let mut builder = Vec::<TransactionBuilder>::new();
        for (key, deposit) in mature_deposits {
            let ix =
                self.ix_builder
                    .auto_roll_lend_order(*margin_account, key, deposit.payer, seq_no);
            seq_no += 1;
            builder.push(ix.into())
        }
        builder.send_and_confirm_condensed(&self.client).await?;
        Ok(())
    }
    pub async fn pause_ticket_redemption(&self) -> Result<Signature> {
        let pause = self.ix_builder.pause_ticket_redemption();

        self.sign_send_transaction(&[pause], &[]).await
    }

    pub async fn resume_ticket_redemption(&self) -> Result<Signature> {
        let resume = self.ix_builder.resume_ticket_redemption();

        self.sign_send_transaction(&[resume], &[]).await
    }

    pub async fn pause_orders(&self) -> Result<Signature> {
        let pause = self.ix_builder.pause_order_matching();

        self.sign_send_transaction(&[pause], &[]).await
    }

    pub async fn resume_orders(&self) -> Result<()> {
        loop {
            if self.load_orderbook_market_state().await?.pause_matching == (false as u8) {
                break;
            }

            let resume = self.ix_builder.resume_order_matching();
            self.sign_send_transaction(&[resume], &[]).await?;
        }

        Ok(())
    }
}

pub struct OrderbookKeypairs {
    pub bids: Keypair,
    pub asks: Keypair,
    pub event_queue: Keypair,
}

impl OrderbookKeypairs {
    fn generate<K: Keygen>(keygen: &K) -> OrderbookKeypairs {
        OrderbookKeypairs {
            event_queue: keygen.generate_key(),
            bids: keygen.generate_key(),
            asks: keygen.generate_key(),
        }
    }
}

impl Clone for OrderbookKeypairs {
    fn clone(&self) -> Self {
        Self {
            bids: self.bids.clone(),
            asks: self.asks.clone(),
            event_queue: self.event_queue.clone(),
        }
    }
}

impl From<&OrderbookKeypairs> for OrderbookAddresses {
    fn from(val: &OrderbookKeypairs) -> Self {
        OrderbookAddresses {
            bids: val.bids.pubkey(),
            asks: val.asks.pubkey(),
            event_queue: val.event_queue.pubkey(),
        }
    }
}

pub async fn init_market(
    rpc: &Arc<dyn SolanaRpcClient>,
    ix_builder: &FixedTermIxBuilder,
    ob: OrderbookKeypairs,
    params: InitializeMarketParams,
) -> Result<()> {
    let init_eq = {
        let rent = rpc
            .get_minimum_balance_for_rent_exemption(event_queue_len(EVENT_QUEUE_CAPACITY))
            .await?;
        ix_builder.initialize_event_queue(&ob.event_queue.pubkey(), EVENT_QUEUE_CAPACITY, rent)
    };
    let orderbook_rent = rpc
        .get_minimum_balance_for_rent_exemption(orderbook_slab_len(ORDERBOOK_CAPACITY))
        .await?;
    let init_bids =
        ix_builder.initialize_orderbook_slab(&ob.bids.pubkey(), ORDERBOOK_CAPACITY, orderbook_rent);
    let init_asks =
        ix_builder.initialize_orderbook_slab(&ob.asks.pubkey(), ORDERBOOK_CAPACITY, orderbook_rent);

    let payer = rpc.payer().pubkey();
    let init_fee_destination = ix_builder.init_default_fee_destination(&payer).unwrap();
    let init_manager = ix_builder.initialize_market(payer, params);
    let init_orderbook = ix_builder.initialize_orderbook(payer, MIN_ORDER_SIZE);

    vec![init_eq, init_bids, init_asks, init_fee_destination]
        .with_signers(&[ob.event_queue, ob.bids, ob.asks])
        .send_and_confirm(rpc)
        .await?;
    vec![init_manager, init_orderbook]
        .with_signers(&[])
        .send_and_confirm(rpc)
        .await?;

    Ok(())
}

/// copy paste from jet_v2::hosted::margin
impl TestManager {
    pub async fn create_authority_if_missing(&self) -> Result<()> {
        if self
            .client
            .get_account(&get_control_authority_address())
            .await?
            .is_none()
        {
            self.init_globals().await?;
        }

        Ok(())
    }

    pub async fn init_globals(&self) -> Result<()> {
        let payer = self.client.payer().pubkey();

        self.client
            .send_and_confirm_condensed(global_initialize_instructions(payer))
            .await?;
        Ok(())
    }

    pub async fn register_adapter_if_unregistered(
        &self,
        adapter: &Pubkey,
        authority: &Keypair,
    ) -> Result<()> {
        if self
            .client
            .get_account(&derive_adapter_config(&self.airspace, adapter))
            .await?
            .is_none()
        {
            self.register_adapter(adapter, authority).await?;
        }

        Ok(())
    }

    pub async fn register_tickets_position_metadatata(
        &self,
        airspace_authority: &Keypair,
    ) -> Result<()> {
        let market = self.load_market().await?;
        self.register_tickets_position_metadatata_impl(
            market.claims_mint,
            market.underlying_token_mint,
            TokenKind::Claim,
            10_00,
            airspace_authority,
        )
        .await?;
        self.register_tickets_position_metadatata_impl(
            market.ticket_collateral_mint,
            market.ticket_mint,
            TokenKind::AdapterCollateral,
            1_00,
            airspace_authority,
        )
        .await?;

        Ok(())
    }

    pub async fn register_tickets_position_metadatata_impl(
        &self,
        position_token_mint: Pubkey,
        underlying_token_mint: Pubkey,
        token_kind: TokenKind,
        value_modifier: u16,
        airspace_authority: &Keypair,
    ) -> Result<()> {
        let margin_config_ix = MarginConfigIxBuilder::new(
            self.airspace,
            self.client.payer().pubkey(),
            Some(airspace_authority.pubkey()),
        );

        self.sign_send_transaction(
            &[margin_config_ix.configure_token(
                position_token_mint,
                Some(TokenConfigUpdate {
                    underlying_mint: underlying_token_mint,
                    admin: TokenAdmin::Adapter(jet_fixed_term::ID),
                    token_kind,
                    value_modifier,
                    max_staleness: 0,
                }),
            )],
            &[airspace_authority],
        )
        .await?;

        Ok(())
    }

    pub async fn register_adapter(&self, adapter: &Pubkey, authority: &Keypair) -> Result<()> {
        let ix = MarginConfigIxBuilder::new(
            self.airspace,
            self.client.payer().pubkey(),
            Some(authority.pubkey()),
        )
        .configure_adapter(*adapter, true);

        send_and_confirm(&self.client, &[ix], &[authority]).await?;
        Ok(())
    }

    pub async fn simulate_new_order(
        &self,
        params: OrderParams,
        side: agnostic_orderbook::state::Side,
    ) -> Result<OrderSummary> {
        let mut eq = self.load_event_queue().await?;
        let mut orderbook = self.load_orderbook().await?;
        orderbook
            .inner()?
            .new_order(
                params.as_new_order_params(side, CallbackInfo::default()),
                &mut eq.inner()?,
                MIN_ORDER_SIZE,
            )
            .map_err(anyhow::Error::new)
    }

    pub async fn simulate_new_order_with_fees(
        &self,
        mut params: OrderParams,
        side: agnostic_orderbook::state::Side,
    ) -> Result<OrderSummary> {
        let mut eq = self.load_event_queue().await?;
        let mut orderbook = self.load_orderbook().await?;
        let market = self.load_market().await?;
        params.max_ticket_qty = market.borrow_order_qty(params.max_ticket_qty);
        params.max_underlying_token_qty = market.borrow_order_qty(params.max_underlying_token_qty);
        orderbook
            .inner()?
            .new_order(
                params.as_new_order_params(side, CallbackInfo::default()),
                &mut eq.inner()?,
                MIN_ORDER_SIZE,
            )
            .map_err(anyhow::Error::new)
    }
}

#[derive(Clone)]
pub struct OwnedBook {
    bids: Vec<u8>,
    asks: Vec<u8>,
}

impl OwnedBook {
    pub fn inner(&mut self) -> Result<OrderBookState<CallbackInfo>> {
        Ok(OrderBookState {
            bids: Slab::from_buffer(&mut self.bids, AccountTag::Bids)?,
            asks: Slab::from_buffer(&mut self.asks, AccountTag::Asks)?,
        })
    }
    pub fn bids(&mut self) -> Result<Vec<LeafNode>> {
        Ok(self.inner()?.bids.into_iter(true).collect())
    }
    pub fn asks(&mut self) -> Result<Vec<LeafNode>> {
        Ok(self.inner()?.asks.into_iter(true).collect())
    }

    pub fn asks_order_callback(&mut self, pos: usize) -> Result<CallbackInfo> {
        let key = self.asks()?[pos].key;
        let handle = self.inner()?.asks.find_by_key(key).unwrap();

        Ok(*self.inner()?.asks.get_callback_info(handle))
    }
    pub fn bids_order_callback(&mut self, pos: usize) -> Result<CallbackInfo> {
        let key = self.bids()?[pos].key;
        let handle = self.inner()?.bids.find_by_key(key).unwrap();

        Ok(*self.inner()?.bids.get_callback_info(handle))
    }
}

impl TestManager {
    pub async fn load_market(&self) -> Result<Market> {
        self.load_anchor(&self.ix_builder.market()).await
    }
    pub async fn load_manager_token_vault(&self) -> Result<TokenAccount> {
        let vault = self.ix_builder.vault();

        self.load_anchor(&vault).await
    }
    pub async fn load_event_queue(&self) -> Result<OwnedEventQueue> {
        let data = self.load_data(&self.orderbook.event_queue.pubkey()).await?;

        Ok(OwnedEventQueue::from(data))
    }
    pub async fn load_orderbook_market_state(&self) -> Result<OrderBookMarketState> {
        let key = self.ix_builder.orderbook_state();
        let mut data = self.load_data(&key).await?;

        Ok(
            *OrderBookMarketState::from_buffer(&mut data, AccountTag::Market)
                .map_err(anyhow::Error::from)?,
        )
    }
    pub async fn load_orderbook(&self) -> Result<OwnedBook> {
        let bids_data = self.load_data(&self.orderbook.bids.pubkey()).await?;
        let asks_data = self.load_data(&self.orderbook.asks.pubkey()).await?;

        Ok(OwnedBook {
            bids: bids_data,
            asks: asks_data,
        })
    }

    pub async fn collected_fees(&self) -> Result<u64> {
        let key = self.load_market().await?.fee_vault;
        let vault = self.load_anchor::<TokenAccount>(&key).await?;

        Ok(vault.amount)
    }

    pub async fn load_data(&self, key: &Pubkey) -> Result<Vec<u8>> {
        Ok(self
            .client
            .get_account(key)
            .await?
            .ok_or_else(|| anyhow::Error::msg("failed to fetch key: {key}"))?
            .data)
    }

    pub async fn load_anchor<T: AccountDeserialize>(&self, key: &Pubkey) -> Result<T> {
        let data = self.load_data(key).await?;

        T::try_deserialize(&mut data.as_slice()).map_err(anyhow::Error::from)
    }

    pub async fn load_margin_user(&self, margin_account: &Pubkey) -> Result<MarginUser> {
        let key = self.ix_builder.margin_user_account(*margin_account);

        self.load_anchor(&key).await
    }

    pub async fn load_mature_deposits(
        &self,
        margin_account: &Pubkey,
    ) -> Result<Vec<(Pubkey, TermDeposit)>> {
        let current_time = self.client.get_clock().await?.unix_timestamp;

        let deposits = self
            .client
            .get_program_accounts(
                &jet_fixed_term::ID,
                Some(std::mem::size_of::<TermDeposit>() + 8),
            )
            .await?
            .into_iter()
            .filter_map(|(k, a)| {
                if let Ok(deposit) = TermDeposit::try_deserialize(&mut a.data.as_slice()) {
                    if &deposit.owner == margin_account && deposit.matures_at <= current_time {
                        return Some((k, deposit));
                    }
                }
                None
            })
            .collect::<Vec<_>>();
        Ok(deposits)
    }

    pub async fn load_outstanding_loans(
        &self,
        margin_account: Pubkey,
    ) -> Result<Vec<(Pubkey, TermLoan)>> {
        let margin_user = self.ix_builder.margin_user(margin_account).address;
        let loans = self
            .client
            .get_program_accounts(
                &jet_fixed_term::ID,
                Some(std::mem::size_of::<TermLoan>() + 8),
            )
            .await?
            .into_iter()
            .filter_map(|(k, a)| {
                if let Ok(loan) = TermLoan::try_deserialize(&mut a.data.as_slice()) {
                    if loan.margin_user == margin_user {
                        return Some((k, loan));
                    }
                }
                None
            })
            .collect::<Vec<_>>();

        Ok(loans)
    }
}

#[async_trait]
pub trait GenerateProxy {
    async fn generate(
        ctx: Arc<MarginTestContext>,
        manager: Arc<TestManager>,
        owner: &Keypair,
        _seed: u16,
    ) -> Result<Self>
    where
        Self: Sized;
}

#[async_trait]
impl GenerateProxy for NoProxy {
    async fn generate(
        _ctx: Arc<MarginTestContext>,
        _manager: Arc<TestManager>,
        owner: &Keypair,
        _seed: u16,
    ) -> Result<Self> {
        Ok(NoProxy(owner.pubkey()))
    }
}

#[async_trait]
impl GenerateProxy for MarginIxBuilder {
    async fn generate(
        ctx: Arc<MarginTestContext>,
        manager: Arc<TestManager>,
        owner: &Keypair,
        seed: u16,
    ) -> Result<Self> {
        ctx.issue_permit(owner.pubkey()).await?;
        let margin = MarginIxBuilder::new(manager.airspace, owner.pubkey(), seed);
        manager
            .sign_send_transaction(&[margin.create_account()], &[owner])
            .await?;

        Ok(margin)
    }
}

pub struct FixedTermUser<P: Proxy> {
    pub owner: Keypair,
    pub proxy: P,
    pub token_acc: Pubkey,
    manager: Arc<TestManager>,
    client: Arc<dyn SolanaRpcClient>,
}

impl<P: Proxy> FixedTermUser<P> {
    pub fn new(manager: Arc<TestManager>, owner: Keypair, proxy: P) -> Result<Self> {
        let token_acc =
            get_associated_token_address(&proxy.pubkey(), &manager.ix_builder.token_mint());

        Ok(Self {
            owner,
            proxy,
            token_acc,
            client: manager.client.clone(),
            manager,
        })
    }

    pub async fn new_funded(manager: Arc<TestManager>, owner: Keypair, proxy: P) -> Result<Self> {
        let user = Self::new(manager, owner, proxy)?;
        user.fund().await?;
        Ok(user)
    }
}

impl FixedTermUser<RefreshingProxy<MarginIxBuilder>> {
    pub fn new_refreshing(manager: Arc<TestManager>, owner: Keypair, seed: u16) -> Self {
        let proxy = RefreshingProxy::full(&manager.client, &owner, seed, manager.airspace);
        let token_acc =
            get_associated_token_address(&proxy.pubkey(), &manager.ix_builder.token_mint());

        Self {
            owner,
            proxy,
            token_acc,
            client: manager.client.clone(),
            manager,
        }
    }
}

impl<P: Proxy + GenerateProxy> FixedTermUser<P> {
    pub async fn generate_for(
        ctx: Arc<MarginTestContext>,
        manager: Arc<TestManager>,
        owner: Keypair,
        seed: u16,
    ) -> Result<Self> {
        let proxy = P::generate(ctx, manager.clone(), &owner, seed).await?;
        Self::new(manager, owner, proxy)
    }

    pub async fn generate(ctx: Arc<MarginTestContext>, manager: Arc<TestManager>) -> Result<Self> {
        let owner = manager.keygen.generate_key();
        manager
            .client
            .airdrop(&owner.pubkey(), 10 * LAMPORTS_PER_SOL)
            .await?;
        let proxy = P::generate(ctx, manager.clone(), &owner, 0).await?;
        Self::new(manager, owner, proxy)
    }

    pub async fn generate_funded(
        ctx: Arc<MarginTestContext>,
        manager: Arc<TestManager>,
    ) -> Result<Self> {
        let user = Self::generate(ctx, manager).await?;
        user.fund().await?;
        Ok(user)
    }
}

impl<P: Proxy> FixedTermUser<P> {
    pub async fn fund(&self) -> Result<()> {
        let create_token = create_associated_token_account(
            &self.manager.client.payer().pubkey(),
            &self.proxy.pubkey(),
            &self.manager.ix_builder.token_mint(),
            &spl_token::id(),
        );
        let create_ticket = create_associated_token_account(
            &self.manager.client.payer().pubkey(),
            &self.proxy.pubkey(),
            &self.manager.ix_builder.ticket_mint(),
            &spl_token::id(),
        );
        let fund = spl_token::instruction::mint_to(
            &spl_token::ID,
            &self.manager.ix_builder.token_mint(),
            &self.token_acc,
            &self.manager.mint_authority.pubkey(),
            &[],
            STARTING_TOKENS,
        )?;

        self.manager
            .sign_send_transaction(&[create_token, create_ticket, fund], &[])
            .await?;

        Ok(())
    }

    pub async fn initialize_margin_user(&self) -> Result<Signature> {
        let ix = self
            .manager
            .ix_builder
            .initialize_margin_user(self.proxy.pubkey());
        self.client
            .send_and_confirm_1tx(&[self.proxy.invoke_signed(ix)], &[&self.owner])
            .await
    }

    pub async fn convert_tokens(&self, amount: u64) -> Result<Signature> {
        let ix = self
            .manager
            .ix_builder
            .convert_tokens(self.proxy.pubkey(), None, None, amount);
        self.client
            .send_and_confirm_1tx(&[self.proxy.invoke_signed(ix)], &[&self.owner])
            .await
    }

    pub async fn stake_tokens(&self, amount: u64, seed: &[u8]) -> Result<Signature> {
        let ix = self
            .manager
            .ix_builder
            .stake_tickets(self.proxy.pubkey(), None, amount, seed);

        self.client
            .send_and_confirm_1tx(&[self.proxy.invoke_signed(ix)], &[&self.owner])
            .await
    }

    pub async fn redeem_claim_ticket(&self, seed: &[u8]) -> Result<Signature> {
        let ticket = self.claim_ticket_key(seed);
        let ix = self
            .manager
            .ix_builder
            .redeem_deposit(self.proxy.pubkey(), ticket, None);
        self.client
            .send_and_confirm_1tx(&[self.proxy.invoke_signed(ix)], &[&self.owner])
            .await
    }

    pub async fn sell_tickets_order(&self, params: OrderParams) -> Result<Signature> {
        let borrow =
            self.manager
                .ix_builder
                .sell_tickets_order(self.proxy.pubkey(), None, None, params);
        self.client
            .send_and_confirm_1tx(&[self.proxy.invoke_signed(borrow)], &[&self.owner])
            .await
    }

    pub async fn margin_sell_tickets_order(
        &self,
        params: OrderParams,
    ) -> Result<Vec<TransactionBuilder>> {
        let ix = self.manager.ix_builder.margin_sell_tickets_order(
            self.proxy.pubkey(),
            None,
            None,
            params,
        );
        self.proxy
            .refresh_and_invoke_signed(ix, clone(&self.owner))
            .await
    }

    pub async fn refresh_and_margin_borrow_order(
        &self,
        params: OrderParams,
    ) -> Result<Vec<TransactionBuilder>> {
        let mut txs = self.proxy.refresh().await?;
        txs.push(self.margin_borrow_order(params).await?);

        Ok(txs)
    }

    pub async fn margin_borrow_order(&self, params: OrderParams) -> Result<TransactionBuilder> {
        let debt_seqno = self.load_margin_user().await?.debt.next_new_loan_seqno();
        let borrow =
            self.manager
                .ix_builder
                .margin_borrow_order(self.proxy.pubkey(), params, debt_seqno);
        Ok(self
            .proxy
            .invoke_signed(borrow)
            .with_signers(&[clone(&self.owner)]))
    }

    pub async fn refresh_and_margin_lend_order(
        &self,
        params: OrderParams,
    ) -> Result<Vec<TransactionBuilder>> {
        let mut txs = self.proxy.refresh().await?;
        txs.push(self.margin_lend_order(params).await?);

        Ok(txs)
    }

    pub async fn margin_lend_order(&self, params: OrderParams) -> Result<TransactionBuilder> {
        let deposit_seqno = self
            .load_margin_user()
            .await?
            .assets
            .next_new_deposit_seqno();
        let ix = self.manager.ix_builder.margin_lend_order(
            self.proxy.pubkey(),
            None,
            params,
            deposit_seqno,
        );
        Ok(self
            .proxy
            .invoke_signed(ix)
            .with_signers(&[clone(&self.owner)]))
    }

    pub async fn lend_order(&self, params: OrderParams, seed: &[u8]) -> Result<Signature> {
        let lend =
            self.manager
                .ix_builder
                .lend_order(self.proxy.pubkey(), None, None, params, seed);
        self.client
            .send_and_confirm_1tx(&[self.proxy.invoke_signed(lend)], &[&self.owner])
            .await
    }

    pub async fn cancel_order(&self, order_id: u128) -> Result<Signature> {
        let cancel = self
            .manager
            .ix_builder
            .cancel_order(self.proxy.pubkey(), order_id);
        self.client
            .send_and_confirm_1tx(&[self.proxy.invoke_signed(cancel)], &[&self.owner])
            .await
    }

    pub async fn settle(&self) -> Result<Signature> {
        let settle = self.manager.ix_builder.settle(self.proxy.pubkey());
        self.client.send_and_confirm_1tx(&[settle], &[]).await
    }

    pub async fn set_roll_config(
        &self,
        side: MarketSide,
        config: AutoRollConfig,
    ) -> Result<Signature> {
        let set_config =
            self.manager
                .ix_builder
                .configure_auto_roll(self.proxy.pubkey(), side, config);
        self.client
            .send_and_confirm_1tx(&[self.proxy.invoke_signed(set_config)], &[&self.owner])
            .await
    }

    pub async fn repay(&self, term_loan_seqno: u64, amount: u64) -> Result<Signature> {
        // we are not sure if the user or a crank paid for the rent, so we just fetch the data
        let payer = {
            let loan_key = self.term_loan_key(&term_loan_seqno.to_le_bytes());
            let loan: TermLoan = self.load_anchor(&loan_key).await?;

            loan.payer
        };
        let source = get_associated_token_address(
            &self.proxy.pubkey(),
            &self.manager.ix_builder.token_mint(),
        );
        let repay = self.manager.ix_builder.margin_repay(
            &self.proxy.pubkey(),
            &payer,
            &self.proxy.pubkey(),
            &source,
            term_loan_seqno,
            amount,
        );

        self.client
            .send_and_confirm_1tx(&[self.proxy.invoke_signed(repay)], &[&self.owner])
            .await
    }

    pub async fn get_active_term_loans(&self) -> Result<Vec<TermLoan>> {
        let mut loans = vec![];

        let user = self.load_margin_user().await?;
        for seqno in user.debt.active_loans() {
            loans.push(self.load_term_loan(seqno).await?);
        }

        Ok(loans)
    }

    pub async fn load_anchor<T: AccountDeserialize>(&self, key: &Pubkey) -> Result<T> {
        let data = self
            .client
            .get_account(key)
            .await?
            .ok_or_else(|| anyhow::Error::msg("failed to fetch key: [key]"))?
            .data;

        T::try_deserialize(&mut data.as_slice()).map_err(anyhow::Error::from)
    }

    pub async fn try_repay_all(&self) -> Result<()> {
        let mut outstanding_loans = self
            .manager
            .load_outstanding_loans(self.proxy.pubkey())
            .await?;
        outstanding_loans.sort_by(|a, b| a.1.sequence_number.cmp(&b.1.sequence_number));

        for (_, loan) in outstanding_loans {
            self.repay(loan.sequence_number, loan.balance).await?;
        }
        Ok(())
    }
}

impl<P: Proxy> FixedTermUser<P> {
    pub fn claim_ticket_key(&self, seed: &[u8]) -> Pubkey {
        self.manager
            .ix_builder
            .term_deposit_key(&self.proxy.pubkey(), seed)
    }

    pub fn term_deposit_key(&self, seed: &[u8]) -> Pubkey {
        self.manager
            .ix_builder
            .term_deposit_key(&self.proxy.pubkey(), seed)
    }
    pub fn term_loan_key(&self, seed: &[u8]) -> Pubkey {
        let margin_user = self
            .manager
            .ix_builder
            .margin_user_account(self.proxy.pubkey());
        self.manager.ix_builder.term_loan_key(&margin_user, seed)
    }
    pub async fn load_term_deposit(&self, seed: &[u8]) -> Result<TermDeposit> {
        let key = self.claim_ticket_key(seed);

        self.manager.load_anchor(&key).await
    }

    pub async fn load_term_loan(&self, seqno: u64) -> Result<TermLoan> {
        let key = self.term_loan_key(&seqno.to_le_bytes());

        self.manager.load_anchor(&key).await
    }
    /// loads the current state of the user token wallet
    pub async fn tokens(&self) -> Result<u64> {
        let key = get_associated_token_address(
            &self.proxy.pubkey(),
            &self.manager.ix_builder.token_mint(),
        );

        self.manager
            .load_anchor::<TokenAccount>(&key)
            .await
            .map(|a| a.amount)
    }

    /// loads the current state of the user ticket wallet
    pub async fn tickets(&self) -> Result<u64> {
        let key = get_associated_token_address(
            &self.proxy.pubkey(),
            &self.manager.ix_builder.ticket_mint(),
        );
        self.manager
            .load_anchor::<TokenAccount>(&key)
            .await
            .map(|a| a.amount)
    }

    /// loads the current state of the user collateral balance
    pub async fn collateral(&self) -> Result<u64> {
        let key = self
            .manager
            .ix_builder
            .margin_user(self.proxy.pubkey())
            .ticket_collateral;
        self.manager
            .load_anchor::<TokenAccount>(&key)
            .await
            .map(|a| a.amount)
    }

    /// loads the current state of the user claim balance
    pub async fn claims(&self) -> Result<u64> {
        let key = self
            .manager
            .ix_builder
            .margin_user(self.proxy.pubkey())
            .claims;
        self.manager
            .load_anchor::<TokenAccount>(&key)
            .await
            .map(|a| a.amount)
    }

    pub async fn load_margin_user(&self) -> Result<MarginUser> {
        let key = self
            .manager
            .ix_builder
            .margin_user_account(self.proxy.pubkey());
        self.manager.load_anchor(&key).await
    }
}

pub struct OrderAmount {
    pub base: u64,
    pub quote: u64,
    pub price: u64,
}

impl OrderAmount {
    /// rate is in basis points
    pub fn from_quote_amount_rate(quote: u64, rate_bps: u64) -> Self {
        let base = quote + quote * rate_bps / 10_000;
        let price = Fp32::from(quote) / base;

        OrderAmount {
            base: u64::MAX,
            quote,
            price: price.downcast_u64().unwrap(),
        }
    }

    /// rate is in basis points
    pub fn from_base_amount_rate(base: u64, rate_bps: u64) -> Self {
        let quote = base * 10_000 / (rate_bps + 10_000);
        let price = Fp32::from(quote) / base;

        OrderAmount {
            base,
            quote: u64::MAX,
            price: price.downcast_u64().unwrap(),
        }
    }

    pub fn params_from_quote_amount_rate(amount: u64, rate_bps: u64) -> OrderParams {
        Self::from_quote_amount_rate(amount, rate_bps).default_order_params()
    }

    pub fn default_order_params(&self) -> OrderParams {
        OrderParams {
            max_ticket_qty: self.base,
            max_underlying_token_qty: self.quote,
            limit_price: self.price,
            match_limit: 1_000,
            post_only: false,
            post_allowed: true,
            auto_stake: true,
            auto_roll: false,
        }
    }
}

pub async fn generate_test_mint(client: &SolanaTestContext) -> Result<(Pubkey, Keypair)> {
    let mint = client.generate_key();
    let mint_authority = client.generate_key();
    initialize_test_mint(client, &mint, &mint_authority.pubkey()).await?;

    Ok((mint.pubkey(), mint_authority))
}

pub async fn initialize_test_mint(
    client: &SolanaTestContext,
    mint: &Keypair,
    mint_authority: &Pubkey,
) -> Result<()> {
    let payer = client.rpc.payer();
    let recent_blockhash = client.rpc.get_latest_blockhash().await?;
    let rent = client
        .rpc
        .get_minimum_balance_for_rent_exemption(Mint::LEN)
        .await?;
    let transaction =
        initialize_test_mint_transaction(mint, payer, mint_authority, 6, rent, recent_blockhash);
    client
        .rpc
        .send_and_confirm_transaction(&transaction)
        .await?;

    Ok(())
}

pub fn initialize_test_mint_transaction(
    mint_keypair: &Keypair,
    payer: &Keypair,
    mint_authority: &Pubkey,
    decimals: u8,
    rent: u64,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let create_mint_account = {
            let space = Mint::LEN;
            system_instruction::create_account(
                &payer.pubkey(),
                &mint_keypair.pubkey(),
                rent,
                space as u64,
                &spl_token::ID,
            )
        };
        let initialize_mint = initialize_mint(
            &spl_token::ID,
            &mint_keypair.pubkey(),
            mint_authority,
            Some(mint_authority),
            decimals,
        )
        .unwrap();

        &[create_mint_account, initialize_mint]
    };
    let signing_keypairs = &[payer, mint_keypair];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}

pub async fn create_and_fund_fixed_term_market_margin_user(
    ctx: &Arc<MarginTestContext>,
    manager: Arc<TestManager>,
    pool_positions: Vec<(Pubkey, u64, u64)>,
) -> FixedTermUser<RefreshingProxy<MarginIxBuilder>> {
    // set up user
    let user = setup_user(ctx, pool_positions).await.unwrap();
    let wallet = user.user.signer;

    // set up proxy
    let proxy = RefreshingProxy::full(&ctx.solana.rpc, &wallet, 0, ctx.airspace);

    let user = FixedTermUser::new_funded(manager.clone(), wallet, proxy.clone())
        .await
        .unwrap();
    user.initialize_margin_user().await.unwrap();

    let margin_user = user.load_margin_user().await.unwrap();
    assert_eq!(margin_user.market, manager.ix_builder.market());

    user
}
