use std::{collections::HashMap, sync::Arc};

use anchor_lang::AccountDeserialize;
use anyhow::{bail, Result};
use async_trait::async_trait;
use jet_bonds::control::state::BondManager;
use jet_margin::MarginAccount;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::{
    bonds::BondsIxBuilder, margin_integrator::PositionRefresher,
    solana::transaction::TransactionBuilder,
};

#[async_trait]
impl PositionRefresher for MarginBondsIntegrator {
    async fn refresh_positions(&self) -> Result<Vec<TransactionBuilder>> {
        let mut ret = vec![];
        for bond_market in self.bond_markets.values() {
            for position in
                get_anchor_account::<MarginAccount>(self.rpc.clone(), &self.margin_account)
                    .await?
                    .positions()
                    .filter(|p| p.adapter == jet_bonds::id())
            {
                if position.address == bond_market.claims()
                    || position.address == bond_market.collateral()
                {
                    ret.push(bond_market.refresh_position(self.margin_account)?.into())
                }
            }
        }

        Ok(ret)
    }
}

struct MarginBondsIntegrator {
    margin_account: Pubkey,
    bond_markets: HashMap<Pubkey, BondsIxBuilder>,
    rpc: Arc<dyn SolanaRpcClient>,
}

impl MarginBondsIntegrator {
    async fn add_bond_market(&mut self, manager: Pubkey) -> Result<()> {
        self.bond_markets.insert(
            manager,
            get_anchor_account::<BondManager>(self.rpc.clone(), &manager)
                .await?
                .into(),
        );

        Ok(())
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
