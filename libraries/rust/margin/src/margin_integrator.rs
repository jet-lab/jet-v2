use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use futures::future::join_all;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use tokio::join;

use crate::{
    ix_builder::MarginIxBuilder, solana::transaction::TransactionBuilder,
    util::asynchronous::MapAsync,
};

///
pub struct FreshInvoker<P: Proxy> {
    proxy: P,
    refreshers: Vec<Arc<dyn PositionRefresher>>,
}

impl<P: Proxy> FreshInvoker<P> {
    async fn refresh_and_invoke_signed(&self, ix: Instruction) -> Result<Vec<TransactionBuilder>> {
        let mut refresh = join_all(
            self.refreshers
                .clone()
                .iter()
                .map(|r| r.refresh_positions()),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<Vec<_>>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        refresh.push(self.proxy.invoke_signed(ix).into());

        Ok(refresh)
    }
}

/// Enable generic refreshing of any margin positions without caring how
#[async_trait]
pub trait PositionRefresher {
    /// same as above
    async fn refresh_positions(&self) -> Result<Vec<TransactionBuilder>>;
}

/// Allows wrapping of instructions for execution by a program that acts as a
/// proxy, such as margin
pub trait Proxy {
    /// the address of the proxying account, such as the margin account
    fn pubkey(&self) -> Pubkey;
    /// when no signature is needed by the proxy
    fn invoke(&self, ix: Instruction) -> Instruction;
    /// when the proxy will need to sign
    fn invoke_signed(&self, ix: Instruction) -> Instruction;
}

/// Dummy proxy implementation that passes along instructions untouched
pub struct NoProxy(pub Pubkey);
impl Proxy for NoProxy {
    fn pubkey(&self) -> Pubkey {
        self.0
    }

    fn invoke(&self, ix: Instruction) -> Instruction {
        ix
    }

    fn invoke_signed(&self, ix: Instruction) -> Instruction {
        ix
    }
}

/// Proxies instructions through margin
impl Proxy for MarginIxBuilder {
    fn pubkey(&self) -> Pubkey {
        self.address
    }

    fn invoke(&self, ix: Instruction) -> Instruction {
        self.accounting_invoke(ix)
    }

    fn invoke_signed(&self, ix: Instruction) -> Instruction {
        self.adapter_invoke(ix)
    }
}
