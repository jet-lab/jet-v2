use std::{sync::Arc, time::Duration};

use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_sdk::pubkey::Pubkey;

use super::FixedTermIxBuilder;
use crate::{solana::transaction::WithSigner, util::no_dupe_queue::AsyncNoDupeQueue};

const BATCH_SIZE: usize = 100;

pub async fn settle_margin_users(
    rpc: Arc<dyn SolanaRpcClient>,
    builder: FixedTermIxBuilder,
    margin_accounts: AsyncNoDupeQueue<Pubkey>,
) {
    tracing::info!(
        "starting settler crank for fixed term market {}",
        builder.market()
    );
    let ix = Arc::new(builder);
    loop {
        let to_settle = margin_accounts.pop_many(BATCH_SIZE).await;
        let has_more = to_settle.len() < BATCH_SIZE;
        for margin_account in to_settle {
            spawn_settle(
                rpc.clone(),
                ix.clone(),
                margin_account,
                Some(margin_accounts.clone()),
            );
        }
        if !has_more {
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

fn spawn_settle(
    rpc: Arc<dyn SolanaRpcClient>,
    builder: Arc<FixedTermIxBuilder>,
    margin_account: Pubkey,
    retry_queue: Option<AsyncNoDupeQueue<Pubkey>>,
) {
    tokio::spawn(async move {
        tracing::debug!("sending settle tx for margin account {margin_account}");
        match builder
            .margin_settle(margin_account)
            .with_signers(&[])
            .send_and_confirm(&rpc)
            .await
        {
            Ok(_) => tracing::debug!("settled margin account {margin_account}"),
            Err(e) => {
                tracing::error!("failed to settle margin account {margin_account} - {e}");
                if let Some(q) = retry_queue {
                    q.push(margin_account).await;
                }
            }
        }
    });
}
