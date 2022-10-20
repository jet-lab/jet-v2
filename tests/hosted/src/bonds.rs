use std::{collections::HashMap, sync::Arc};

use agnostic_orderbook::state::{
    critbit::{LeafNode, Slab},
    event_queue::EventQueue,
    market_state::MarketState as OrderBookMarketState,
    orderbook::OrderBookState,
    AccountTag,
};
use anchor_lang::Discriminator;
use anchor_lang::{AccountDeserialize, AnchorSerialize, InstructionData, ToAccountMetas};
use anchor_spl::token::TokenAccount;
use anyhow::Result;
use async_trait::async_trait;
use jet_bonds::{
    control::state::BondManager,
    margin::state::MarginUser,
    orderbook::state::{event_queue_len, orderbook_slab_len, CallbackInfo, OrderParams},
    tickets::state::ClaimTicket,
};

use jet_margin_sdk::{
    bonds::{bonds_pda, event_builder::build_consume_events_info, BondsIxBuilder},
    ix_builder::{
        get_control_authority_address, get_metadata_address, ControlIxBuilder, MarginIxBuilder,
    },
    margin_integrator::{NoProxy, Proxy},
    solana::{
        keypair::clone,
        transaction::{SendTransactionBuilder, TransactionBuilder},
    },
    tx_builder::global_initialize_instructions,
};
use jet_metadata::{PositionTokenMetadata, TokenKind};
use jet_proto_math::fixed_point::Fp32;
use jet_simulation::{
    create_wallet, generate_keypair, send_and_confirm, solana_rpc_api::SolanaRpcClient,
};
use solana_sdk::{
    hash::Hash,
    instruction::Instruction,
    message::Message,
    native_token::LAMPORTS_PER_SOL,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    system_instruction, system_program,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::{instruction::initialize_mint, state::Mint};

use crate::tokens::TokenManager;

pub const LOCALNET_URL: &str = "http://127.0.0.1:8899";
pub const DEVNET_URL: &str = "https://api.devnet.solana.com/";

pub const STARTING_TOKENS: u64 = 1_000_000_000;
pub const BOND_MANAGER_SEED: [u8; 32] = *b"verygoodlongseedfrombytewemakeit";
pub const BOND_MANAGER_TAG: u64 = u64::from_le_bytes(*b"zachzach");
pub const FEEDER_FUND_SEED: u64 = u64::from_le_bytes(*b"feedingf");
pub const ORDERBOOK_CAPACITY: usize = 1_000;
pub const EVENT_QUEUE_CAPACITY: usize = 1_000;
pub const STAKE_DURATION: i64 = 3; // in seconds
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
    pub ix_builder: BondsIxBuilder,
    pub kps: Keys<Keypair>,
    pub keys: Keys<Pubkey>,
}

impl Clone for TestManager {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            ix_builder: self.ix_builder.clone(),
            kps: Keys(
                self.kps
                    .0
                    .iter()
                    .map(|(k, v)| (k.clone(), Keypair::from_bytes(&v.to_bytes()).unwrap()))
                    .collect(),
            ),
            keys: self.keys.clone(),
        }
    }
}

impl TestManager {
    pub async fn full(client: Arc<dyn SolanaRpcClient>) -> Result<Self> {
        let mint = generate_keypair();
        let oracle = TokenManager::new(client.clone())
            .create_oracle(&mint.pubkey())
            .await?;
        let bond_ticket_mint = bonds_pda(&[
            jet_bonds::seeds::BOND_TICKET_MINT,
            BondsIxBuilder::bond_manager_key(
                &Pubkey::default(), //todo airspace
                &mint.pubkey(),
                BOND_MANAGER_SEED,
            )
            .as_ref(),
        ]);
        let ticket_oracle = TokenManager::new(client.clone())
            .create_oracle(&bond_ticket_mint)
            .await?;
        TestManager::new(
            client,
            &mint,
            &generate_keypair(),
            &generate_keypair(),
            &generate_keypair(),
            oracle.price,
            ticket_oracle.price,
        )
        .await?
        .with_crank()
        .await?
        .with_margin()
        .await
    }

    pub async fn new(
        client: Arc<dyn SolanaRpcClient>,
        mint: &Keypair,
        eq_kp: &Keypair,
        bids_kp: &Keypair,
        asks_kp: &Keypair,
        underlying_oracle: Pubkey,
        ticket_oracle: Pubkey,
    ) -> Result<Self> {
        let payer = client.payer();
        let recent_blockhash = client.get_latest_blockhash().await?;
        let rent = client
            .get_minimum_balance_for_rent_exemption(Mint::LEN)
            .await?;
        let transaction = initialize_test_mint_transaction(mint, payer, 6, rent, recent_blockhash);
        client.send_and_confirm_transaction(&transaction).await?;

        let ix_builder = BondsIxBuilder::new_from_seed(
            &Pubkey::default(),
            &mint.pubkey(),
            BOND_MANAGER_SEED,
            payer.pubkey(),
            underlying_oracle,
            ticket_oracle,
        )
        .with_payer(&payer.pubkey());
        let mut this = Self {
            client: client.clone(),
            ix_builder,
            kps: Keys::new(),
            keys: Keys::new(),
        };
        this.insert_kp("token_mint", clone(mint));

        let init_eq = {
            let rent = this
                .client
                .get_minimum_balance_for_rent_exemption(event_queue_len(
                    EVENT_QUEUE_CAPACITY as usize,
                ))
                .await?;
            this.ix_builder.initialize_event_queue(
                &eq_kp.pubkey(),
                EVENT_QUEUE_CAPACITY as usize,
                rent,
            )?
        };

        let init_bids = {
            let rent = this
                .client
                .get_minimum_balance_for_rent_exemption(orderbook_slab_len(
                    ORDERBOOK_CAPACITY as usize,
                ))
                .await?;
            this.ix_builder.initialize_orderbook_slab(
                &bids_kp.pubkey(),
                ORDERBOOK_CAPACITY as usize,
                rent,
            )?
        };
        let init_asks = {
            let rent = this
                .client
                .get_minimum_balance_for_rent_exemption(orderbook_slab_len(
                    ORDERBOOK_CAPACITY as usize,
                ))
                .await?;
            this.ix_builder.initialize_orderbook_slab(
                &asks_kp.pubkey(),
                ORDERBOOK_CAPACITY as usize,
                rent,
            )?
        };
        this.ix_builder = this.ix_builder.with_orderbook_accounts(
            bids_kp.pubkey(),
            asks_kp.pubkey(),
            eq_kp.pubkey(),
        );
        this.insert_kp("eq", clone(eq_kp));
        this.insert_kp("bids", clone(bids_kp));
        this.insert_kp("asks", clone(asks_kp));

        let init_manager = this.ix_builder.initialize_manager(
            this.client.payer().pubkey(),
            BOND_MANAGER_TAG,
            BOND_MANAGER_SEED,
            STAKE_DURATION,
        )?;
        let init_orderbook = this.ix_builder.initialize_orderbook(
            this.client.payer().pubkey(),
            eq_kp.pubkey(),
            bids_kp.pubkey(),
            asks_kp.pubkey(),
            MIN_ORDER_SIZE,
        )?;

        this.sign_send_transaction(
            &[init_eq, init_bids, init_asks, init_manager, init_orderbook],
            None,
        )
        .await?;

        Ok(this)
    }

    pub async fn with_crank(mut self) -> Result<Self> {
        let crank = generate_keypair();

        self.ix_builder = self.ix_builder.with_crank(&crank.pubkey());
        let auth_crank = self
            .ix_builder
            .authorize_crank(self.client.payer().pubkey(), crank.pubkey())?;
        self.insert_kp("crank", crank);

        self.sign_send_transaction(&[auth_crank], None).await?;
        Ok(self)
    }

    /// set up metadata authorization for margin to invoke bonds
    pub async fn with_margin(self) -> Result<Self> {
        self.create_authority_if_missing().await?;
        self.register_adapter_if_unregistered(&jet_bonds::ID)
            .await?;
        self.register_bonds_position_metadatata().await?;

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
}

impl TestManager {
    pub async fn consume_events(&self) -> Result<Signature> {
        let mut eq = self.load_event_queue().await?;

        let info = build_consume_events_info(eq.inner())?;
        let (accounts, num_events, seeds) = info.as_params();
        let consume = self
            .ix_builder
            .consume_events(accounts, num_events, seeds)?;

        self.sign_send_transaction(&[consume], None).await
    }
    pub async fn pause_ticket_redemption(&self) -> Result<Signature> {
        let pause = self.ix_builder.pause_ticket_redemption()?;

        self.sign_send_transaction(&[pause], None).await
    }
    pub async fn resume_ticket_redemption(&self) -> Result<Signature> {
        let resume = self.ix_builder.resume_ticket_redemption()?;

        self.sign_send_transaction(&[resume], None).await
    }

    pub async fn pause_orders(&self) -> Result<Signature> {
        let pause = self.ix_builder.pause_order_matching()?;

        self.sign_send_transaction(&[pause], None).await
    }

    pub async fn resume_orders(&self) -> Result<()> {
        loop {
            if self.load_orderbook_market_state().await?.pause_matching == (false as u8) {
                break;
            }

            let resume = self.ix_builder.resume_order_matching()?;
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
            .send_and_confirm(global_initialize_instructions(payer))
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

    pub async fn register_bonds_position_metadatata(&self) -> Result<()> {
        let manager = self.load_manager().await?;
        self.register_bonds_position_metadatata_impl(
            manager.claims_mint,
            manager.underlying_token_mint,
            TokenKind::Claim,
        )
        .await?;
        self.register_bonds_position_metadatata_impl(
            manager.collateral_mint,
            manager.bond_ticket_mint,
            TokenKind::AdapterCollateral,
        )
        .await?;

        Ok(())
    }

    pub async fn register_bonds_position_metadatata_impl(
        &self,
        position_token_mint: Pubkey,
        underlying_token_mint: Pubkey,
        token_kind: TokenKind,
    ) -> Result<()> {
        let pos_data = PositionTokenMetadata {
            position_token_mint,
            underlying_token_mint,
            adapter_program: jet_bonds::ID,
            token_kind,
            value_modifier: 1_000,
            max_staleness: 1_000,
        };
        let address = get_metadata_address(&position_token_mint);

        let create = Instruction {
            program_id: jet_metadata::ID,
            accounts: jet_metadata::accounts::CreateEntry {
                key_account: position_token_mint,
                metadata_account: address,
                authority: get_control_authority_address(),
                payer: self.client.payer().pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: jet_metadata::instruction::CreateEntry {
                seed: String::new(),
                space: 8 + std::mem::size_of::<PositionTokenMetadata>() as u64,
            }
            .data(),
        };
        let mut metadata = PositionTokenMetadata::discriminator().to_vec();
        pos_data.serialize(&mut metadata)?;

        let set = Instruction {
            program_id: jet_metadata::ID,
            accounts: jet_metadata::accounts::SetEntry {
                metadata_account: address,
                authority: get_control_authority_address(),
            }
            .to_account_metas(None),
            data: jet_metadata::instruction::SetEntry {
                offset: 0,
                data: metadata,
            }
            .data(),
        };

        self.sign_send_transaction(&[create, set], None).await?;

        Ok(())
    }

    pub async fn register_adapter(&self, adapter: &Pubkey) -> Result<()> {
        let ix = ControlIxBuilder::new(self.client.payer().pubkey()).register_adapter(adapter);

        send_and_confirm(&self.client, &[ix], &[]).await?;
        Ok(())
    }
}

pub struct OwnedEQ(Vec<u8>);

impl OwnedEQ {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn inner(&mut self) -> EventQueue<CallbackInfo> {
        EventQueue::from_buffer(
            &mut self.0,
            agnostic_orderbook::state::AccountTag::EventQueue,
        )
        .unwrap()
    }
}

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
}

impl TestManager {
    pub async fn load_manager(&self) -> Result<BondManager> {
        self.load_anchor(&self.ix_builder.manager()).await
    }
    pub async fn load_manager_token_vault(&self) -> Result<TokenAccount> {
        let vault = self.ix_builder.vault();

        self.load_anchor(&vault).await
    }
    pub async fn load_event_queue(&self) -> Result<OwnedEQ> {
        let data = self.load_account("eq").await?;

        Ok(OwnedEQ::new(data))
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
        let margin = MarginIxBuilder::new(owner.pubkey(), 0);
        manager
            .sign_send_transaction(&[margin.create_account()], Some(&[owner]))
            .await?;

        Ok(margin)
    }
}

pub struct BondsUser<P: Proxy> {
    pub owner: Keypair,
    pub proxy: P,
    pub token_acc: Pubkey,
    manager: Arc<TestManager>,
    client: Arc<dyn SolanaRpcClient>,
}

impl<P: Proxy> BondsUser<P> {
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

impl<P: Proxy + GenerateProxy> BondsUser<P> {
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

impl<P: Proxy> BondsUser<P> {
    pub async fn fund(&self) -> Result<()> {
        let create_token = create_associated_token_account(
            &self.manager.client.payer().pubkey(),
            &self.proxy.pubkey(),
            &self.manager.ix_builder.token_mint(),
        );
        let create_ticket = create_associated_token_account(
            &self.manager.client.payer().pubkey(),
            &self.proxy.pubkey(),
            &self.manager.ix_builder.ticket_mint(),
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
            .initialize_margin_user(self.proxy.pubkey())?;
        self.client
            .send_and_confirm_1tx(&[self.proxy.invoke_signed(ix)], &[&self.owner])
            .await
    }

    pub async fn convert_tokens(&self, amount: u64) -> Result<Signature> {
        let ix = self
            .manager
            .ix_builder
            .convert_tokens(self.proxy.pubkey(), None, None, amount)?;
        self.client
            .send_and_confirm_1tx(&[self.proxy.invoke_signed(ix)], &[&self.owner])
            .await
    }

    pub async fn stake_tokens(&self, amount: u64, seed: &[u8]) -> Result<Signature> {
        let ix = self
            .manager
            .ix_builder
            .stake_tickets(self.proxy.pubkey(), None, amount, seed)?;

        self.client
            .send_and_confirm_1tx(&[self.proxy.invoke_signed(ix)], &[&self.owner])
            .await
    }

    pub async fn redeem_claim_ticket(&self, seed: &[u8]) -> Result<Signature> {
        let ticket = self.claim_ticket_key(seed);
        let ix = self
            .manager
            .ix_builder
            .redeem_ticket(self.proxy.pubkey(), ticket, None)?;
        self.client
            .send_and_confirm_1tx(&[self.proxy.invoke_signed(ix)], &[&self.owner])
            .await
    }

    pub async fn sell_tickets_order(&self, params: OrderParams) -> Result<Signature> {
        let borrow =
            self.manager
                .ix_builder
                .sell_tickets_order(self.proxy.pubkey(), None, None, params)?;
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
        )?;
        self.proxy
            .refresh_and_invoke_signed(ix, clone(&self.owner))
            .await
    }

    pub async fn margin_borrow_order(
        &self,
        params: OrderParams,
    ) -> Result<Vec<TransactionBuilder>> {
        let borrow = self
            .manager
            .ix_builder
            .margin_borrow_order(self.proxy.pubkey(), params)?;
        self.proxy
            .refresh_and_invoke_signed(borrow, clone(&self.owner))
            .await
    }

    pub async fn margin_lend_order(
        &self,
        params: OrderParams,
        seed: &[u8],
    ) -> Result<Vec<TransactionBuilder>> {
        let ix =
            self.manager
                .ix_builder
                .margin_lend_order(self.proxy.pubkey(), None, params, seed)?;
        self.proxy
            .refresh_and_invoke_signed(ix, clone(&self.owner))
            .await
    }

    pub async fn lend_order(&self, params: OrderParams, seed: &[u8]) -> Result<Signature> {
        let lend =
            self.manager
                .ix_builder
                .lend_order(self.proxy.pubkey(), None, None, params, seed)?;
        self.client
            .send_and_confirm_1tx(&[self.proxy.invoke_signed(lend)], &[&self.owner])
            .await
    }

    pub async fn cancel_order(&self, order_id: u128) -> Result<Signature> {
        let cancel = self
            .manager
            .ix_builder
            .cancel_order(self.proxy.pubkey(), order_id)?;
        self.client
            .send_and_confirm_1tx(&[self.proxy.invoke_signed(cancel)], &[&self.owner])
            .await
    }

    pub async fn settle(&self) -> Result<Signature> {
        let cancel = self
            .manager
            .ix_builder
            .settle(self.proxy.pubkey(), None, None)?;
        self.client
            .send_and_confirm_1tx(&[self.proxy.invoke_signed(cancel)], &[&self.owner])
            .await
    }
}

impl<P: Proxy> BondsUser<P> {
    pub fn claim_ticket_key(&self, seed: &[u8]) -> Pubkey {
        self.manager
            .ix_builder
            .claim_ticket_key(&self.proxy.pubkey(), seed)
    }
    pub async fn load_claim_ticket(&self, seed: &[u8]) -> Result<ClaimTicket> {
        let key = self.claim_ticket_key(seed);

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
            .collateral;
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
    pub fn from_amount_rate(amount: u64, rate_bps: u64) -> Self {
        let quote = amount;
        let base = quote + ((quote * rate_bps) / 10_000);
        let price = Fp32::from(quote) / base;

        OrderAmount {
            base,
            quote,
            price: price.downcast_u64().unwrap(),
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
