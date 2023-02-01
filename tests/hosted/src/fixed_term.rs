use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

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
    margin::{
        instructions::MarketSide,
        state::{AutoRollConfig, MarginUser, TermLoan},
    },
    orderbook::state::{event_queue_len, orderbook_slab_len, CallbackInfo, OrderParams},
    tickets::state::TermDeposit,
};
use jet_instructions::{
    airspace::derive_airspace, fixed_term::derive_market, margin::MarginConfigIxBuilder,
};
use jet_margin::{TokenAdmin, TokenConfigUpdate, TokenKind};
use jet_margin_sdk::{
    fixed_term::{
        event_consumer::{download_markets, EventConsumer},
        fixed_term_address,
        settler::{settle_margin_users_loop, SettleMarginUsersConfig},
        FixedTermIxBuilder, OrderBookAddresses, OwnedEventQueue,
    },
    ix_builder::{
        get_control_authority_address, get_metadata_address, ControlIxBuilder, MarginIxBuilder,
    },
    margin_integrator::{NoProxy, Proxy, RefreshingProxy},
    solana::{
        keypair::clone,
        transaction::{SendTransactionBuilder, TransactionBuilder, WithSigner},
    },
    tx_builder::{
        fixed_term::FixedTermPositionRefresher, global_initialize_instructions, MarginTxBuilder,
    },
    util::no_dupe_queue::AsyncNoDupeQueue,
};
use jet_program_common::Fp32;
use jet_simulation::{create_wallet, send_and_confirm, solana_rpc_api::SolanaRpcClient};
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

#[derive(Debug, Default, Clone)]
pub struct Keys<T>(HashMap<String, T>);

impl<T> Keys<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn insert(&mut self, k: &str, v: T) {
        self.0.insert(k.into(), v);
    }
    pub fn unwrap(&self, k: &str) -> Result<&T> {
        self.0
            .get(k)
            .ok_or_else(|| anyhow::Error::msg("missing key: {k}"))
    }

    pub fn inner(&self) -> &HashMap<String, T> {
        &self.0
    }
}

pub struct TestManager {
    pub client: Arc<dyn SolanaRpcClient>,
    pub keygen: Arc<dyn Keygen>,
    pub ix_builder: FixedTermIxBuilder,
    pub event_consumer: Arc<EventConsumer>,
    pub kps: Keys<Keypair>,
    pub keys: Keys<Pubkey>,
    pub margin_accounts_to_settle: AsyncNoDupeQueue<Pubkey>,
    airspace: String,
}

impl Clone for TestManager {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            ix_builder: self.ix_builder.clone(),
            event_consumer: self.event_consumer.clone(),
            kps: Keys(
                self.kps
                    .0
                    .iter()
                    .map(|(k, v)| (k.clone(), Keypair::from_bytes(&v.to_bytes()).unwrap()))
                    .collect(),
            ),
            keys: self.keys.clone(),
            keygen: self.keygen.clone(),
            margin_accounts_to_settle: AsyncNoDupeQueue::new(),
            airspace: self.airspace.clone(),
        }
    }
}

impl TestManager {
    pub async fn full(client: &MarginTestContext) -> Result<Self> {
        let mint = client.solana.generate_key();
        let oracle = TokenManager::new(client.solana.clone())
            .create_oracle(&mint.pubkey())
            .await?;
        let ticket_mint = fixed_term_address(&[
            jet_fixed_term::seeds::TICKET_MINT,
            derive_market(
                &client.margin.airspace_address(),
                &mint.pubkey(),
                MARKET_SEED,
            )
            .as_ref(),
        ]);
        let ticket_oracle = TokenManager::new(client.solana.clone())
            .create_oracle(&ticket_mint)
            .await?;
        TestManager::new(
            client.solana.clone(),
            client.margin.airspace(),
            &mint,
            &client.generate_key(),
            &client.generate_key(),
            &client.generate_key(),
            oracle.price,
            ticket_oracle.price,
        )
        .await?
        .with_crank()
        .await?
        .with_margin(&client.airspace_authority)
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        client: SolanaTestContext,
        airspace: String,
        mint: &Keypair,
        eq_kp: &Keypair,
        bids_kp: &Keypair,
        asks_kp: &Keypair,
        underlying_oracle: Pubkey,
        ticket_oracle: Pubkey,
    ) -> Result<Self> {
        let SolanaTestContext {
            rpc: client,
            keygen,
        } = client;
        let payer = client.payer();
        let recent_blockhash = client.get_latest_blockhash().await?;
        let rent = client
            .get_minimum_balance_for_rent_exemption(Mint::LEN)
            .await?;
        let transaction = initialize_test_mint_transaction(mint, payer, 6, rent, recent_blockhash);
        client.send_and_confirm_transaction(&transaction).await?;

        let ix_builder = FixedTermIxBuilder::new_from_seed(
            payer.pubkey(),
            &derive_airspace(&airspace),
            &mint.pubkey(),
            MARKET_SEED,
            payer.pubkey(),
            underlying_oracle,
            ticket_oracle,
            None,
            OrderBookAddresses {
                bids: bids_kp.pubkey(),
                asks: asks_kp.pubkey(),
                event_queue: eq_kp.pubkey(),
            },
        );
        let mut this = Self {
            client: client.clone(),
            event_consumer: Arc::new(EventConsumer::new(client.clone())),
            keygen,
            ix_builder,
            kps: Keys::new(),
            keys: Keys::new(),
            margin_accounts_to_settle: Default::default(),
            airspace,
        };
        this.insert_kp("token_mint", clone(mint));

        let init_eq = {
            let rent = this
                .client
                .get_minimum_balance_for_rent_exemption(event_queue_len(EVENT_QUEUE_CAPACITY))
                .await?;
            this.ix_builder
                .initialize_event_queue(&eq_kp.pubkey(), EVENT_QUEUE_CAPACITY, rent)
        };

        let init_bids = {
            let rent = this
                .client
                .get_minimum_balance_for_rent_exemption(orderbook_slab_len(ORDERBOOK_CAPACITY))
                .await?;
            this.ix_builder
                .initialize_orderbook_slab(&bids_kp.pubkey(), ORDERBOOK_CAPACITY, rent)
        };
        let init_asks = {
            let rent = this
                .client
                .get_minimum_balance_for_rent_exemption(orderbook_slab_len(ORDERBOOK_CAPACITY))
                .await?;
            this.ix_builder
                .initialize_orderbook_slab(&asks_kp.pubkey(), ORDERBOOK_CAPACITY, rent)
        };
        this.insert_kp("eq", clone(eq_kp));
        this.insert_kp("bids", clone(bids_kp));
        this.insert_kp("asks", clone(asks_kp));

        let payer = this.client.payer().pubkey();
        let init_fee_destination = this
            .ix_builder
            .init_default_fee_destination(&payer)
            .unwrap();
        let init_manager = this.ix_builder.initialize_market(
            payer,
            MARKET_TAG,
            MARKET_SEED,
            BORROW_TENOR,
            LEND_TENOR,
            ORIGINATION_FEE,
        );
        let init_orderbook = this
            .ix_builder
            .initialize_orderbook(this.client.payer().pubkey(), MIN_ORDER_SIZE);

        this.sign_send_transaction(&[init_eq, init_bids, init_asks, init_fee_destination], None)
            .await?;
        this.sign_send_transaction(&[init_manager, init_orderbook], None)
            .await?;

        Ok(this)
    }

    pub async fn with_crank(self) -> Result<Self> {
        let auth_crank = self
            .ix_builder
            .authorize_crank(self.client.payer().pubkey());

        self.sign_send_transaction(&[auth_crank], None).await?;
        Ok(self)
    }

    /// set up metadata authorization for margin to invoke fixed term and
    /// register relevant positions.
    pub async fn with_margin(self, airspace_authority: &Keypair) -> Result<Self> {
        self.create_authority_if_missing().await?;
        self.register_adapter_if_unregistered(&jet_fixed_term::ID)
            .await?;
        self.register_tickets_position_metadatata(airspace_authority)
            .await?;
        register_deposit(
            &self.client,
            self.airspace_address(),
            airspace_authority,
            self.ix_builder.token_mint(),
        )
        .await?;
        register_deposit(
            &self.client,
            self.airspace_address(),
            airspace_authority,
            self.ix_builder.ticket_mint(),
        )
        .await?;

        Ok(self)
    }

    pub async fn sign_send_transaction(
        &self,
        instructions: &[Instruction],
        add_signers: Option<&[&Keypair]>,
    ) -> Result<Signature> {
        let mut signers = Vec::<&Keypair>::new();
        let owned_kps = self.kps.inner();
        let mut keypairs = owned_kps.iter().map(|(_, v)| v).collect::<Vec<&Keypair>>();
        if let Some(extra_signers) = add_signers {
            keypairs.extend_from_slice(extra_signers);
        }
        keypairs.push(self.client.payer());

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

    pub fn airspace_address(&self) -> Pubkey {
        derive_airspace(&self.airspace)
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

            println!("pending = {pending}");
            self.event_consumer.consume().await?;
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

    pub async fn pause_ticket_redemption(&self) -> Result<Signature> {
        let pause = self.ix_builder.pause_ticket_redemption();

        self.sign_send_transaction(&[pause], None).await
    }

    pub async fn resume_ticket_redemption(&self) -> Result<Signature> {
        let resume = self.ix_builder.resume_ticket_redemption();

        self.sign_send_transaction(&[resume], None).await
    }

    pub async fn pause_orders(&self) -> Result<Signature> {
        let pause = self.ix_builder.pause_order_matching();

        self.sign_send_transaction(&[pause], None).await
    }

    pub async fn resume_orders(&self) -> Result<()> {
        loop {
            if self.load_orderbook_market_state().await?.pause_matching == (false as u8) {
                break;
            }

            let resume = self.ix_builder.resume_order_matching();
            self.sign_send_transaction(&[resume], None).await?;
        }

        Ok(())
    }

    pub fn insert_kp(&mut self, k: &str, kp: Keypair) {
        self.keys.insert(k, kp.pubkey());
        self.kps.insert(k, kp);
    }
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

    pub async fn register_adapter_if_unregistered(&self, adapter: &Pubkey) -> Result<()> {
        if self
            .client
            .get_account(&get_metadata_address(adapter))
            .await?
            .is_none()
        {
            self.register_adapter(adapter).await?;
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
            self.airspace_address(),
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
            Some(&[airspace_authority]),
        )
        .await?;

        Ok(())
    }

    pub async fn register_adapter(&self, adapter: &Pubkey) -> Result<()> {
        let ix = ControlIxBuilder::new(self.client.payer().pubkey()).register_adapter(adapter);

        send_and_confirm(&self.client, &[ix], &[]).await?;
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
        let data = self.load_account("eq").await?;

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
        let bids_data = self.load_account("bids").await?;
        let asks_data = self.load_account("asks").await?;

        Ok(OwnedBook {
            bids: bids_data,
            asks: asks_data,
        })
    }

    pub async fn load_account(&self, k: &str) -> Result<Vec<u8>> {
        self.load_data(self.keys.unwrap(k)?).await
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
}

#[async_trait]
pub trait GenerateProxy {
    async fn generate(manager: Arc<TestManager>, owner: &Keypair) -> Result<Self>
    where
        Self: Sized;
}

#[async_trait]
impl GenerateProxy for NoProxy {
    async fn generate(_manager: Arc<TestManager>, owner: &Keypair) -> Result<Self> {
        Ok(NoProxy(owner.pubkey()))
    }
}

#[async_trait]
impl GenerateProxy for MarginIxBuilder {
    async fn generate(manager: Arc<TestManager>, owner: &Keypair) -> Result<Self> {
        let margin = MarginIxBuilder::new(manager.airspace.clone(), owner.pubkey(), 0);
        manager
            .sign_send_transaction(&[margin.create_account()], Some(&[owner]))
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
    pub fn new_with_proxy(manager: Arc<TestManager>, owner: Keypair, proxy: P) -> Result<Self> {
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

    pub async fn new_with_proxy_funded(
        manager: Arc<TestManager>,
        owner: Keypair,
        proxy: P,
    ) -> Result<Self> {
        let user = Self::new_with_proxy(manager, owner, proxy)?;
        user.fund().await?;
        Ok(user)
    }
}

impl<P: Proxy + GenerateProxy> FixedTermUser<P> {
    pub async fn new(manager: Arc<TestManager>) -> Result<Self> {
        let owner = create_wallet(&manager.client, 10 * LAMPORTS_PER_SOL).await?;
        let proxy = P::generate(manager.clone(), &owner).await?;
        Self::new_with_proxy(manager, owner, proxy)
    }

    pub async fn new_funded(manager: Arc<TestManager>) -> Result<Self> {
        let user = Self::new(manager).await?;
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
            &self.manager.ix_builder.token_mint(),
            &[],
            STARTING_TOKENS,
        )?;

        self.manager
            .sign_send_transaction(&[create_token, create_ticket, fund], Some(&[&self.owner]))
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
            .redeem_ticket(self.proxy.pubkey(), ticket, None);
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
        let borrow = self.manager.ix_builder.margin_borrow_order(
            self.proxy.pubkey(),
            None,
            params,
            debt_seqno,
        );
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

    pub async fn load_anchor<T: AccountDeserialize>(&self, key: &Pubkey) -> Result<T> {
        let data = self
            .client
            .get_account(key)
            .await?
            .ok_or_else(|| anyhow::Error::msg("failed to fetch key: [key]"))?
            .data;

        T::try_deserialize(&mut data.as_slice()).map_err(anyhow::Error::from)
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

pub fn initialize_test_mint_transaction(
    mint_keypair: &Keypair,
    payer: &Keypair,
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
            &mint_keypair.pubkey(),
            Some(&mint_keypair.pubkey()),
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

pub async fn create_fixed_term_market_margin_user(
    ctx: &Arc<MarginTestContext>,
    manager: Arc<TestManager>,
    pool_positions: Vec<(Pubkey, u64, u64)>,
) -> FixedTermUser<RefreshingProxy<MarginIxBuilder>> {
    let client = manager.client.clone();

    // set up user
    let user = setup_user(ctx, pool_positions).await.unwrap();
    let margin = user.user.tx.ix.clone();
    let wallet = user.user.signer;

    // set up proxy
    let proxy = RefreshingProxy {
        proxy: margin.clone(),
        refreshers: vec![
            Arc::new(MarginTxBuilder::new(
                client.clone(),
                None,
                wallet.pubkey(),
                0,
                ctx.margin.airspace(),
            )),
            Arc::new(
                FixedTermPositionRefresher::new(
                    margin.pubkey(),
                    client.clone(),
                    &[manager.ix_builder.market()],
                )
                .await
                .unwrap(),
            ),
        ],
    };

    let user = FixedTermUser::new_with_proxy_funded(manager.clone(), wallet, proxy.clone())
        .await
        .unwrap();
    user.initialize_margin_user().await.unwrap();

    let margin_user = user.load_margin_user().await.unwrap();
    assert_eq!(margin_user.market, manager.ix_builder.market());

    user
}
