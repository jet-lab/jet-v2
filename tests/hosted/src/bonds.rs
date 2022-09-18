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
    control::state::BondManager,
    orderbook::state::{event_queue_len, orderbook_slab_len, CallbackInfo, OrderParams},
    tickets::state::ClaimTicket,
};

use jet_bonds_lib::builder::{event_builder::build_consume_events_info, BondsIxBuilder};
use jet_margin_sdk::ix_builder::{
    get_control_authority_address, get_metadata_address, ControlIxBuilder, MarginIxBuilder,
};
use jet_proto_math::fixed_point::Fp32;
use jet_simulation::{
    create_test_runtime, create_wallet,
    runtime::TestRuntime,
    send_and_confirm,
    solana_rpc_api::{RpcConnection, SolanaRpcClient},
};
use rand::rngs::OsRng;
use solana_sdk::{
    commitment_config::CommitmentConfig,
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
use tokio::sync::OnceCell;

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

mod keys {
    json_keypairs! {
        payer = "[222,147,115,219,200,207,183,34,103,192,44,23,43,203,127,70,67,170,118,146,40,128,166,176,91,184,240,89,157,92,138,41,12,48,55,127,230,6,125,75,21,171,39,213,6,155,83,215,2,250,164,163,97,165,211,0,204,244,39,28,66,112,134,180]";
        authority = "[39,147,77,63,116,164,246,7,32,209,175,208,128,14,177,244,45,71,65,156,25,123,37,149,13,154,122,109,65,99,210,163,119,197,146,64,183,117,85,212,178,252,172,16,127,0,85,40,51,163,146,80,31,186,233,84,244,109,213,213,255,149,121,207]";
        // test_mint = "[250,147,202,203,141,69,148,144,94,77,227,139,131,238,119,177,155,59,20,90,232,125,84,36,38,159,178,180,109,242,88,156,151,27,163,56,120,190,145,77,103,139,67,48,60,172,93,127,35,86,111,179,36,15,254,100,98,127,5,36,144,37,67,23]";
        // event_queue = "[42,34,186,11,198,208,249,238,14,243,74,72,179,215,135,80,229,102,180,177,101,238,158,154,53,132,165,200,59,29,76,35,194,139,110,207,15,55,88,75,12,9,247,35,74,68,152,56,166,95,89,33,229,86,189,111,82,60,98,107,37,70,81,127]";
        // crank = "[78,122,206,47,0,102,125,42,154,126,250,137,110,198,174,2,137,75,111,54,34,93,221,115,77,222,133,247,129,233,156,0,50,26,219,183,209,148,208,168,131,217,2,159,31,202,77,155,22,129,62,12,119,47,130,91,28,192,91,204,32,21,101,165]";
    }

    macro_rules! json_keypairs {
        ($($name:ident = $json:literal;)+) => {
            $(pub fn $name() -> solana_sdk::signature::Keypair {
                key_strings::get(key_strings::$name)
            })+
            mod key_strings {
                $(#[allow(non_upper_case_globals)] pub const $name: &str = $json;)+
                pub fn get(s: &str) -> solana_sdk::signature::Keypair {
                    solana_sdk::signature::read_keypair(&mut s.as_bytes().clone()).unwrap()
                }
            }
        };
    }
    use json_keypairs;
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
            // .with_crank()
            // .await?
            .with_margin()
            .await
    }

    pub async fn new(client: Arc<dyn SolanaRpcClient>) -> Result<Self> {
        let payer = client.payer();
        let program_authority = keys::authority();

        let test_token_mint = {
            let mint_keypair = Keypair::new();
            let recent_blockhash = client.get_latest_blockhash().await?;
            let rent = client
                .get_minimum_balance_for_rent_exemption(Mint::LEN)
                .await?;
            let transaction =
                initialize_test_mint_transaction(&mint_keypair, payer, 6, rent, recent_blockhash);
            client.send_and_confirm_transaction(&transaction).await?;
            mint_keypair
        };

        let ix_builder =
            BondsIxBuilder::new_from_seed(&test_token_mint.pubkey(), BOND_MANAGER_SEED)
                .with_payer(&payer.pubkey())
                .with_authority(&program_authority.pubkey());

        let mut manager = Self {
            client: client.clone(),
            ix_builder,
            kps: Keys::new(),
            keys: Keys::new(),
        };
        manager.insert_kp(
            "payer",
            Keypair::from_base58_string(&payer.to_base58_string()),
        );
        manager.insert_kp("authority", program_authority);
        manager.insert_kp("token_mint", test_token_mint);

        Ok(manager)
    }

    pub async fn with_bonds(mut self) -> Result<Self> {
        let eq_kp = Keypair::new();
        let bids_kp = Keypair::new();
        let asks_kp = Keypair::new();

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
        self.insert_kp("eq", eq_kp);
        self.insert_kp("bids", bids_kp);
        self.insert_kp("asks", asks_kp);

        self.sign_send_transaction(&ixns, None).await?;

        let init_manager = self.ix_builder.initialize_manager(
            BOND_MANAGER_TAG,
            BOND_MANAGER_SEED,
            STAKE_DURATION,
            self.keys.unwrap("token_mint")?,
            &Pubkey::default(),
            &Pubkey::default(),
        )?;
        let init_orderbook = self
            .ix_builder
            .initialize_orderbook(self.keys.unwrap("authority")?, MIN_ORDER_SIZE)?;
        self.sign_send_transaction(&[init_manager, init_orderbook], None)
            .await?;

        Ok(self)
    }

    pub async fn with_crank(mut self) -> Result<Self> {
        let crank = Keypair::new();

        self.ix_builder = self.ix_builder.with_crank(&crank.pubkey());
        let auth_crank = self.ix_builder.authorize_crank_instruction()?;
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
        let msg = Message::new(instructions, Some(self.keys.unwrap("payer")?));
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
            self.manager.keys.unwrap("payer")?,
            &self.proxy.pubkey(),
            self.manager.keys.unwrap("token_mint")?,
        );
        let create_ticket = create_associated_token_account(
            self.manager.keys.unwrap("payer")?,
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
