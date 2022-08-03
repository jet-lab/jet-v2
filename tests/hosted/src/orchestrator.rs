use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;

use itertools::Itertools;
use jet_margin_sdk::swap::SwapPool;
use jet_margin_sdk::tokens::TokenPrice;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use jet_static_program_registry::orca_swap_v2;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use jet_margin_pool::{Amount, TokenChange};
use tokio::try_join;

use crate::clone;
use crate::context::MarginTestContext;
use crate::margin::MarginUser;
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
    mints: &[Pubkey],
) -> Result<SwapRegistry> {
    let mut registry = SwapRegistry::new();
    for (one, two) in mints
        .into_iter()
        .combinations(2)
        .map(|c| (c[0].clone(), c[1].clone()))
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
    pub fn new(rpc: &Arc<dyn SolanaRpcClient>, swap_registry: &SwapRegistry) -> Self {
        Self {
            rpc: rpc.clone(),
            tokens: TokenManager::new(rpc.clone()),
            payer: clone(rpc.payer()),
            swap_registry: swap_registry.clone(),
        }
    }

    pub async fn set_price(&self, mint: &Pubkey, price: f64) -> Result<()> {
        try_join!(
            self.set_oracle_price(mint, price),
            self.set_price_in_swap_pools(mint, price)
        )?;

        Ok(())
    }

    /// If you set the price to disagree with the oracle, you may see some odd behavior.
    /// Think carefully about how other tokens are impacted.
    pub async fn set_price_in_swap_pools(&self, mint: &Pubkey, price: f64) -> Result<()> {
        for (other_mint, pool) in self.swap_registry[mint].iter() {
            let other_price = self.get_oracle_price(other_mint).await?;
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
                    confidence: 0,
                    twap: price as u64,
                },
            )
            .await?;

        Ok(())
    }

    pub async fn get_oracle_price(&self, mint: &Pubkey) -> Result<f64> {
        let px = self.tokens.get_price(mint).await?;
        let price = px.agg.price as f64 * (10f64.powf(px.expo.into()));

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

/// A MarginUser that takes some extra liberties
#[derive(Clone)]
pub struct TestUser<'a> {
    pub ctx: &'a MarginTestContext,
    pub user: MarginUser,
    pub mint_to_token_account: HashMap<Pubkey, Pubkey>,
}

impl<'a> std::fmt::Debug for TestUser<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestUser")
            .field("user", &self.user.address())
            // .field("liquidator", &self.liquidator.address())
            .field("mint_to_token_account", &self.mint_to_token_account)
            .finish()
    }
}

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

    pub async fn ephemeral_token_account(&self, mint: &Pubkey, amount: u64) -> Result<Pubkey> {
        self.ctx
            .tokens
            .create_account_funded(&mint, &self.user.owner(), amount)
            .await
    }

    pub async fn mint(&mut self, mint: &Pubkey, amount: u64) -> Result<()> {
        let token_account = self.token_account(mint).await?;
        self.ctx.tokens.mint(mint, &token_account, amount).await
    }

    pub async fn deposit(&self, mint: &Pubkey, amount: u64) -> Result<()> {
        let token_account = self.ephemeral_token_account(mint, amount).await?;
        self.user
            .deposit(mint, &token_account, TokenChange::shift(amount))
            .await?;
        self.ctx.tokens.refresh_to_same_price(mint).await
    }

    pub async fn deposit_from_wallet(&mut self, mint: &Pubkey, amount: u64) -> Result<()> {
        let token_account = self.token_account(mint).await?;
        self.user
            .deposit(mint, &token_account, TokenChange::shift(amount))
            .await
    }

    pub async fn borrow(&self, mint: &Pubkey, amount: u64) -> Result<()> {
        self.ctx.tokens.refresh_to_same_price(mint).await?;
        self.user.refresh_all_pool_positions().await?;
        self.user.borrow(mint, TokenChange::shift(amount)).await
    }

    pub async fn borrow_to_wallet(&self, mint: &Pubkey, amount: u64) -> Result<()> {
        self.borrow(mint, amount).await?;
        self.withdraw(mint, amount).await
    }

    pub async fn margin_repay(&self, mint: &Pubkey, amount: u64) -> Result<()> {
        self.user
            .margin_repay(mint, TokenChange::shift(amount))
            .await
    }

    pub async fn withdraw(&self, mint: &Pubkey, amount: u64) -> Result<()> {
        let token_account = self.ephemeral_token_account(mint, 0).await?;
        self.user.refresh_all_pool_positions().await?;
        self.user
            .withdraw(mint, &token_account, TokenChange::shift(amount))
            .await
    }

    pub async fn withdraw_to_wallet(&mut self, mint: &Pubkey, amount: u64) -> Result<()> {
        let token_account = self.token_account(mint).await?;
        self.user.refresh_all_pool_positions().await?;
        self.user
            .withdraw(mint, &token_account, TokenChange::shift(amount))
            .await
    }

    pub async fn swap(
        &self,
        swaps: &SwapRegistry,
        src: &Pubkey,
        dst: &Pubkey,
        amount: u64,
    ) -> Result<()> {
        let pool = swaps.get(src).unwrap().get(dst).unwrap();
        let transit_src = self
            .ctx
            .tokens
            .create_account(&src, &self.user.address())
            .await?;
        let transit_dst = self
            .ctx
            .tokens
            .create_account(&dst, &self.user.address())
            .await?;
        self.user
            .swap(
                &orca_swap_v2::id(),
                src,
                dst,
                &transit_src,
                &transit_dst,
                pool,
                Amount::tokens(amount),
                Amount::tokens(0),
            )
            .await
    }

    pub async fn liquidate_end(&self, liquidator: Pubkey) -> Result<()> {
        self.user.liquidate_end(Some(liquidator)).await
    }
}

#[derive(Debug)]
pub struct TestLiquidator<'a> {
    pub ctx: &'a MarginTestContext,
    pub wallet: Keypair,
}

impl<'a> TestLiquidator<'a> {
    pub async fn new(ctx: &'a MarginTestContext) -> Result<TestLiquidator> {
        Ok(TestLiquidator {
            ctx,
            wallet: ctx.create_liquidator(100).await?,
        })
    }

    pub async fn begin(&self, user: &MarginUser) -> Result<TestUser<'a>> {
        let liquidation = self
            .ctx
            .margin
            .liquidator(&self.wallet, &user.owner(), user.seed())
            .await?;
        liquidation.liquidate_begin(true).await?;

        Ok(TestUser {
            ctx: self.ctx.clone(),
            user: liquidation,
            mint_to_token_account: HashMap::new(),
        })
    }

    pub async fn liquidate(
        &self,
        user: &MarginUser,
        swaps: &SwapRegistry,
        collateral: &Pubkey,
        sell: u64,
        loan: &Pubkey,
        repay: u64,
    ) -> Result<()> {
        let liq = self.begin(user).await?;
        liq.swap(&swaps, collateral, loan, sell).await?;
        liq.margin_repay(loan, repay).await?;
        liq.liquidate_end(self.wallet.pubkey()).await
    }
}
