use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::Result;

use itertools::Itertools;
use jet_margin_sdk::cat;
use jet_margin_sdk::solana::transaction::{SendTransactionBuilder, TransactionBuilder};
use jet_margin_sdk::spl_swap::SplSwapPool;
use jet_margin_sdk::tokens::TokenPrice;
use jet_margin_sdk::util::asynchronous::{AndAsync, MapAsync};
use jet_rpc::solana_rpc_api::{AsyncSigner, SolanaConnection, SolanaRpcClient};
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;

use tokio::try_join;

use crate::swap::{SwapPoolConfig, SwapRegistry};
use crate::tokens::TokenManager;

pub const ONE: u64 = 1_000_000_000;

pub struct TokenPricer {
    rpc: Arc<dyn SolanaConnection>,
    pub tokens: TokenManager,
    vaults: HashMap<Pubkey, Pubkey>,
    swap_registry: HashMap<Pubkey, HashMap<Pubkey, SplSwapPool>>,
}

impl Clone for TokenPricer {
    fn clone(&self) -> Self {
        Self {
            rpc: self.rpc.clone(),
            tokens: self.tokens.clone(),
            vaults: self.vaults.clone(),
            swap_registry: self.swap_registry.clone(),
        }
    }
}

impl TokenPricer {
    pub fn new_without_swaps(rpc: Arc<dyn SolanaConnection>) -> Self {
        Self {
            rpc: rpc.clone(),
            tokens: TokenManager::new(rpc.clone()),
            vaults: HashMap::new(),
            swap_registry: SwapRegistry::new(),
        }
    }

    pub fn new(
        rpc: Arc<dyn SolanaConnection>,
        vaults: HashMap<Pubkey, Pubkey>,
        swap_registry: &SwapRegistry,
    ) -> Self {
        Self {
            rpc: rpc.clone(),
            tokens: TokenManager::new(rpc.clone()),
            vaults,
            swap_registry: swap_registry.clone(),
        }
    }

    pub async fn refresh_all_oracles_timestamps(&self) -> Result<()> {
        self.refresh_oracles_timestamps(&self.vaults.keys().collect::<Vec<&Pubkey>>())
            .await
    }

    /// Updates oracles to say the same prices with a more recent timestamp
    pub async fn refresh_oracles_timestamps(&self, mints: &[&Pubkey]) -> Result<()> {
        let txs = mints
            .iter()
            .map_async(|mint| self.tokens.refresh_to_same_price_tx(mint))
            .await?;
        let ixns = txs
            .clone()
            .into_iter()
            .flat_map(|tx| tx.instructions)
            .collect::<Vec<Instruction>>();
        let signers = txs
            .into_iter()
            .flat_map(|tx| AsyncSigner::new(tx))
            .collect::<Vec<AsyncSigner>>();
        self.rpc.sign_send_instructions(&ixns, &signers).await?;

        Ok(())
    }

    pub async fn summarize_price(&self, mint: &Pubkey) -> Result<()> {
        println!("price summary for {mint}");
        let oracle_price = self.get_oracle_price(mint).await?;
        println!("    oracle: {oracle_price}");
        for (other_mint, pool) in self.swap_registry[mint].iter() {
            println!("    relative to {other_mint}");
            let other_price = self.get_oracle_price(other_mint).await?;
            let balances = pool.balances(&self.rpc).await?;
            let this_balance = *balances.get(mint).unwrap();
            let other_balance = *balances.get(other_mint).unwrap();
            let relative_price = other_balance as f64 / this_balance as f64;
            let derived_price = relative_price * other_price;
            println!("        other price    {other_price}");
            println!("        this  balance  {this_balance}");
            println!("        other balance  {other_balance}");
            println!("        relative price {relative_price}");
            println!("        derived price  {derived_price}");
        }
        Ok(())
    }

    /// Sets price in oracle and swap for only a single asset
    pub async fn set_price(&self, mint: &Pubkey, price: f64) -> Result<()> {
        let mut txs = self.set_price_in_swap_pools_tx(mint, price).await?;
        let oracle_tx = self.set_oracle_price_tx(mint, price)?;
        txs.push(oracle_tx);
        self.rpc.send_and_confirm_condensed(txs).await?;

        Ok(())
    }

    /// Effeciently sets prices in oracle and swap for many assets at once
    pub async fn set_prices(
        &self,
        mint_prices: Vec<(Pubkey, f64)>,
        refresh_unchanged: bool,
    ) -> Result<()> {
        let mut target_prices: HashMap<Pubkey, f64> = mint_prices.clone().into_iter().collect();
        let mints = target_prices.clone().into_keys().collect();
        let (swap_snapshot, oracle_snapshot) = try_join!(
            self.swap_snapshot(if refresh_unchanged {
                None
            } else {
                Some(&mints)
            }),
            self.oracle_snapshot(&mints),
        )?;
        target_prices.extend(oracle_snapshot);

        let mut txs = vec![];
        for pair in target_prices.iter().combinations(2) {
            let (mint_a, target_price_a) = pair[0];
            let (mint_b, target_price_b) = pair[1];
            if refresh_unchanged || mints.contains(mint_a) || mints.contains(mint_b) {
                let pool = self.swap_registry.get(mint_a).unwrap().get(mint_b).unwrap();
                let &(balance_a, balance_b) =
                    swap_snapshot.get(mint_a).unwrap().get(mint_b).unwrap();
                let desired_relative_price = target_price_a / target_price_b;
                let (a_to_swap, b_to_swap) =
                    set_constant_product_price(balance_a, balance_b, desired_relative_price);
                let vault_a = self.vaults.get(mint_a).unwrap();
                let vault_b = self.vaults.get(mint_b).unwrap();
                txs.push(cat![
                    pool.swap_tx(
                        self.rpc as Arc<dyn SolanaRpcClient>,
                        vault_a,
                        vault_b,
                        a_to_swap,
                        &self.payer
                    )
                    .await?,
                    pool.swap_tx(&self.rpc, vault_b, vault_a, b_to_swap, &self.payer)
                        .await?,
                ]);
            }
        }
        for (mint, price) in if refresh_unchanged {
            target_prices.into_iter().collect()
        } else {
            mint_prices
        } {
            txs.push(self.set_oracle_price_tx(&mint, price)?)
        }

        self.rpc.send_and_confirm_condensed(txs).await?;

        Ok(())
    }

    pub async fn oracle_snapshot(
        &self,
        blacklist: &HashSet<Pubkey>,
    ) -> Result<HashMap<Pubkey, f64>> {
        Ok(self
            .vaults
            .keys()
            .filter(|m| !blacklist.contains(m))
            .map_async(|m| (*m).and_result(self.get_oracle_price(m)))
            .await?
            .into_iter()
            .collect())
    }

    pub async fn swap_snapshot(&self, whitelist: Option<&HashSet<Pubkey>>) -> Result<SwapSnapshot> {
        let mut pools = HashSet::new();
        let mut balance_checks = Vec::new();
        for (mint_a, others) in self.swap_registry.iter() {
            for (mint_b, pool) in others {
                let pair = (*mint_a, *mint_b);
                if !pools.contains(&pair)
                    && (whitelist.is_none()
                        || whitelist.unwrap().contains(mint_a)
                        || whitelist.unwrap().contains(mint_b))
                {
                    pools.insert(pair);
                    pools.insert((*mint_b, *mint_a));
                    balance_checks.push((pair, pool));
                }
            }
        }
        let mut snapshot = SwapSnapshot::new();
        for ((mint_a, mint_b), balances) in balance_checks
            .iter()
            .map_async(|(mints, pool)| mints.and_result(pool.balances(&self.rpc)))
            .await?
        {
            snapshot.entry(*mint_a).or_default().insert(
                *mint_b,
                (
                    *balances.get(mint_a).unwrap(),
                    *balances.get(mint_b).unwrap(),
                ),
            );
            snapshot.entry(*mint_b).or_default().insert(
                *mint_a,
                (
                    *balances.get(mint_b).unwrap(),
                    *balances.get(mint_a).unwrap(),
                ),
            );
        }

        Ok(snapshot)
    }

    /// If you set the price to disagree with the oracle, you may see some odd behavior.
    /// Think carefully about how other tokens are impacted.
    pub async fn set_price_in_swap_pools_tx(
        &self,
        mint: &Pubkey,
        price: f64,
    ) -> Result<Vec<TransactionBuilder>> {
        let mut txs = vec![];
        for (other_mint, pool) in self.swap_registry[mint].iter() {
            let other_price = self.get_oracle_price(other_mint).await?;
            let desired_relative_price = price / other_price;
            let balances = pool.balances(&self.rpc).await?;
            let (this_to_swap, other_to_swap) = set_constant_product_price(
                *balances.get(mint).unwrap(),
                *balances.get(other_mint).unwrap(),
                desired_relative_price,
            );
            let this = self.vaults.get(mint).unwrap();
            let other = self.vaults.get(other_mint).unwrap();
            txs.push(cat![
                pool.swap_tx(&self.rpc, this, other, this_to_swap, &self.payer)
                    .await?,
                pool.swap_tx(&self.rpc, other, this, other_to_swap, &self.payer)
                    .await?,
            ]);
        }

        Ok(txs)
    }

    pub fn set_oracle_price_tx(&self, mint: &Pubkey, price: f64) -> Result<TransactionBuilder> {
        let price = (price * 100_000_000.0) as i64;
        self.tokens.set_price_tx(
            mint,
            &TokenPrice {
                exponent: -8,
                price,
                confidence: 0,
                twap: price as u64,
            },
        )
    }

    pub async fn get_oracle_price(&self, mint: &Pubkey) -> Result<f64> {
        let px = self.tokens.get_price(mint).await?;
        let price = px.agg.price as f64 * (10f64.powf(px.expo.into()));

        Ok(price)
    }
}

type SwapSnapshot = HashMap<Pubkey, HashMap<Pubkey, (u64, u64)>>;

/// Returns the amount of assets a and b to swap into the pool to slip
/// the assets into the desired relative price
fn set_constant_product_price(
    balance_a: u64,
    balance_b: u64,
    desired_price_of_a: f64,
) -> (u64, u64) {
    let product = balance_a as f64 * balance_b as f64;
    let current_price_of_a = balance_b as f64 / balance_a as f64;
    if desired_price_of_a == current_price_of_a {
        (0, 0)
    } else if desired_price_of_a < current_price_of_a {
        ((product / desired_price_of_a).sqrt() as u64 - balance_a, 0)
    } else {
        (
            0,
            ((product * desired_price_of_a).sqrt() as u64 - balance_b),
        )
    }
    /*
        derivation

        balance_a * balance_b = product
        new_balance_a * new_balance_b = product
        new_balance_a = balance_a + move_a
        new_balance_b = balance_b + move_b

        new_balance_a * (balance_b + move_b) = product
        new_balance_a * balance_b + new_balance_a * move_b = product
        new_balance_a * move_b = product - new_balance_a * balance_b
        move_b = (product - new_balance_a * balance_b) / new_balance_a
        move_b = product / new_balance_a - balance_b
        move_b = product / (balance_a + move_a) - balance_b

        let desired_price = new_balance_b / new_balance_a

        desired_price = (balance_b + move_b) / (balance_a + move_a)
        desired_price = (balance_b + product / (balance_a + move_a) - balance_b) / (balance_a + move_a)
        desired_price = (product / (balance_a + move_a)) / (balance_a + move_a)
        desired_price = product / (balance_a + move_a)^2
        sqrt(product/desired_price) = balance_a + move_a

        move_a = sqrt(product/desired_price) - balance_a
    */
}
