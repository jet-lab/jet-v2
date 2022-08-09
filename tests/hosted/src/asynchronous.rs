use anyhow::Result;
use async_recursion::async_recursion;
use async_trait::async_trait;
use futures::future::join_all;
use futures::Future;
use std::marker::Send;
use std::time::Duration;
use tokio::select;

#[async_trait]
pub trait MapAsync<Item: Send>: Iterator<Item = Item> + Sized {
    async fn map_async<
        Ret: std::fmt::Debug + Send,
        Fut: futures::Future<Output = Result<Ret>> + Send,
        F: Fn(Item) -> Fut + Send,
    >(
        self,
        f: F,
    ) -> Result<Vec<Ret>> {
        join_all(self.map(f))
            .await
            .into_iter()
            .collect::<Result<Vec<Ret>>>()
    }

    async fn map_async_chunked<
        Ret: std::fmt::Debug + Send + Clone,
        Fut: futures::Future<Output = Result<Ret>> + Send,
        F: Fn(Item) -> Fut + Send,
    >(
        mut self,
        chunk_size: usize,
        f: F,
    ) -> Result<Vec<Ret>> {
        let mut ret = vec![];
        loop {
            let mut progress = vec![];
            for _ in 0..chunk_size {
                match self.next() {
                    Some(x) => progress.push(f(x)),
                    None => {
                        let all = join_all(progress)
                            .await
                            .into_iter()
                            .collect::<Result<Vec<Ret>>>()?;
                        ret.extend_from_slice(&all);
                        return Ok(ret);
                    }
                }
            }
            let all = join_all(progress)
                .await
                .into_iter()
                .collect::<Result<Vec<Ret>>>()?;
            ret.extend_from_slice(&all)
        }
    }
}

impl<Item: Send, Iter: Iterator<Item = Item> + Sized> MapAsync<Item> for Iter {}

/// Useful since async lambdas are unstable
#[async_trait]
pub trait AndAsync: Sized {
    async fn and<R, Fut: futures::Future<Output = R> + Send>(self, fut: Fut) -> (Self, R) {
        (self, fut.await)
    }

    async fn and_result<R, Fut: futures::Future<Output = Result<R>> + Send>(
        self,
        fut: Fut,
    ) -> Result<(Self, R)> {
        Ok((self, fut.await?))
    }
}

impl<T: Sized> AndAsync for T {}

pub async fn with_retries_and_timeout<T, Fut: Future<Output = T> + Send, F: Fn() -> Fut + Send>(
    f: F,
    first_delay: Duration,
    timeout: u64,
) -> Result<T> {
    Ok(tokio::time::timeout(Duration::from_secs(timeout), with_retries(f, first_delay)).await?)
}

#[async_recursion]
pub async fn with_retries<T, Fut: Future<Output = T> + Send, F: Fn() -> Fut + Send>(
    f: F,
    next_delay: Duration,
) -> T {
    select! {
        x = f() => x,
        x = sleep_then_retry(f, next_delay) => x,
    }
}

#[async_recursion]
async fn sleep_then_retry<T, Fut: Future<Output = T> + Send, F: Fn() -> Fut + Send>(
    f: F,
    next_delay: Duration,
) -> T {
    tokio::time::sleep(next_delay).await;
    with_retries(f, next_delay * 2).await
}
