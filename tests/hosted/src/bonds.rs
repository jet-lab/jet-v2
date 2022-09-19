use std::{collections::HashMap, sync::Arc};

use agnostic_orderbook::state::{
    critbit::{LeafNode, Slab},
    event_queue::EventQueue,
    market_state::MarketState as OrderBookMarketState,
    orderbook::OrderBookState,
    AccountTag,
};
use anchor_lang::AccountDeserialize;
use anchor_spl::token::TokenAccount;
use anyhow::Result;
use async_trait::async_trait;
use jet_bonds::{
    control::{instructions::InitializeBondManagerParams, state::BondManager},
    orderbook::state::{event_queue_len, orderbook_slab_len, CallbackInfo, OrderParams},
    tickets::state::ClaimTicket,
};

use jet_margin_sdk::{
    bonds::{event_builder::build_consume_events_info, BondsIxBuilder},
    ix_builder::{
        get_control_authority_address, get_metadata_address, ControlIxBuilder, MarginIxBuilder,
    },
};
use jet_proto_math::fixed_point::Fp32;
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

lazy_static::lazy_static! {
    static ref PAYER: Keypair = map_keypair_file(
        shellexpand::env("$PWD/tests/keypairs/test-mint.json")
            .unwrap()
            .to_string()
    )
    .unwrap();
    /// Testing mint keypair
    static ref MINT: Keypair = map_keypair_file(
        shellexpand::env("$PWD/tests/keypairs/test-mint.json")
            .unwrap()
            .to_string()
    )
    .unwrap();

    static ref QUEUE: Keypair = map_keypair_file(shellexpand::env("$PWD/tests/keypairs/event_queue.json")
        .unwrap()
        .to_string()).unwrap();
    static ref BIDS: Keypair = map_keypair_file(shellexpand::env("$PWD/tests/keypairs/bids.json")
        .unwrap()
        .to_string()).unwrap();
    static ref ASKS: Keypair = map_keypair_file(shellexpand::env("$PWD/tests/keypairs/asks.json")
        .unwrap()
        .to_string()).unwrap();

}

fn map_keypair_file(path: String) -> Result<Keypair> {
    solana_clap_utils::keypair::keypair_from_path(&Default::default(), &path, "", false)
        .map_err(|_| anyhow::Error::msg("failed to read keypair"))
}

fn clone_kp(kp: &Keypair) -> Keypair {
    Keypair::from_base58_string(&kp.to_base58_string())
}

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
    client: Arc<dyn SolanaRpcClient>,
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
        TestManager::new(client)
            .await?
            .with_bonds()
            .await?
            .with_crank()
            .await?
            .with_margin()
            .await
    }

    pub async fn new(client: Arc<dyn SolanaRpcClient>) -> Result<Self> {
        let payer = client.payer();

        let test_token_mint = {
            let recent_blockhash = client.get_latest_blockhash().await?;
            let rent = client
                .get_minimum_balance_for_rent_exemption(Mint::LEN)
                .await?;
            let transaction =
                initialize_test_mint_transaction(&MINT, payer, 6, rent, recent_blockhash);
            client.send_and_confirm_transaction(&transaction).await?;
            &MINT
        };

        let ix_builder =
            BondsIxBuilder::new_from_seed(&test_token_mint.pubkey(), BOND_MANAGER_SEED)
                .with_payer(&payer.pubkey());
        let mut manager = Self {
            client: client.clone(),
            ix_builder,
            kps: Keys::new(),
            keys: Keys::new(),
        };
        manager.insert_kp(
            "token_mint",
            Keypair::from_base58_string(&MINT.to_base58_string()),
        );

        Ok(manager)
    }

    pub async fn with_bonds(mut self) -> Result<Self> {
        let eq_kp = &QUEUE;
        let bids_kp = &BIDS;
        let asks_kp = &ASKS;

        let init_eq = {
            let rent = self
                .client
                .get_minimum_balance_for_rent_exemption(event_queue_len(
                    EVENT_QUEUE_CAPACITY as usize,
                ))
                .await?;
            self.ix_builder.initialize_event_queue(
                &eq_kp.pubkey(),
                EVENT_QUEUE_CAPACITY as usize,
                rent,
            )?
        };

        let init_bids = {
            let rent = self
                .client
                .get_minimum_balance_for_rent_exemption(orderbook_slab_len(
                    ORDERBOOK_CAPACITY as usize,
                ))
                .await?;
            self.ix_builder.initialize_orderbook_slab(
                &bids_kp.pubkey(),
                ORDERBOOK_CAPACITY as usize,
                rent,
            )?
        };
        let init_asks = {
            let rent = self
                .client
                .get_minimum_balance_for_rent_exemption(orderbook_slab_len(
                    ORDERBOOK_CAPACITY as usize,
                ))
                .await?;
            self.ix_builder.initialize_orderbook_slab(
                &asks_kp.pubkey(),
                ORDERBOOK_CAPACITY as usize,
                rent,
            )?
        };

        self.ix_builder = self.ix_builder.with_orderbook_accounts(
            Some(bids_kp.pubkey()),
            Some(asks_kp.pubkey()),
            Some(eq_kp.pubkey()),
        );

        let ixns = vec![init_eq, init_bids, init_asks];
        self.insert_kp("eq", clone_kp(eq_kp));
        self.insert_kp("bids", clone_kp(bids_kp));
        self.insert_kp("asks", clone_kp(asks_kp));

        self.sign_send_transaction(&ixns, None).await?;

        let ctl = ControlIxBuilder::new(self.client.payer().pubkey());
        let init_manager = ctl.create_bond_market(
            &MINT.pubkey(),
            InitializeBondManagerParams {
                version_tag: BOND_MANAGER_TAG,
                seed: BOND_MANAGER_SEED,
                duration: STAKE_DURATION,
            },
        );
        let init_orderbook = ctl.initialize_bond_orderbook(
            &self.ix_builder.manager(),
            &QUEUE.pubkey(),
            &BIDS.pubkey(),
            &ASKS.pubkey(),
            jet_bonds::control::instructions::InitializeOrderbookParams {
                min_base_order_size: MIN_ORDER_SIZE,
            },
        );
        self.sign_send_transaction(&[init_manager, init_orderbook], None)
            .await?;

        Ok(self)
    }

    pub async fn with_crank(mut self) -> Result<Self> {
        let crank = Keypair::new();

        self.ix_builder = self.ix_builder.with_crank(&crank.pubkey());
        let auth_crank = ControlIxBuilder::new(self.client.payer().pubkey())
            .register_orderbook_crank(&crank.pubkey());
        self.insert_kp("crank", crank);

        self.sign_send_transaction(&[auth_crank], None).await?;
        Ok(self)
    }

    /// set up metadata authorization for margin to invoke bonds
    pub async fn with_margin(self) -> Result<Self> {
        self.create_authority_if_missing().await?;
        self.register_adapter_if_unregistered(&jet_bonds::ID)
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
        keypairs.push(&self.client.payer());

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
            self.create_authority().await?;
        }

        Ok(())
    }

    pub async fn create_authority(&self) -> Result<()> {
        let ix = ControlIxBuilder::new(self.client.payer().pubkey()).create_authority();

        send_and_confirm(&self.client, &[ix], &[]).await?;
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
pub trait Proxy {
    async fn generate(manager: Arc<TestManager>, owner: &Keypair) -> Result<Self>
    where
        Self: Sized;
    fn pubkey(&self) -> Pubkey;
    fn invoke(&self, ix: Instruction) -> Instruction;
    fn invoke_signed(&self, ix: Instruction) -> Instruction;
}

pub struct NoProxy(Pubkey);
#[async_trait]
impl Proxy for NoProxy {
    fn pubkey(&self) -> Pubkey {
        self.0
    }

    fn invoke(&self, ix: Instruction) -> Instruction {
        ix
    }

    fn invoke_signed(&self, ix: Instruction) -> Instruction {
        ix
    }

    async fn generate(_manager: Arc<TestManager>, owner: &Keypair) -> Result<Self> {
        Ok(NoProxy(owner.pubkey()))
    }
}

#[async_trait]
impl Proxy for MarginIxBuilder {
    fn pubkey(&self) -> Pubkey {
        self.address
    }

    fn invoke(&self, ix: Instruction) -> Instruction {
        self.accounting_invoke(ix)
    }

    fn invoke_signed(&self, ix: Instruction) -> Instruction {
        self.adapter_invoke(ix)
    }

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
}

impl<P: Proxy> BondsUser<P> {
    pub fn new_with_proxy(manager: Arc<TestManager>, owner: Keypair, proxy: P) -> Result<Self> {
        let token_acc =
            get_associated_token_address(&proxy.pubkey(), manager.keys.unwrap("token_mint")?);

        Ok(Self {
            owner,
            proxy,
            token_acc,
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
            self.manager.keys.unwrap("token_mint")?,
        );
        let create_ticket = create_associated_token_account(
            &self.manager.client.payer().pubkey(),
            &self.proxy.pubkey(),
            &self.manager.ix_builder.ticket_mint(),
        );
        let fund = spl_token::instruction::mint_to(
            &spl_token::ID,
            self.manager.keys.unwrap("token_mint")?,
            &self.token_acc,
            self.manager.keys.unwrap("token_mint")?,
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
        self.manager
            .sign_send_transaction(&[self.proxy.invoke_signed(ix)], Some(&[&self.owner]))
            .await
    }

    pub async fn convert_tokens(&self, amount: u64) -> Result<Signature> {
        let ix = self.manager.ix_builder.convert_tokens(
            Some(&self.proxy.pubkey()),
            None,
            None,
            None,
            amount,
        )?;
        self.manager
            .sign_send_transaction(&[self.proxy.invoke_signed(ix)], Some(&[&self.owner]))
            .await
    }

    pub async fn stake_tokens(&self, amount: u64, seed: Vec<u8>) -> Result<Signature> {
        let ix = self
            .manager
            .ix_builder
            .stake_tickets(&self.proxy.pubkey(), None, amount, seed)?;

        self.manager
            .sign_send_transaction(&[self.proxy.invoke_signed(ix)], Some(&[&self.owner]))
            .await
    }

    pub async fn redeem_claim_ticket(&self, seed: Vec<u8>) -> Result<Signature> {
        let ticket = self.claim_ticket_key(seed);
        let ix = self
            .manager
            .ix_builder
            .redeem_ticket(&self.proxy.pubkey(), &ticket, None)?;
        self.manager
            .sign_send_transaction(&[self.proxy.invoke_signed(ix)], Some(&[&self.owner]))
            .await
    }

    pub async fn sell_tickets_order(&self, params: OrderParams) -> Result<Signature> {
        let borrow =
            self.manager
                .ix_builder
                .sell_tickets_order(&self.proxy.pubkey(), None, None, params)?;
        self.manager
            .sign_send_transaction(&[self.proxy.invoke_signed(borrow)], Some(&[&self.owner]))
            .await
    }

    pub async fn margin_borrow_order(&self, params: OrderParams) -> Result<Signature> {
        let borrow = self
            .manager
            .ix_builder
            .margin_borrow_order(self.proxy.pubkey(), params)?;
        self.manager
            .sign_send_transaction(&[self.proxy.invoke_signed(borrow)], Some(&[&self.owner]))
            .await
    }

    pub async fn lend_order(&self, params: OrderParams, seed: Vec<u8>) -> Result<Signature> {
        let lend =
            self.manager
                .ix_builder
                .lend_order(&self.proxy.pubkey(), None, None, params, seed)?;
        self.manager
            .sign_send_transaction(&[self.proxy.invoke_signed(lend)], Some(&[&self.owner]))
            .await
    }
}

impl<P: Proxy> BondsUser<P> {
    pub fn claim_ticket_key(&self, seed: Vec<u8>) -> Pubkey {
        self.manager
            .ix_builder
            .claim_ticket_key(&self.proxy.pubkey(), seed)
    }
    pub async fn load_claim_ticket(&self, seed: Vec<u8>) -> Result<ClaimTicket> {
        let key = self.claim_ticket_key(seed);

        self.manager.load_anchor(&key).await
    }
    /// loads the current state of the user token wallet
    pub async fn tokens(&self) -> Result<u64> {
        let key = get_associated_token_address(
            &self.proxy.pubkey(),
            self.manager.keys.unwrap("token_mint")?,
        );

        self.manager
            .load_anchor::<TokenAccount>(&key)
            .await
            .map(|a| a.amount)
    }

    /// loads the current state of the user token wallet
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
}

pub struct OrderAmount {
    pub base: u64,
    pub quote: u64,
    pub price: u64,
}

impl OrderAmount {
    pub fn from_amount_rate(amount: u64, rate: u64) -> Self {
        let quote = amount;
        let base = quote + ((quote * rate) / 10_000);
        let price = Fp32::from(quote) / base;

        OrderAmount {
            base,
            quote,
            price: price.as_u64().unwrap(),
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
