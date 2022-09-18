use std::{collections::HashMap, sync::Arc};

use agnostic_orderbook::state::{
    critbit::{LeafNode, Slab},
    event_queue::EventQueue,
    market_state::MarketState as OrderBookMarketState,
    orderbook::OrderBookState,
    AccountTag,
};
use anchor_lang::AccountDeserialize;
use anchor_spl::token::{Mint, TokenAccount};
use anyhow::Result;
use jet_bonds::{
    control::state::BondManager,
    orderbook::state::{event_queue_len, orderbook_slab_len, CallbackInfo},
};
use jet_bonds_sdk::builder::{event_builder::build_consume_events_info, BondsIxBuilder, UnwrapKey};
use jet_margin_sdk::ix_builder::{
    get_control_authority_address, get_metadata_address, ControlIxBuilder,
};
use jet_rpc::{
    create_test_wallet,
    solana_rpc_api::{AsyncSigner, SolanaConnection, SolanaRpc},
    transaction::sign_send_instructions,
};
use solana_sdk::{
    hash::Hash,
    instruction::Instruction,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    system_instruction,
    transaction::Transaction,
};
use spl_token::instruction::initialize_mint;

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

pub struct BondsTestManager {
    ctx: Arc<dyn SolanaConnection>,
    pub ix_builder: BondsIxBuilder,
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

impl Clone for BondsTestManager {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl BondsTestManager {
    pub async fn full(client: Arc<dyn SolanaConnection>) -> Result<Self> {
        Self::new(client).await?.with_bonds().await
        // .with_crank()
        // .await?
        // .with_margin()
        // .await
    }

    pub async fn new(ctx: Arc<dyn SolanaConnection>) -> Result<Self> {
        let ix = BondsIxBuilder::new(Pubkey::default());

        Ok(Self {
            ctx,
            ix_builder: ix,
        })
    }

    pub async fn with_bonds(mut self) -> Result<Self> {
        todo!("run init through ctl")
    }

    pub async fn with_crank(mut self) -> Result<Self> {
        todo!()
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
        add_signers: &[AsyncSigner],
    ) -> Result<Signature> {
        sign_send_instructions(self.ctx.clone(), instructions, add_signers).await
    }
    pub async fn consume_events(&self) -> Result<Signature> {
        let mut eq = self.load_event_queue().await?;

        let info = build_consume_events_info(eq.inner())?;
        let (accounts, num_events, seeds) = info.as_params();
        let consume = self
            .ix_builder
            .consume_events(accounts, num_events, seeds)?;

        self.sign_send_transaction(&[consume], &[]).await
    }
    pub async fn pause_ticket_redemption(&self) -> Result<Signature> {
        let pause = self.ix_builder.pause_ticket_redemption()?;

        self.sign_send_transaction(&[pause], &[]).await
    }
    pub async fn resume_ticket_redemption(&self) -> Result<Signature> {
        let resume = self.ix_builder.resume_ticket_redemption()?;

        self.sign_send_transaction(&[resume], &[]).await
    }

    pub async fn pause_orders(&self) -> Result<Signature> {
        let pause = self.ix_builder.pause_order_matching()?;

        self.sign_send_transaction(&[pause], &[]).await
    }

    pub async fn resume_orders(&self) -> Result<()> {
        loop {
            if self.load_orderbook_market_state().await?.pause_matching == (false as u8) {
                break;
            }

            let resume = self.ix_builder.resume_order_matching()?;
            self.sign_send_transaction(&[resume], &[]).await?;
        }

        Ok(())
    }
    pub async fn create_authority_if_missing(&self) -> Result<()> {
        if self
            .ctx
            .get_account(&get_control_authority_address())
            .await?
            .is_none()
        {
            self.create_authority().await?;
        }

        Ok(())
    }

    pub async fn create_authority(&self) -> Result<()> {
        let ix = ControlIxBuilder::new(self.ctx.payer().pubkey()).create_authority();

        sign_send_instructions(self.ctx.clone(), &[ix], &[]).await?;
        Ok(())
    }

    pub async fn register_adapter_if_unregistered(&self, adapter: &Pubkey) -> Result<()> {
        if self
            .ctx
            .get_account(&get_metadata_address(adapter))
            .await?
            .is_none()
        {
            self.register_adapter(adapter).await?;
        }

        Ok(())
    }

    pub async fn register_adapter(&self, adapter: &Pubkey) -> Result<()> {
        let ix = ControlIxBuilder::new(self.ctx.payer().pubkey()).register_adapter(adapter);

        sign_send_instructions(self.ctx.clone(), &[ix], &[]).await?;
        Ok(())
    }

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
        self.load_data(&self.ix_builder.unwrap_key(k)?).await
    }
    pub async fn load_data(&self, key: &Pubkey) -> Result<Vec<u8>> {
        Ok(self
            .ctx
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
