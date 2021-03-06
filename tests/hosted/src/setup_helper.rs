use anyhow::{Error, Result};

use jet_margin_sdk::tokens::TokenPrice;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use jet_margin_pool::{MarginPoolConfig, PoolFlags, TokenChange};
use jet_metadata::TokenKind;
use jet_simulation::create_wallet;

use crate::{
    context::{test_context, MarginTestContext},
    margin::{MarginPoolSetupInfo, MarginUser},
};

const DEFAULT_POOL_CONFIG: MarginPoolConfig = MarginPoolConfig {
    borrow_rate_0: 10,
    borrow_rate_1: 20,
    borrow_rate_2: 30,
    borrow_rate_3: 40,
    utilization_rate_1: 10,
    utilization_rate_2: 20,
    management_fee_rate: 10,
    flags: PoolFlags::ALLOW_LENDING.bits(),
    reserved: 0,
};

pub struct TestEnvironment {
    pub mints: Vec<Pubkey>,
    pub users: Vec<TestUser>,
    pub liquidator: Keypair,
}

pub struct TestUser {
    pub user: MarginUser,
    pub liquidator: MarginUser,
}

pub async fn setup_token(
    ctx: &MarginTestContext,
    decimals: u8,
    collateral_weight: u16,
    leverage_max: u16,
    price: i64,
) -> Result<Pubkey, Error> {
    let token = ctx.tokens.create_token(decimals, None, None).await?;
    let token_oracle = ctx.tokens.create_oracle(&token).await?;

    ctx.margin
        .create_pool(&MarginPoolSetupInfo {
            token,
            collateral_weight,
            max_leverage: leverage_max,
            token_kind: TokenKind::Collateral,
            config: DEFAULT_POOL_CONFIG,
            oracle: token_oracle,
        })
        .await?;

    // set price to $1
    ctx.tokens
        .set_price(
            &token,
            &TokenPrice {
                exponent: -8,
                price: price * 100_000_000_i64,
                confidence: 1_000_000,
                twap: 100_000_000,
            },
        )
        .await?;

    Ok(token)
}
/// (token_mint, balance in wallet, balance in pools)
pub async fn setup_user(
    ctx: &MarginTestContext,
    liquidator_wallet: &Keypair,
    tokens: Vec<(Pubkey, u64, u64)>,
) -> Result<TestUser> {
    // Create our two user wallets, with some SOL funding to get started
    let wallet = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;

    // Create the user context helpers, which give a simple interface for executing
    // common actions on a margin account
    let user = ctx.margin.user(&wallet, 0).await?;
    let user_liq = ctx
        .margin
        .liquidator(liquidator_wallet, &wallet.pubkey(), 0)
        .await?;

    // Initialize the margin accounts for each user
    user.create_account().await?;

    for (mint, in_wallet, in_pool) in tokens {
        // Create some tokens for each user to deposit
        let token_account = ctx
            .tokens
            .create_account_funded(&mint, &wallet.pubkey(), in_wallet + in_pool)
            .await?;

        if in_pool > 0 {
            // Deposit user funds into their margin accounts
            user.deposit(&mint, &token_account, TokenChange::shift(in_pool))
                .await?;
        }

        // Verify user tokens have been deposited
        assert_eq!(in_wallet, ctx.tokens.get_balance(&token_account).await?);
    }

    user.refresh_all_pool_positions().await?;

    Ok(TestUser {
        user,
        liquidator: user_liq,
    })
}

/// Environment where no user has a balance
pub async fn build_environment_with_no_balances(
    number_of_mints: u64,
    number_of_users: u64,
) -> Result<(&'static MarginTestContext, TestEnvironment), Error> {
    let ctx = test_context().await;
    let liquidator = ctx.create_liquidator(100).await?;
    let mut mints: Vec<Pubkey> = Vec::new();
    for _ in 0..number_of_mints {
        let mint = setup_token(ctx, 6, 1_00, 10_00, 1).await?;
        mints.push(mint);
    }
    let mut users: Vec<TestUser> = Vec::new();
    for _ in 0..number_of_users {
        users.push(setup_user(ctx, &liquidator, vec![]).await?);
    }

    Ok((
        ctx,
        TestEnvironment {
            mints,
            users,
            liquidator,
        },
    ))
}

/// Environment where every user has 100 of every token in their wallet but no pool deposits
pub async fn build_environment_with_raw_token_balances(
    number_of_mints: u64,
    number_of_users: u64,
) -> Result<(&'static MarginTestContext, TestEnvironment), Error> {
    let ctx = test_context().await;
    let liquidator = ctx.create_liquidator(100).await?;
    let mut mints: Vec<Pubkey> = Vec::new();
    let mut wallets: Vec<(Pubkey, u64, u64)> = Vec::new();
    for _ in 0..number_of_mints {
        let mint = setup_token(ctx, 6, 1_00, 10_00, 1).await?;
        mints.push(mint);
        wallets.push((mint, 100, 0));
    }
    let mut users: Vec<TestUser> = Vec::new();
    for _ in 0..number_of_users {
        users.push(setup_user(ctx, &liquidator, wallets.clone()).await?);
    }

    Ok((
        ctx,
        TestEnvironment {
            mints,
            users,
            liquidator,
        },
    ))
}
