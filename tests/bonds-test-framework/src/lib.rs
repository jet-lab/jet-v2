use std::{collections::HashMap, sync::Arc};

use agnostic_orderbook::state::{
    critbit::{LeafNode, Slab},
    event_queue::EventQueue,
    market_state::MarketState as OrderBookMarketState,
    orderbook::OrderBookState,
    AccountTag,
};
use anchor_client::{anchor_lang::AccountDeserialize, solana_client::rpc_client::RpcClient};
use anchor_spl::token::TokenAccount;
use anyhow::Result;
use jet_bonds::{
    control::state::BondManager,
    orderbook::state::{CallbackInfo, OrderParams, EVENT_QUEUE_LEN, ORDERBOOK_SLAB_LEN},
    tickets::state::ClaimTicket,
};

use jet_bonds_lib::builder::BondsIxBuilder;
use jet_simulation::{
    create_test_runtime,
    runtime::TestRuntime,
    solana_rpc_api::{RpcConnection, SolanaRpcClient},
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    message::Message,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::state::Mint;
use transactions::initialize_test_mint_transaction;

#[cfg(test)]
mod tests;
pub mod transactions;

pub const LOCALNET_URL: &str = "http://127.0.0.1:8899";
pub const DEVNET_URL: &str = "https://api.devnet.solana.com/";

pub const MINT_DECIMALS: u8 = 6;
pub const STARTING_TOKENS: u64 = 1_000_000_000 * ONE_TOKEN;
pub const ONE_TOKEN: u64 = 10u64.pow(MINT_DECIMALS as u32);
pub const BOND_MANAGER_SEED: u64 = u64::from_le_bytes(*b"verygood");
pub const BOND_MANAGER_TAG: u64 = u64::from_le_bytes(*b"zachzach");
pub const FEEDER_FUND_SEED: u64 = u64::from_le_bytes(*b"feedingf");
pub const STAKE_DURATION: i64 = 3; // in seconds
pub const CONVERSION_DECIMALS: i8 = -3;
pub const MIN_ORDER_SIZE: u64 = 1_000;

pub fn emulated_client() -> TestRuntime {
    create_test_runtime![jet_bonds, bonds_metadata]
}

pub fn localhost_client() -> RpcConnection {
    let payer = read_keypair_file(&*shellexpand::tilde("~/.config/solana/id.json")).unwrap();
    let rpc =
        RpcClient::new_with_commitment(LOCALNET_URL.to_string(), CommitmentConfig::confirmed());
    RpcConnection::new(payer, rpc)
}

#[derive(Debug, Default)]
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

impl TestManager {
    pub async fn new(client: Arc<dyn SolanaRpcClient>) -> Result<Self> {
        let payer = client.payer();
        let program_authority = read_keypair_file(
            &*shellexpand::env("$PWD/tests/deps/keypairs/authority-keypair.json").unwrap(),
        )
        .unwrap();

        let test_token_mint = {
            let mint_keypair = read_keypair_file(
                &*shellexpand::env("$PWD/tests/deps/keypairs/test_mint-keypair.json").unwrap(),
            )
            .unwrap();
            let recent_blockhash = client.get_latest_blockhash().await?;
            let rent = client
                .get_minimum_balance_for_rent_exemption(Mint::LEN)
                .await?;
            let transaction = initialize_test_mint_transaction(
                &mint_keypair,
                payer,
                MINT_DECIMALS,
                rent,
                recent_blockhash,
            );
            client.send_and_confirm_transaction(&transaction).await?;
            mint_keypair
        };

        let manager = Pubkey::find_program_address(
            &[
                jet_bonds::seeds::BOND_MANAGER,
                test_token_mint.pubkey().as_ref(),
                BOND_MANAGER_SEED.to_le_bytes().as_ref(),
            ],
            &jet_bonds::ID,
        )
        .0;

        let ix_builder = BondsIxBuilder::new(manager)
            .with_payer(&payer.pubkey())
            .with_mint(&test_token_mint.pubkey())
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
        let eq_kp = read_keypair_file(&*shellexpand::env(
            "$PWD/tests/deps/keypairs/event_queue-keypair.json",
        )?)
        .unwrap();
        let bids_kp = Keypair::new();
        let asks_kp = Keypair::new();

        let init_eq = {
            let rent = self
                .client
                .get_minimum_balance_for_rent_exemption(EVENT_QUEUE_LEN as usize)
                .await?;
            solana_sdk::system_instruction::create_account(
                self.keys.unwrap("payer")?,
                &eq_kp.pubkey(),
                rent,
                EVENT_QUEUE_LEN as u64,
                &jet_bonds::ID,
            )
        };
        let init_bids = {
            let rent = self
                .client
                .get_minimum_balance_for_rent_exemption(ORDERBOOK_SLAB_LEN as usize)
                .await?;
            solana_sdk::system_instruction::create_account(
                self.keys.unwrap("payer")?,
                &bids_kp.pubkey(),
                rent,
                ORDERBOOK_SLAB_LEN as u64,
                &jet_bonds::ID,
            )
        };
        let init_asks = {
            let rent = self
                .client
                .get_minimum_balance_for_rent_exemption(ORDERBOOK_SLAB_LEN as usize)
                .await?;
            solana_sdk::system_instruction::create_account(
                self.keys.unwrap("payer")?,
                &asks_kp.pubkey(),
                rent,
                ORDERBOOK_SLAB_LEN as u64,
                &jet_bonds::ID,
            )
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
            CONVERSION_DECIMALS,
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
        let crank = read_keypair_file(&*shellexpand::env(
            "$PWD/tests/deps/keypairs/crank-keypair.json",
        )?)
        .unwrap();

        self.ix_builder = self.ix_builder.with_crank(&crank.pubkey());
        let auth_crank = self.ix_builder.authorize_crank_instruction()?;
        self.insert_kp("crank", crank);

        self.sign_send_transaction(&[auth_crank], None).await?;
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
        let (event_accounts, num_events, seeds) =
            jet_bonds_orderbook_crank::populate_event_accounts(
                eq.inner(),
                &mut rand::rngs::OsRng::default(),
            );
        let remaining_accounts = event_accounts.iter().collect::<Vec<&Pubkey>>();
        let consume =
            self.ix_builder
                .consume_events(remaining_accounts, num_events as u32, seeds)?;

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

pub struct BondsUser {
    kp: Keypair,
    token_acc: Pubkey,
    manager: Arc<TestManager>,
}

impl BondsUser {
    pub fn new(manager: Arc<TestManager>) -> Result<Self> {
        let kp = Keypair::new();
        let token_acc =
            get_associated_token_address(&kp.pubkey(), manager.keys.unwrap("token_mint")?);

        Ok(Self {
            kp,
            token_acc,
            manager,
        })
    }

    pub async fn new_funded(manager: Arc<TestManager>) -> Result<Self> {
        let user = Self::new(manager)?;

        let create_token = create_associated_token_account(
            user.manager.keys.unwrap("payer")?,
            &user.kp.pubkey(),
            user.manager.keys.unwrap("token_mint")?,
        );
        let create_ticket = create_associated_token_account(
            user.manager.keys.unwrap("payer")?,
            &user.kp.pubkey(),
            &user.manager.ix_builder.ticket_mint(),
        );
        let fund = spl_token::instruction::mint_to(
            &spl_token::ID,
            user.manager.keys.unwrap("token_mint")?,
            &user.token_acc,
            user.manager.keys.unwrap("token_mint")?,
            &[],
            STARTING_TOKENS,
        )?;

        user.manager
            .sign_send_transaction(&[create_token, create_ticket, fund], Some(&[&user.kp]))
            .await?;
        Ok(user)
    }
}

impl BondsUser {
    pub async fn convert_tokens(&self, amount: u64) -> Result<Signature> {
        let ix = self.manager.ix_builder.convert_tokens(
            Some(&self.kp.pubkey()),
            None,
            None,
            None,
            amount,
        )?;
        self.manager
            .sign_send_transaction(&[ix], Some(&[&self.kp]))
            .await
    }

    pub async fn stake_tokens(&self, amount: u64, seed: u64) -> Result<Signature> {
        let ix = self
            .manager
            .ix_builder
            .stake_tickets(&self.kp.pubkey(), None, amount, seed)?;

        self.manager
            .sign_send_transaction(&[ix], Some(&[&self.kp]))
            .await
    }

    pub async fn redeem_claim_ticket(&self, seed: u64) -> Result<Signature> {
        let ticket = self.claim_ticket_key(seed);
        let ix = self
            .manager
            .ix_builder
            .redeem_ticket(&self.kp.pubkey(), &ticket, None)?;
        self.manager
            .sign_send_transaction(&[ix], Some(&[&self.kp]))
            .await
    }

    pub async fn borrow_order(&self, params: OrderParams) -> Result<Signature> {
        let borrow = self
            .manager
            .ix_builder
            .borrow_order(&self.kp.pubkey(), None, None, params)?;
        self.manager
            .sign_send_transaction(&[borrow], Some(&[&self.kp]))
            .await
    }

    pub async fn lend_order(&self, params: OrderParams, seed: u64) -> Result<Signature> {
        let lend =
            self.manager
                .ix_builder
                .lend_order(&self.kp.pubkey(), None, None, params, seed)?;
        self.manager
            .sign_send_transaction(&[lend], Some(&[&self.kp]))
            .await
    }
}

impl BondsUser {
    pub fn claim_ticket_key(&self, seed: u64) -> Pubkey {
        self.manager
            .ix_builder
            .claim_ticket_key(&self.kp.pubkey(), seed)
    }
    pub async fn load_claim_ticket(&self, seed: u64) -> Result<ClaimTicket> {
        let key = self.claim_ticket_key(seed);

        self.manager.load_anchor(&key).await
    }
    /// loads the current state of the user token wallet
    pub async fn tokens(&self) -> Result<u64> {
        let key = get_associated_token_address(
            &self.kp.pubkey(),
            self.manager.keys.unwrap("token_mint")?,
        );

        self.manager
            .load_anchor::<TokenAccount>(&key)
            .await
            .map(|a| a.amount)
    }

    /// loads the current state of the user token wallet
    pub async fn tickets(&self) -> Result<u64> {
        let key =
            get_associated_token_address(&self.kp.pubkey(), &self.manager.ix_builder.ticket_mint());

        self.manager
            .load_anchor::<TokenAccount>(&key)
            .await
            .map(|a| a.amount)
    }
}
