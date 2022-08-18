use std::collections::hash_map::Entry;
use std::collections::HashMap;

use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use jet_margin_pool::{Amount, TokenChange};
use jet_static_program_registry::orca_swap_v2;

use crate::context::MarginTestContext;
use crate::margin::MarginUser;
use crate::swap::SwapRegistry;

pub const ONE: u64 = 1_000_000_000;

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
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => *entry.insert(
                self.ctx
                    .tokens
                    .create_account(mint, self.user.owner())
                    .await?,
            ),
        };

        Ok(token_account)
    }

    pub async fn ephemeral_token_account(&self, mint: &Pubkey, amount: u64) -> Result<Pubkey> {
        self.ctx
            .tokens
            .create_account_funded(mint, self.user.owner(), amount)
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
            .create_account(src, self.user.address())
            .await?;
        let transit_dst = self
            .ctx
            .tokens
            .create_account(dst, self.user.address())
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
    pub async fn new(ctx: &'a MarginTestContext) -> Result<TestLiquidator<'a>> {
        Ok(TestLiquidator {
            ctx,
            wallet: ctx.create_liquidator(100).await?,
        })
    }

    pub async fn begin(&self, user: &MarginUser) -> Result<TestUser<'a>> {
        let liquidation = self
            .ctx
            .margin
            .liquidator(&self.wallet, user.owner(), user.seed())
            .await?;
        liquidation.liquidate_begin(true).await?;

        Ok(TestUser {
            ctx: self.ctx,
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
        liq.swap(swaps, collateral, loan, sell).await?;
        liq.margin_repay(loan, repay).await?;
        liq.liquidate_end(self.wallet.pubkey()).await
    }
}
