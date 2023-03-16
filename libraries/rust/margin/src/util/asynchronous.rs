use anyhow::Result;
use async_recursion::async_recursion;
use async_trait::async_trait;
use futures::future::join_all;
use futures::Future;
use std::marker::Send;
use std::time::Duration;
use tokio::select;

/// Execute iterator of asyncs in parallel and return a vec of the results
#[async_trait]
pub trait MapAsync<Item: Send>: Iterator<Item = Item> + Sized {
    /// base case, execute all at once
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

    /// Execute only chunk_size at a time before starting the next chunk
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
                        extend_with_joined(&mut ret, progress).await?;
                        return Ok(ret);
                    }
                }
            }
            extend_with_joined(&mut ret, progress).await?;
        }
    }
}

async fn extend_with_joined<
    T: std::fmt::Debug + Send + Clone,
    Fut: futures::Future<Output = Result<T>> + Send,
>(
    extendable: &mut Vec<T>,
    extend_with: Vec<Fut>,
) -> Result<()> {
    let all = join_all(extend_with)
        .await
        .into_iter()
        .collect::<Result<Vec<T>>>()?;
    extendable.extend_from_slice(&all);

    Ok(())
}

impl<Item: Send, Iter: Iterator<Item = Item> + Sized> MapAsync<Item> for Iter {}

/// Useful since async lambdas are unstable
#[async_trait]
pub trait AndAsync: Sized {
    /// combine item with a future result to return from a lambda
    async fn and<R, Fut: futures::Future<Output = R> + Send>(self, fut: Fut) -> (Self, R) {
        (self, fut.await)
    }

    /// combine item with a future Result to return from a lambda
    async fn and_result<R, Fut: futures::Future<Output = Result<R>> + Send>(
        self,
        fut: Fut,
    ) -> Result<(Self, R)> {
        Ok((self, fut.await?))
    }
}

impl<T: Sized> AndAsync for T {}

/// Execute job derived from a future builder, retrying with some frequency, with backoff. cancel all after a timeout
pub async fn with_retries_and_timeout<T, Fut: Future<Output = T> + Send, F: Fn() -> Fut + Send>(
    f: F,
    first_delay: Duration,
    timeout: u64,
) -> Result<T> {
    Ok(tokio::time::timeout(Duration::from_secs(timeout), with_retries(f, first_delay)).await?)
}

/// Execute job derived from a future builder, retrying with some frequency, with backoff
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

#[cfg(test)]
mod test {
    use super::*;

    static mut COUNT: u8 = 0;

    #[tokio::test(flavor = "multi_thread")]
    async fn retry_test() -> Result<(), anyhow::Error> {
        let x = with_retries(counter, Duration::from_millis(1)).await;
        assert_eq!(3, x);
        Ok(())
    }

    async fn counter() -> u8 {
        unsafe {
            COUNT += 1;
            if COUNT < 3 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            COUNT
        }
    }
}
