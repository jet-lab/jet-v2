use std::collections::hash_map::Entry;
use std::collections::HashMap;

use anyhow::Result;
use jet_margin_sdk::cat;
use jet_margin_sdk::solana::transaction::{SendTransactionBuilder, TransactionBuilder};
use jet_margin_sdk::util::asynchronous::{AndAsync, MapAsync};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature, Signer};

use jet_margin_pool::TokenChange;
use jet_static_program_registry::orca_swap_v2;

use crate::context::MarginTestContext;
use crate::margin::MarginUser;
use crate::swap::SwapRegistry;
use crate::tokens::TokenManager;

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

    pub async fn borrow(&self, mint: &Pubkey, amount: u64) -> Result<Vec<Signature>> {
        let mut txs = vec![self.ctx.tokens.refresh_to_same_price_tx(mint).await?];
        txs.extend(self.user.tx.refresh_all_pool_positions().await?);
        txs.push(
            self.user
                .tx
                .borrow(mint, TokenChange::shift(amount))
                .await?,
        );

        self.ctx.rpc.send_and_confirm_condensed(txs).await
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
        change: TokenChange,
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
                change,
                1, // at least 1 token back
            )
            .await
    }

    pub async fn liquidate_begin(&self, refresh_positions: bool) -> Result<()> {
        let mut txs = if refresh_positions {
            self.refresh_position_oracles_txs().await?
        } else {
            vec![]
        };
        txs.push(self.user.liquidate_begin_tx(refresh_positions).await?);
        self.ctx.rpc.send_and_confirm_condensed(txs).await?;

        Ok(())
    }

    pub async fn verify_healthy(&self) -> Result<()> {
        self.user.verify_healthy().await
    }

    pub async fn liquidate_end(&self, liquidator: Option<Pubkey>) -> Result<()> {
        self.user.liquidate_end(liquidator).await
    }

    pub async fn refresh_position_oracles_txs(&self) -> Result<Vec<TransactionBuilder>> {
        let tokens = TokenManager::new(self.ctx.rpc.clone());
        self.user
            .positions()
            .await?
            .iter()
            .map_async(|position| tokens.refresh_to_same_price_tx(&position.token))
            .await
    }

    pub async fn refresh_positions_with_oracles_txs(&self) -> Result<Vec<TransactionBuilder>> {
        let tokens = TokenManager::new(self.ctx.rpc.clone());
        Ok(self
            .user
            .tx
            .refresh_all_pool_positions_underlying_to_tx()
            .await?
            .into_iter()
            .map_async(|(ul, pos)| pos.and_result(tokens.refresh_to_same_price_tx2(ul)))
            .await?
            .into_iter()
            .map(|(tx2, tx1)| cat![tx1, tx2])
            .collect())
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

    pub fn for_user(&self, user: &MarginUser) -> Result<TestUser<'a>> {
        let liquidation = self
            .ctx
            .margin
            .liquidator(&self.wallet, user.owner(), user.seed())?;

        Ok(TestUser {
            ctx: self.ctx,
            user: liquidation,
            mint_to_token_account: HashMap::new(),
        })
    }

    pub async fn begin(&self, user: &MarginUser, refresh_positions: bool) -> Result<TestUser<'a>> {
        let test_liquidation = self.for_user(user)?;
        test_liquidation
            .user
            .liquidate_begin(refresh_positions)
            .await?;

        Ok(test_liquidation)
    }

    pub async fn liquidate(
        &self,
        user: &MarginUser,
        swaps: &SwapRegistry,
        collateral: &Pubkey,
        loan: &Pubkey,
        change: TokenChange,
        repay: u64,
    ) -> Result<()> {
        let liq = self.begin(user, true).await?;
        liq.swap(swaps, collateral, loan, change).await?;
        liq.margin_repay(loan, repay).await?;
        liq.liquidate_end(Some(self.wallet.pubkey())).await
    }
}
