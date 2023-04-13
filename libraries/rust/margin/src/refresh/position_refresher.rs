use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use jet_margin::MarginAccount;
use jet_simulation::SolanaRpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::{get_state::get_margin_account, solana::transaction::TransactionBuilder};

/// see method
#[async_trait]
pub trait PositionRefresher<M> {
    /// Generically refresh margin account positions without caring how.  
    async fn refresh_positions(&self, margin_account: &M) -> Result<Vec<TransactionBuilder>>;
}

/// Combines multiple refreshers together and executes refreshes efficiently,
/// optionally with an RpcClient and/or address for the margin account.
#[derive(Clone)]
pub struct SmartRefresher<Address = Pubkey, Rpc = Arc<dyn SolanaRpcClient>> {
    /// This refresher is built on top of others
    pub refreshers: Vec<Arc<dyn PositionRefresher<MarginAccount> + Send + Sync>>,
    /// This refresher optionally takes an Rpc client, which can be used to
    /// retrieve the margin account
    pub rpc: Rpc,
    /// This refresher optionally takes an margin account address, which can be
    /// used to retrieve the margin account.
    pub margin_account: Address,
}

/// Builder methods for the SmartRefresher instance
impl<Address, Rpc> SmartRefresher<Address, Rpc> {
    /// include the margin account address in the struct, so refresh can be
    /// called without providing an address.
    pub fn for_address(self, margin_account: Pubkey) -> SmartRefresher<Pubkey, Rpc> {
        SmartRefresher {
            refreshers: self.refreshers,
            rpc: self.rpc,
            margin_account,
        }
    }
}

/// impl PositionRefresher for SmartRefresher
#[async_trait]
impl<A: Send + Sync, R: Send + Sync> PositionRefresher<MarginAccount> for SmartRefresher<A, R> {
    async fn refresh_positions(
        &self,
        margin_account: &MarginAccount,
    ) -> Result<Vec<TransactionBuilder>> {
        let mut output = vec![];
        for item in &self.refreshers {
            output.extend(item.refresh_positions(margin_account).await?);
        }
        Ok(output)
    }
}

pub use fancy::{HasMarginAccountAddress, HasRpc};

/// Simplifies call sites. When the data structure already contains an rpc
/// client or a margin account address, it should implement the relevant trait,
/// and then you don't have to provide it when calling refresh_positions.
mod fancy {
    use super::*;

    /// The struct contains an rpc client
    pub trait HasRpc {
        /// Returns the rpc client contained by the struct
        fn rpc(&self) -> Arc<dyn SolanaRpcClient>;
    }

    /// The struct contains a margin account's address
    pub trait HasMarginAccountAddress {
        /// Returns the address contained by the struct
        fn margin_account_address(&self) -> Pubkey;
    }

    impl<Address> HasRpc for SmartRefresher<Address> {
        fn rpc(&self) -> Arc<dyn SolanaRpcClient> {
            self.rpc.clone()
        }
    }

    impl<Rpc> HasMarginAccountAddress for SmartRefresher<Pubkey, Rpc> {
        fn margin_account_address(&self) -> Pubkey {
            self.margin_account
        }
    }

    #[async_trait]
    impl<P: HasRpc + PositionRefresher<MarginAccount> + Sync> PositionRefresher<Pubkey> for P {
        async fn refresh_positions(
            &self,
            margin_account: &Pubkey,
        ) -> Result<Vec<TransactionBuilder>> {
            self.refresh_positions(&get_margin_account(&self.rpc(), margin_account).await?)
                .await
        }
    }

    #[async_trait]
    impl<P: HasMarginAccountAddress + PositionRefresher<Pubkey> + Sync> PositionRefresher<()> for P {
        async fn refresh_positions(&self, _: &()) -> Result<Vec<TransactionBuilder>> {
            self.refresh_positions(&self.margin_account_address()).await
        }
    }
}

/// Defines a new position refresher type based on the typical pattern:
/// - needs an rpc client
/// - has no margin account address
/// - delegates to a function that takes (&rpc, &MarginAccount)
macro_rules! define_refresher {
    ($RefresherName:ident, $refresh_function:ident) => {
        /// refreshes positions known in this scope
        pub struct $RefresherName {
            /// need to read state to determine how to refresh positions
            pub rpc: std::sync::Arc<dyn jet_simulation::SolanaRpcClient>,
        }

        #[async_trait::async_trait]
        impl crate::refresh::position_refresher::PositionRefresher<jet_margin::MarginAccount>
            for $RefresherName
        {
            async fn refresh_positions(
                &self,
                margin_account: &jet_margin::MarginAccount,
            ) -> anyhow::Result<Vec<jet_solana_client::transaction::TransactionBuilder>> {
                $refresh_function(&self.rpc, margin_account).await
            }
        }

        impl crate::refresh::position_refresher::HasRpc for $RefresherName {
            fn rpc(&self) -> std::sync::Arc<dyn jet_simulation::SolanaRpcClient> {
                self.rpc.clone()
            }
        }
    };
}
pub(crate) use define_refresher;
