use std::{fmt::Debug, hash::Hash, time::Duration};

use async_trait::async_trait;
use futures::{
    future::{join_all, try_join_all},
    join, Future,
};
use jet_solana_client::clone_to_async;

use crate::util::no_dupe_queue::AsyncNoDupeQueue;

/// Performance settings for the QueueProcessor. The defaults are designed for
/// use as a Settler.
#[derive(Clone, Copy, Debug)]
pub struct QueueProcessorConfig {
    /// Number of items to process simultaneously. This will be broken down into
    /// multiple instructions of chunk_size that are executed simultaneously.
    pub batch_size: usize,

    /// Quantity to send to the processor with a single call.
    pub chunk_size: usize,

    /// Time to wait between batches when the previous batch maxed out the
    /// batch_size and there are still more items to process.
    pub batch_delay: Duration,

    /// Time to wait between batches when the previous batch cleared the entire
    /// queue and there are no more items to process at this time.
    pub wait_for_more_delay: Duration,
}
impl Default for QueueProcessorConfig {
    /// Attempts to set a conservative default that can safely coexist with an
    /// event consumer that deserves higher CPU priority since its tasks are
    /// more important.
    fn default() -> Self {
        Self {
            batch_size: 10,
            chunk_size: 3,
            batch_delay: Duration::from_secs(1),
            wait_for_more_delay: Duration::from_secs(5),
        }
    }
}

/// A QueueProcessor that always uses the same function to process its items.
pub struct StaticQueueProcessor<T: Processable, C: ChunkProcessor<T>> {
    queue_processor: QueueProcessor<T>,
    chunk_processor: C,
}
/// Implement this to use StaticQueueProcesso
#[async_trait]
pub trait ChunkProcessor<T>: Clone + Send + Sync + 'static {
    /// process a single chunk of chunk_size
    async fn process(&self, chunk: Vec<T>) -> anyhow::Result<()>;
}

impl<T: Processable, C: ChunkProcessor<T>> StaticQueueProcessor<T, C> {
    /// constructor
    pub fn new(
        input_queue: AsyncNoDupeQueue<T>,
        config: QueueProcessorConfig,
        chunk_processor: C,
    ) -> anyhow::Result<Self> {
        if config.batch_size == 0 {
            anyhow::bail!("invalid queue processor batch size of 0");
        }

        Ok(Self {
            queue_processor: QueueProcessor::new(input_queue, config)?,
            chunk_processor,
        })
    }

    /// see QueueProcessor::process_forever
    pub async fn process_forever(&self) {
        self.queue_processor
            .process_forever(clone_to_async! { (p = self.chunk_processor) |x| p.process(x).await  })
            .await
    }

    /// see QueueProcessor::process_all
    pub async fn process_all(&self) -> anyhow::Result<()> {
        self.queue_processor
            .process_all(clone_to_async! { (p = self.chunk_processor) |x| p.process(x).await })
            .await
    }
}

/// Keeps track of a queue and exposes functions to process the events in that
/// queue
#[derive(Clone)]
pub struct QueueProcessor<T: Processable> {
    input_queue: AsyncNoDupeQueue<T>,
    retry_queue: AsyncNoDupeQueue<T>,
    config: QueueProcessorConfig,
}

impl<T: Processable> QueueProcessor<T> {
    /// constructor
    pub fn new(
        input_queue: AsyncNoDupeQueue<T>,
        config: QueueProcessorConfig,
    ) -> anyhow::Result<Self> {
        if config.batch_size == 0 {
            anyhow::bail!("invalid queue processor batch size of 0");
        }

        Ok(Self {
            input_queue,
            retry_queue: Default::default(),
            config,
        })
    }

    /// Loops forever to keep checking the queue for items. Sends a separate
    /// Settle transaction for each without blocking. Limits rate based on
    /// config. Any items that failed to process retried indefinitely.
    pub async fn process_forever<F, Fut>(&self, f: F)
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        loop {
            let me = self.clone();
            let f = f.clone();
            join!(
                tokio::time::sleep(self.config.wait_for_more_delay),
                async move {
                    me.process_all(clone_to_async! { (f, me)
                        |mut items| {
                            if let Err(e) = f(items.clone()).await {
                                tracing::error!("failed to process {items:?} - {e}");
                                items.reverse();
                                me.retry_queue.push_many(items).await;
                            }
                            Ok(())
                        }
                    })
                    .await
                    .expect("recovery makes this is infallible")
                }
            );
        }
    }

    /// Apply the processor function to items in the queue until both the
    /// primary and the retry queues are empty.
    pub async fn process_all<F, Fut>(&self, f: F) -> anyhow::Result<()>
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let mut remaining = self.input_queue.len().await;
        while remaining > 0 {
            let mut spawned = vec![];
            while remaining > 0 {
                let me = self.clone();
                let processor = f.clone();
                spawned.push(tokio::spawn(
                    async move { me.process_batch(processor).await },
                ));
                remaining = self.input_queue.len().await;
                if remaining > 0 {
                    tokio::time::sleep(self.config.batch_delay).await;
                }
            }
            try_join_all(spawned).await?;
            if !self.retry_queue.is_empty().await {
                // Just grab a single batch of retries so they don't get cause a
                // DoS for new items that appear in the main queue.
                self.input_queue
                    .push_many(self.retry_queue.pop_many(self.config.batch_size).await)
                    .await;
            }
            remaining = self.input_queue.len().await;
        }

        Ok(())
    }

    /// Apply the processor function to a group of items from the primary queue,
    /// equal to batch_size, chunked in sizes of chunk_size
    async fn process_batch<F, Fut>(&self, f: F) -> anyhow::Result<()>
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send,
    {
        join_all(
            self.input_queue
                .pop_many(self.config.batch_size)
                .await
                .chunks(self.config.chunk_size)
                .map(|items| {
                    let items = items.to_vec();
                    f(items)
                }),
        )
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(())
    }
}

/// Everything that needs to be implemented to be used with QueueProcessor
pub trait Processable: Clone + Debug + Hash + Eq + Send + Sync + 'static {}
impl<T> Processable for T where T: Clone + Debug + Hash + Eq + Send + Sync + 'static {}
