use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use jet_instructions::margin::derive_margin_account;
use jet_simulation::SolanaRpcClient;
use jet_solana_client::signature::StandardizeSigner;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer};

use crate::{
    ix_builder::MarginIxBuilder,
    refresh::{
        canonical_position_refresher,
        position_refresher::{PositionRefresher, SmartRefresher},
    },
    solana::transaction::{TransactionBuilder, WithSigner},
};

/// A variant of Proxy with the ability to refresh a margin account's positions.
/// This makes it easier to invoke_signed an adapter program while abstracting
/// away all the special requirements of the margin account into only this
/// single struct.
///
/// This is a separate struct rather than having the functionality added to
/// MarginIxBuilder because it has expensive dependencies like rpc clients and
/// other adapter implementations, and it's not appropriate to make these things
/// required for a simple single-program instruction builder like MarginIxBuilder.
#[derive(Clone)]
pub struct RefreshingProxy<P: Proxy> {
    /// underlying proxy
    pub proxy: P,
    /// adapter-specific implementations to refresh positions in a margin account
    pub refresher: SmartRefresher,
}

impl RefreshingProxy<MarginIxBuilder> {
    /// This returns a proxy that is expected to know about all the possible
    /// positions that may need to be refreshed.
    pub fn full(
        rpc: &Arc<dyn SolanaRpcClient>,
        wallet: &Keypair,
        seed: u16,
        airspace: Pubkey,
    ) -> Self {
        RefreshingProxy {
            proxy: MarginIxBuilder::new(airspace, wallet.pubkey(), seed)
                .with_payer(rpc.payer().pubkey()),
            refresher: canonical_position_refresher(rpc.clone())
                .for_address(derive_margin_account(&airspace, &wallet.pubkey(), seed)),
        }
    }
}

impl<P: Proxy> RefreshingProxy<P> {
    /// The instructions to refresh any positions that are refreshable by the
    /// included refreshers.
    pub async fn refresh(&self) -> Result<Vec<TransactionBuilder>> {
        self.refresher.refresh_positions(&()).await
    }
}

#[async_trait(?Send)]
impl<P: Proxy> Proxy for RefreshingProxy<P> {
    async fn refresh_and_invoke_signed(
        &self,
        ix: Instruction,
        signer: Keypair,
    ) -> Result<Vec<TransactionBuilder>> {
        let mut refresh = self.refresh().await?;
        refresh.push(
            self.proxy
                .invoke_signed(ix)
                .with_signer(signer.standardize()),
        );

        Ok(refresh)
    }

    async fn refresh(&self) -> Result<Vec<TransactionBuilder>> {
        self.refresh().await
    }

    fn pubkey(&self) -> Pubkey {
        self.proxy.pubkey()
    }

    fn invoke(&self, ix: Instruction) -> Instruction {
        self.proxy.invoke(ix)
    }

    fn invoke_signed(&self, ix: Instruction) -> Instruction {
        self.proxy.invoke_signed(ix)
    }
}

/// Allows wrapping of instructions for execution by a program that acts as a
/// proxy, such as margin
#[async_trait(?Send)]
pub trait Proxy {
    /// the address of the proxying account, such as the margin account
    fn pubkey(&self) -> Pubkey;
    /// when no signature is needed by the proxy
    fn invoke(&self, ix: Instruction) -> Instruction;
    /// when the proxy will need to sign
    fn invoke_signed(&self, ix: Instruction) -> Instruction;
    /// attempt to refresh any positions where the refresh method is understood
    /// by the proxy implementation.
    async fn refresh_and_invoke_signed(
        &self,
        ix: Instruction,
        signer: Keypair, //todo Signer
    ) -> Result<Vec<TransactionBuilder>> {
        Ok(vec![self
            .invoke_signed(ix)
            .with_signer(signer.standardize())])
    }
    /// attempt to refresh any positions where the refresh method is understood
    /// by the proxy implementation.
    async fn refresh(&self) -> Result<Vec<TransactionBuilder>> {
        Ok(vec![])
    }
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
