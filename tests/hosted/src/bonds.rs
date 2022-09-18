use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use jet_bonds::orderbook::state::{event_queue_len, orderbook_slab_len};
use jet_bonds_lib::builder::BondsIxBuilder;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_sdk::{
    instruction::Instruction,
    message::Message,
    pubkey::Pubkey,
    signature::Signature,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};

pub struct MarketConfig {
    pub token_mint: Pubkey,
    pub seed: [u8; 32],
    pub authority: Pubkey,
    pub event_queue: Keypair,
    pub bids: Keypair,
    pub asks: Keypair,
    pub event_queue_capacity: usize,
    pub orderbook_capacity: usize,
}

#[derive(Clone)]
pub struct BondsTestManager {
    pub rpc: Arc<dyn SolanaRpcClient>,
    pub markets: BondMarketManager,
}

impl BondsTestManager {
    pub async fn new(rpc: Arc<dyn SolanaRpcClient>) -> Self {
        Self {
            rpc: rpc.clone(),
            markets: BondMarketManager::new(rpc),
        }
    }
}

#[derive(Clone)]
pub struct BondMarketManager {
    rpc: Arc<dyn SolanaRpcClient>,
    pub markets: HashMap<Pubkey, BondsIxBuilder>,
}

impl BondMarketManager {
    pub fn new(rpc: Arc<dyn SolanaRpcClient>) -> Self {
        Self {
            rpc,
            markets: Default::default(),
        }
    }
    pub async fn create_test_market(&mut self, config: MarketConfig) -> Result<Pubkey> {
        let ix = BondsIxBuilder::new_from_seed(&config.token_mint, config.seed)
            .with_payer(&self.rpc.payer().pubkey())
            .with_authority(&config.authority)
            .with_orderbook_accounts(
                Some(config.bids.pubkey()),
                Some(config.asks.pubkey()),
                Some(config.event_queue.pubkey()),
            );

        let init_eq = {
            let rent = self
                .rpc
                .get_minimum_balance_for_rent_exemption(event_queue_len(
                    config.event_queue_capacity,
                ))
                .await?;
            ix.initialize_event_queue(
                &config.event_queue.pubkey(),
                config.event_queue_capacity,
                rent,
            )?
        };

        let init_bids = {
            let rent = self
                .rpc
                .get_minimum_balance_for_rent_exemption(orderbook_slab_len(
                    config.orderbook_capacity,
                ))
                .await?;
            ix.initialize_orderbook_slab(&config.bids.pubkey(), config.orderbook_capacity, rent)?
        };
        let init_asks = {
            let rent = self
                .rpc
                .get_minimum_balance_for_rent_exemption(orderbook_slab_len(
                    config.orderbook_capacity,
                ))
                .await?;
            ix.initialize_orderbook_slab(&config.asks.pubkey(), config.orderbook_capacity, rent)?
        };

        self.rpc
            .sign_send(&[init_eq, init_asks, init_bids], &[])
            .await?;

        let key = ix.manager();
        self.markets.insert(key, ix);
        Ok(key)
    }
}

#[async_trait]
trait TransactionSender {
    async fn sign_send(
        &self,
        instructions: &[Instruction],
        signers: &[Keypair],
    ) -> Result<Signature>;
}

#[async_trait]
impl TransactionSender for Arc<dyn SolanaRpcClient> {
    async fn sign_send(
        &self,
        instructions: &[Instruction],
        signers: &[Keypair],
    ) -> Result<Signature> {
        let mut tx =
            Transaction::new_unsigned(Message::new(instructions, Some(&self.payer().pubkey())));
        let recent_blockhash = self.get_latest_blockhash().await?;
        for signer in signers {
            for key in tx.clone().message.signer_keys() {
                if key == &signer.pubkey() {
                    tx.partial_sign(&[signer], recent_blockhash);
                }
            }
        }
        self.send_and_confirm_transaction(&tx).await
    }
}
