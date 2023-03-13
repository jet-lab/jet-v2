use std::sync::Arc;

use anchor_lang::AccountDeserialize;
use anyhow::{bail, Result};
use async_trait::async_trait;
use jet_margin::MarginAccount;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_sdk::{pubkey::Pubkey, signer::Signer};

use crate::{
    fixed_term::{find_markets, FixedTermIxBuilder},
    ix_builder::accounting_invoke,
    margin_integrator::PositionRefresher,
    solana::transaction::TransactionBuilder,
};

#[async_trait]
impl PositionRefresher for FixedTermPositionRefresher {
    async fn refresh_positions(&self) -> Result<Vec<TransactionBuilder>> {
        let mut ret = vec![];
        let markets = find_markets(&self.rpc).await?;
        for market in markets.values() {
            let bldr = FixedTermIxBuilder::new_from_state(self.rpc.payer().pubkey(), market);
            for position in
                get_anchor_account::<MarginAccount>(self.rpc.clone(), &self.margin_account)
                    .await?
                    .positions()
                    .filter(|p| p.adapter == jet_fixed_term::id())
            {
                if position.token == bldr.claims() || position.token == bldr.collateral() {
                    ret.push(
                        accounting_invoke(
                            bldr.airspace(),
                            self.margin_account,
                            bldr.refresh_position(self.margin_account, false),
                        )
                        .into(),
                    )
                }
            }
        }

        Ok(ret)
    }
}

/// Refreshes margin positions that are managed by the Fixed Term Market program
pub struct FixedTermPositionRefresher {
    /// the address to search for positions
    margin_account: Pubkey,
    /// client to execute search for margin account
    rpc: Arc<dyn SolanaRpcClient>,
}

impl FixedTermPositionRefresher {
    /// instantiate
    pub fn new(margin_account: Pubkey, rpc: Arc<dyn SolanaRpcClient>) -> Self {
        Self {
            margin_account,
            rpc,
        }
    }
}

/// read an account on chain as an anchor type
pub async fn get_anchor_account<T: AccountDeserialize>(
    rpc: Arc<dyn SolanaRpcClient>,
    address: &Pubkey,
) -> Result<T> {
    let account_data = rpc.get_account(address).await?;

    match account_data {
        None => bail!("no account state found for account {}", address),
        Some(account) => Ok(T::try_deserialize(&mut &account.data[..])?),
    }
}
