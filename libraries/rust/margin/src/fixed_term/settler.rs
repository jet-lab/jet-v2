use std::{sync::Arc, time::Duration};

use futures::{
    future::{join_all, try_join_all},
    join, Future,
};
use jet_instructions::margin::accounting_invoke;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use jet_solana_client::clone_to_async;
use solana_sdk::pubkey::Pubkey;

use super::FixedTermIxBuilder;
use crate::{
    solana::transaction::{InverseSendTransactionBuilder, WithSigner},
    util::no_dupe_queue::AsyncNoDupeQueue,
};

pub const SETTLES_PER_TX: usize = 3;

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
            .with_signers(&[])
        })
        .collect::<Vec<_>>()
        .send_and_confirm_condensed(&rpc)
        .await?;
    tracing::debug!("settled margin accounts {margin_accounts:?}");
    Ok(())
}

/// Performance settings for the Settler. Use the default to process
/// settlements at a relaxed pace.
#[derive(Clone, Copy, Debug)]
pub struct SettleMarginUsersConfig {
    /// Number of margin users to process simultaneously. All settle
    /// instructions will be sent at once.
    pub batch_size: usize,

    /// Time to wait between batches when the previous batch maxed out the
    /// batch_size and there are still more accounts to settle.
    pub batch_delay: Duration,

    /// Time to wait between batches when the previous batch cleared the entire
    /// queue and there are no more accounts to settle at this time.
    pub wait_for_more_delay: Duration,
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
        }
    }
}

/// Tracks margin accounts in a queue and exposes functions to settle the
/// accounts in that queue.
#[derive(Clone)]
pub struct Settler {
    rpc: Arc<dyn SolanaRpcClient>,
    builder: Arc<FixedTermIxBuilder>,
    margin_accounts: AsyncNoDupeQueue<Pubkey>,
    retry_queue: AsyncNoDupeQueue<Pubkey>,
    config: SettleMarginUsersConfig,
}

impl Settler {
    pub fn new(
        rpc: Arc<dyn SolanaRpcClient>,
        builder: impl Into<FixedTermIxBuilder>,
        margin_accounts: AsyncNoDupeQueue<Pubkey>,
        config: SettleMarginUsersConfig,
    ) -> anyhow::Result<Settler> {
        if config.batch_size == 0 {
            anyhow::bail!("invalid settler batch size of 0");
        }

        Ok(Settler {
            rpc,
            builder: Arc::new(builder.into()),
            margin_accounts,
            retry_queue: Default::default(),
            config,
        })
    }

    /// Settles all accounts that currently need to be settled. Returns after
    /// all settlement attempts have completed and there are no accounts
    /// remaining in the queue. If any new accounts appear in the queue while
    /// this is processing, it will also attempt to settle those accounts.
    /// Returns error if any attempts fail.
    pub async fn settle_all(&self) -> anyhow::Result<()> {
        self.process_all(clone_to_async! { me = self
            |addrs| { try_settle(me.rpc.clone(), me.builder.clone(), &addrs).await }
        })
        .await
    }

    /// Loops forever to keep checking the queue for margin accounts. Sends a
    /// separate Settle transaction for each without blocking. Limits rate based
    /// on config. Any margin accounts that failed to settle are retried
    /// indefinitely.
    pub async fn run_forever(&self) {
        tracing::info!(
            "starting settler crank for fixed term market {}",
            self.builder.market()
        );
        loop {
            let me = self.clone();
            join!(
                tokio::time::sleep(self.config.wait_for_more_delay),
                async move {
                    me.process_all(clone_to_async! { me
                        |mut addrs| {
                            if let Err(e) = try_settle(me.rpc.clone(), me.builder.clone(), &addrs).await {
                                tracing::error!("failed to settle margin accounts {addrs:?} - {e}");
                                addrs.reverse();
                                me.retry_queue.push_many(addrs).await;
                            }
                            Ok(())
                        }
                    })
                    .await
                    .expect("settle_with_recovery is infallible")
                }
            );
        }
    }

    /// Apply the processor function to margin accounts in the queue until both
    /// the primary and the retry queues are empty.
    async fn process_all<F, Fut>(&self, f: F) -> anyhow::Result<()>
    where
        F: Fn(Vec<Pubkey>) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let mut remaining = self.margin_accounts.len().await;
        while remaining > 0 {
            let mut spawned = vec![];
            while remaining > 0 {
                let me = self.clone();
                let processor = f.clone();
                spawned.push(tokio::spawn(
                    async move { me.process_batch(processor).await },
                ));
                remaining = self.margin_accounts.len().await;
                if remaining > 0 {
                    tokio::time::sleep(self.config.batch_delay).await;
                }
            }
            try_join_all(spawned).await?;
            if !self.retry_queue.is_empty().await {
                // Just grab a single batch of retries so they don't get cause a
                // DoS for new accounts that appear in the main queue.
                self.margin_accounts
                    .push_many(self.retry_queue.pop_many(self.config.batch_size).await)
                    .await;
            }
            remaining = self.margin_accounts.len().await;
        }

        Ok(())
    }

    /// Apply the processor function to a group of margin accounts from the
    /// primary queue, equal to batch_size.
    async fn process_batch<F, Fut>(&self, processor: F) -> anyhow::Result<()>
    where
        F: Fn(Vec<Pubkey>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send,
    {
        join_all(
            self.margin_accounts
                .pop_many(self.config.batch_size)
                .await
                .chunks(SETTLES_PER_TX)
                .map(|addrs| {
                    let addrs = addrs.to_vec();
                    processor(addrs)
                }),
        )
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(())
    }
}
