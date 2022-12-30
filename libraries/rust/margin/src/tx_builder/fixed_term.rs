use std::{collections::HashMap, sync::Arc};

use anchor_lang::AccountDeserialize;
use anyhow::{bail, Result};
use async_trait::async_trait;
use jet_fixed_term::control::state::Market;
use jet_margin::MarginAccount;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_sdk::{pubkey::Pubkey, signer::Signer};

use crate::{
    fixed_term::FixedTermIxBuilder, ix_builder::accounting_invoke,
    margin_integrator::PositionRefresher, solana::transaction::TransactionBuilder,
};

#[async_trait]
impl PositionRefresher for FixedTermPositionRefresher {
    async fn refresh_positions(&self) -> Result<Vec<TransactionBuilder>> {
        let mut ret = vec![];
        for fixed_term_market in self.fixed_term_markets.values() {
            for position in
                get_anchor_account::<MarginAccount>(self.rpc.clone(), &self.margin_account)
                    .await?
                    .positions()
                    .filter(|p| p.adapter == jet_fixed_term::id())
            {
                if position.token == fixed_term_market.claims()
                    || position.token == fixed_term_market.collateral()
                {
                    ret.push(
                        accounting_invoke(
                            self.margin_account,
                            fixed_term_market.refresh_position(self.margin_account, false),
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
    /// known fixed term markets that may or may not have positions
    fixed_term_markets: HashMap<Pubkey, FixedTermIxBuilder>,
    /// client to execute search for margin account
    rpc: Arc<dyn SolanaRpcClient>,
}

impl FixedTermPositionRefresher {
    /// search for the fixed term markets and then instantiate the struct
    pub async fn new(
        margin_account: Pubkey,
        rpc: Arc<dyn SolanaRpcClient>,
        fixed_term_markets: &[Pubkey],
    ) -> Result<Self> {
        let mut ret = Self {
            margin_account,
            fixed_term_markets: HashMap::new(),
            rpc,
        };
        for b in fixed_term_markets {
            ret.add_fixed_term_market(*b).await?;
        }

        Ok(ret)
    }

    /// register a fixed term market to check when refreshing positions
    pub async fn add_fixed_term_market(&mut self, address: Pubkey) -> Result<()> {
        let market = get_anchor_account::<Market>(self.rpc.clone(), &address).await?;
        let builder = FixedTermIxBuilder::new_from_state(self.rpc.payer().pubkey(), &market);

        self.fixed_term_markets.insert(address, builder);

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
