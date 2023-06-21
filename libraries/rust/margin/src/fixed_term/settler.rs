use std::sync::Arc;

use async_trait::async_trait;
use jet_instructions::margin::accounting_invoke;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use jet_solana_client::signature::StandardSigner;
use solana_sdk::pubkey::Pubkey;

use super::FixedTermIxBuilder;
use crate::{
    solana::transaction::{InverseSendTransactionBuilder, WithSigner},
    util::{
        no_dupe_queue::AsyncNoDupeQueue,
        queue_processor::{ChunkProcessor, QueueProcessorConfig, StaticQueueProcessor},
    },
};

pub const SETTLES_PER_TX: usize = 3;

pub type Settler = StaticQueueProcessor<Pubkey, ChunkSettler>;
pub fn settler(
    rpc: Arc<dyn SolanaRpcClient>,
    builder: impl Into<FixedTermIxBuilder>,
    margin_accounts: AsyncNoDupeQueue<Pubkey>,
    config: QueueProcessorConfig,
) -> anyhow::Result<StaticQueueProcessor<Pubkey, ChunkSettler>> {
    StaticQueueProcessor::new(
        margin_accounts,
        config,
        ChunkSettler {
            rpc,
            builder: Arc::new(builder.into()),
        },
    )
}

/// Attempt to settle the provided margin accounts, return error on failure.
async fn try_settle(
    rpc: Arc<dyn SolanaRpcClient>,
    builder: Arc<FixedTermIxBuilder>,
    margin_accounts: &[Pubkey],
) -> anyhow::Result<()> {
    tracing::debug!("sending settle tx for margin accounts {margin_accounts:?}");
    margin_accounts
        .iter()
        .map(|margin_account| {
            vec![accounting_invoke(
                builder.airspace(),
                *margin_account,
                builder.settle(*margin_account),
            )]
            .with_signers(Vec::<StandardSigner>::new())
        })
        .collect::<Vec<_>>()
        .send_and_confirm_condensed(&rpc)
        .await?;
    tracing::debug!("settled margin accounts {margin_accounts:?}");
    Ok(())
}

#[derive(Clone)]
pub struct ChunkSettler {
    rpc: Arc<dyn SolanaRpcClient>,
    builder: Arc<FixedTermIxBuilder>,
}
#[async_trait]
impl ChunkProcessor<Pubkey> for ChunkSettler {
    async fn process(&self, chunk: Vec<Pubkey>) -> anyhow::Result<()> {
        try_settle(self.rpc.clone(), self.builder.clone(), &chunk).await
    }
}
