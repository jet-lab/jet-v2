use std::{sync::Arc, time::Duration};

use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_sdk::pubkey::Pubkey;

use super::FixedTermIxBuilder;
use crate::{solana::transaction::WithSigner, util::no_dupe_queue::AsyncNoDupeQueue};

pub async fn settle_margin_users_loop(
    rpc: Arc<dyn SolanaRpcClient>,
    builder: FixedTermIxBuilder,
    margin_accounts: AsyncNoDupeQueue<Pubkey>,
    config: SettleMarginUsersConfig,
) {
    tracing::info!(
        "starting settler crank for fixed term market {}",
        builder.market()
    );
    let ix = Arc::new(builder);
    loop {
        let to_settle = margin_accounts.pop_many(config.batch_size).await;
        let has_more = to_settle.len() < config.batch_size;
        for margin_account in to_settle {
            tokio::spawn(settle_with_recovery(
                rpc.clone(),
                ix.clone(),
                margin_account,
                Some(margin_accounts.clone()),
            ));
        }
        if has_more {
            tokio::time::sleep(config.batch_delay).await;
        } else if config.exit_when_done {
            break;
        } else {
            tokio::time::sleep(config.wait_for_more_delay).await;
        }
    }
}

/// Attempts to settle a margin user. Does not return an error on failure.
/// Instead, it logs an error and then pushes the margin account back to the end
/// of the queue.
async fn settle_with_recovery(
    rpc: Arc<dyn SolanaRpcClient>,
    builder: Arc<FixedTermIxBuilder>,
    margin_account: Pubkey,
    retry_queue: Option<AsyncNoDupeQueue<Pubkey>>,
) {
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
}

/// Performance settings for the settlement loop. Use the default to process
/// settlements at a relaxed pace.
pub struct SettleMarginUsersConfig {
    /// Number of margin users to process simultaneously. All settle
    /// instructions will be sent at once.
    pub batch_size: usize,

    /// Time to wait between batches when the previous batch maxed out
    /// the batch_size and there are still more accounts to settle.
    pub batch_delay: Duration,

    /// Time to wait between batches when the previous batch cleared
    /// the entire queue and there are no more accounts to settle at this time.
    pub wait_for_more_delay: Duration,

    /// The loop will exit when there are no more events in the queue. Usually
    /// you probably want this to be false. Some scenarios where you may want
    /// this to be true:
    /// - unit tests
    /// - ad hoc or emergency executions that are intended to allocate some
    ///   extra resources when the standard settler isn't moving fast enough.
    pub exit_when_done: bool,
}

impl Default for SettleMarginUsersConfig {
    /// Attempts to set a conservative default that can safely coexist with an
    /// event consumer that deserves higher CPU priority since its tasks are
    /// more important.
    fn default() -> Self {
        Self {
            batch_size: 10,
            batch_delay: Duration::from_secs(1),
            wait_for_more_delay: Duration::from_secs(5),
            exit_when_done: false,
        }
    }
}
