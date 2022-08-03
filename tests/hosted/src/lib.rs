use anyhow::Result;
use async_trait::async_trait;
use futures::future::join_all;
use solana_sdk::signature::Keypair;

pub mod context;
pub mod load;
pub mod margin;
pub mod orchestrator;
pub mod setup_helper;
pub mod swap;
pub mod tokens;

pub fn clone(keypair: &Keypair) -> Keypair {
    Keypair::from_bytes(&keypair.to_bytes()).unwrap()
}

#[async_trait]
pub trait MapAsync<Item>: Iterator<Item = Item> + std::marker::Send + Sized {
    async fn map_async<
        Ret: std::fmt::Debug + std::marker::Send,
        Fut: futures::Future<Output = Result<Ret>> + std::marker::Send,
        F: Fn(Item) -> Fut + std::marker::Send,
    >(
        self,
        f: F,
    ) -> Result<Vec<Ret>> {
        join_all(self.map(f))
            .await
            .into_iter()
            .collect::<Result<Vec<Ret>>>()
    }
}

impl<Item, Iter: Iterator<Item = Item> + std::marker::Send + Sized> MapAsync<Item> for Iter {}
