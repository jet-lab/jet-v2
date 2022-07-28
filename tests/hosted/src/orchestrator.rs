use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Error, Result};

use itertools::Itertools;
use jet_margin_sdk::swap::SwapPool;
use jet_margin_sdk::tokens::TokenPrice;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use jet_static_program_registry::orca_swap_v2;
use solana_sdk::clock::UnixTimestamp;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use jet_margin_pool::{MarginPoolConfig, PoolFlags, TokenChange};
use jet_metadata::TokenKind;
use jet_simulation::create_wallet;

use crate::clone;
use crate::context::MarginTestContext;
use crate::setup_helper::TestUser;
use crate::swap::SwapPoolConfig;
use crate::tokens::TokenManager;

pub struct TokenPricer {
    rpc: Arc<dyn SolanaRpcClient>,
    tokens: TokenManager,
    payer: Keypair,
    // ctx: &'a MarginTestContext,
    // tokens: HashMap<Pubkey, TestToken>,
    swap_registry: HashMap<Pubkey, HashMap<Pubkey, SwapPool>>,
}

pub type SwapRegistry = HashMap<Pubkey, HashMap<Pubkey, SwapPool>>;

pub async fn create_swap_pools(
    rpc: &Arc<dyn SolanaRpcClient>,
    mints: Vec<Pubkey>,
) -> Result<SwapRegistry> {
    let mut registry = SwapRegistry::new();
    for (one, two) in mints
        .into_iter()
        .combinations(2)
        .map(|c| (c[0], c[1]))
        .collect::<Vec<(Pubkey, Pubkey)>>()
    {
        let pool = SwapPool::configure(
            &rpc,
            &orca_swap_v2::id(),
            &one,
            &two,
            100_000_000,
            100_000_000,
        )
        .await?;
        registry.entry(one).or_default().insert(two, pool.clone());
        registry.entry(two).or_default().insert(one, pool);
    }

    Ok(registry)
}

impl TokenPricer {
    pub fn new(rpc: &Arc<dyn SolanaRpcClient>, swap_registry: SwapRegistry) -> Self {
        Self {
            rpc: rpc.clone(),
            tokens: TokenManager::new(rpc.clone()),
            payer: clone(rpc.payer()),
            swap_registry,
        }
    }

    pub async fn set_price(&self, mint: &Pubkey, price: f64) -> Result<()> {
        self.set_price_in_swap_pools(mint, price).await?;
        self.set_oracle_price(mint, price).await?;

        Ok(())
    }

    /// If you set the price to disagree with the oracle, you may see some odd behavior.
    /// Think carefully about how other tokens are impacted.
    pub async fn set_price_in_swap_pools(&self, mint: &Pubkey, price: f64) -> Result<()> {
        for (other_mint, pool) in self.swap_registry[mint].iter() {
            let other_price = self.get_oracle_price(mint).await?;
            let desired_relative_price = price / other_price;
            let balances = pool.balances(&self.rpc).await?;
            let (this_to_swap, other_to_swap) = set_constant_product_price(
                *balances.get(mint).unwrap(),
                *balances.get(other_mint).unwrap(),
                desired_relative_price,
            );
            let this = self
                .tokens
                .create_account_funded(&mint, &self.payer.pubkey(), this_to_swap)
                .await?;
            let other = self
                .tokens
                .create_account_funded(&other_mint, &self.payer.pubkey(), other_to_swap)
                .await?;
            pool.swap(&self.rpc, &this, &other, this_to_swap, &self.payer)
                .await?;
            pool.swap(&self.rpc, &other, &this, other_to_swap, &self.payer)
                .await?;
        }

        Ok(())
    }

    pub async fn set_oracle_price(&self, mint: &Pubkey, price: f64) -> Result<()> {
        let price = (price * 100_000_000.0) as i64;
        self.tokens
            .set_price(
                mint,
                &TokenPrice {
                    exponent: -8,
                    price,
                    confidence: 100_000_000,
                    twap: price as u64,
                },
            )
            .await?;

        Ok(())
    }

    pub async fn get_oracle_price(&self, mint: &Pubkey) -> Result<f64> {
        let px = self.tokens.get_price(mint).await?;
        let price = px.agg.price as f64 / (10u64 ^ px.expo as u64) as f64;

        Ok(price)
    }
}

/// Returns the amount of assets a and b to swap into the pool to slip
/// the assets into the desired relative price
fn set_constant_product_price(
    balance_a: u64,
    balance_b: u64,
    desired_price_of_a: f64,
) -> (u64, u64) {
    let product = balance_a as f64 * balance_b as f64;
    let new_balance_a = (product / desired_price_of_a).sqrt() as u64;
    let new_balance_b = (product * desired_price_of_a).sqrt() as u64;

    if new_balance_a > balance_a {
        (new_balance_a - balance_a, 0)
    } else {
        (0, new_balance_b - balance_b)
    }
}

struct TestToken {
    mint: Pubkey,
    last_updated: UnixTimestamp,
    latest_price: TokenPrice,
}

struct TokenPool {}

impl<'a> TestUser<'a> {
    pub async fn token_account(&mut self, mint: &Pubkey) -> Result<Pubkey> {
        let token_account = match self.mint_to_token_account.entry(*mint) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => *entry.insert(
                self.ctx
                    .tokens
                    .create_account(&mint, &self.user.owner())
                    .await?,
            ),
        };

        Ok(token_account)
    }

    pub async fn mint(&mut self, mint: &Pubkey, amount: u64) -> Result<()> {
        let token_account = self.token_account(mint).await?;
        self.ctx.tokens.mint(mint, &token_account, amount).await
    }

    pub async fn deposit(&mut self, mint: &Pubkey, amount: u64) -> Result<()> {
        let token_account = self.token_account(mint).await?;
        self.user
            .deposit(mint, &token_account, TokenChange::shift(amount))
            .await
    }

    pub async fn borrow(&mut self, mint: &Pubkey, amount: u64) -> Result<()> {
        self.ctx.tokens.refresh_to_same_price(mint).await?;
        self.user.borrow(mint, TokenChange::shift(amount)).await
    }

    pub async fn withdraw(&mut self, mint: &Pubkey, amount: u64) -> Result<()> {
        let token_account = self.token_account(mint).await?;
        self.user
            .withdraw(mint, &token_account, TokenChange::shift(amount))
            .await
    }
}
