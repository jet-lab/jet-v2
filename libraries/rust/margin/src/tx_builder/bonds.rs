use std::{collections::HashMap, sync::Arc};

use anchor_lang::AccountDeserialize;
use anyhow::{bail, Result};
use async_trait::async_trait;
use jet_margin::MarginAccount;
use jet_market::control::state::MarketManager;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::{
    bonds::FixedMarketIxBuilder, ix_builder::accounting_invoke, margin_integrator::PositionRefresher,
    solana::transaction::TransactionBuilder,
};

#[async_trait]
impl PositionRefresher for FixedPositionRefresher {
    async fn refresh_positions(&self) -> Result<Vec<TransactionBuilder>> {
        let mut ret = vec![];
        for fixed_market in self.fixed_markets.values() {
            for position in
                get_anchor_account::<MarginAccount>(self.rpc.clone(), &self.margin_account)
                    .await?
                    .positions()
                    .filter(|p| p.adapter == jet_market::id())
            {
                if position.token == fixed_market.claims()
                    || position.token == fixed_market.collateral()
                {
                    ret.push(
                        accounting_invoke(
                            self.margin_account,
                            fixed_market.refresh_position(self.margin_account, false)?,
                        )
                        .into(),
                    )
                }
            }
        }

        Ok(ret)
    }
}

/// Refreshes margin positions that are managed by the bonds program
pub struct FixedPositionRefresher {
    /// the address to search for positions
    margin_account: Pubkey,
    /// known fixed markets that may or may not have positions
    fixed_markets: HashMap<Pubkey, FixedMarketIxBuilder>,
    /// client to execute search for margin account
    rpc: Arc<dyn SolanaRpcClient>,
}

impl FixedPositionRefresher {
    /// search for the fixed markets and then instantiate the struct
    pub async fn new(
        margin_account: Pubkey,
        rpc: Arc<dyn SolanaRpcClient>,
        fixed_markets: &[Pubkey],
    ) -> Result<Self> {
        let mut ret = Self {
            margin_account,
            fixed_markets: HashMap::new(),
            rpc,
        };
        for b in fixed_markets {
            ret.add_fixed_market(*b).await?;
        }

        Ok(ret)
    }

    /// register a fixed market to check when refreshing positions
    pub async fn add_fixed_market(&mut self, manager: Pubkey) -> Result<()> {
        self.fixed_markets.insert(
            manager,
            get_anchor_account::<MarketManager>(self.rpc.clone(), &manager)
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
