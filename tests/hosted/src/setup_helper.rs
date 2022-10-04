use std::collections::HashMap;

use anyhow::{Error, Result};

use jet_margin::TokenKind;
use jet_margin_sdk::solana::transaction::SendTransactionBuilder;
use jet_margin_sdk::tokens::TokenPrice;
use jet_margin_sdk::util::asynchronous::MapAsync;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;

use jet_margin_pool::{MarginPoolConfig, PoolFlags, TokenChange};
use jet_simulation::{create_wallet, generate_keypair};
use tokio::try_join;

use crate::pricing::TokenPricer;
use crate::swap::{create_swap_pools, SwapRegistry};
use crate::test_user::{TestLiquidator, TestUser};
use crate::{
    context::{test_context, MarginTestContext},
    margin::MarginPoolSetupInfo,
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

pub struct TestEnvironment<'a> {
    pub mints: Vec<Pubkey>,
    pub users: Vec<TestUser<'a>>,
}

pub async fn setup_token(
    ctx: &MarginTestContext,
    decimals: u8,
    collateral_weight: u16,
    leverage_max: u16,
    price: f64,
) -> Result<Pubkey, Error> {
    let token_keypair = generate_keypair();
    let token = token_keypair.pubkey();
    let (token, token_oracle) = try_join!(
        ctx.tokens
            .create_token_from(token_keypair, decimals, None, None),
        ctx.tokens.create_oracle(&token)
    )?;
    let setup = MarginPoolSetupInfo {
        token,
        collateral_weight,
        max_leverage: leverage_max,
        token_kind: TokenKind::Collateral,
        config: DEFAULT_POOL_CONFIG,
        oracle: token_oracle,
    };
    let price = TokenPrice {
        exponent: -8,
        price: (price * 100_000_000.0) as i64,
        confidence: 1_000_000,
        twap: 100_000_000,
    };
    try_join!(
        ctx.margin.create_pool(&setup),
        ctx.tokens.set_price(&token, &price)
    )?;

    Ok(token)
}

pub async fn users<const N: usize>(ctx: &MarginTestContext) -> Result<[TestUser; N]> {
    Ok(create_users(ctx, N).await?.try_into().unwrap())
}

pub async fn liquidators<const N: usize>(ctx: &MarginTestContext) -> Result<[TestLiquidator; N]> {
    Ok((0..N)
        .map_async(|_| TestLiquidator::new(ctx))
        .await?
        .try_into()
        .unwrap())
}

pub async fn tokens<const N: usize>(
    ctx: &MarginTestContext,
) -> Result<([Pubkey; N], SwapRegistry, TokenPricer)> {
    let (tokens, swaps, pricer) = create_tokens(ctx, N).await?;

    Ok((tokens.try_into().unwrap(), swaps, pricer))
}

pub async fn create_users(ctx: &MarginTestContext, n: usize) -> Result<Vec<TestUser>> {
    (0..n).map_async(|_| setup_user(ctx, vec![])).await
}

pub async fn create_tokens(
    ctx: &MarginTestContext,
    n: usize,
) -> Result<(Vec<Pubkey>, SwapRegistry, TokenPricer)> {
    let tokens: Vec<Pubkey> = (0..n)
        .map_async(|_| setup_token(ctx, 9, 1_00, 4_00, 1.0))
        .await?;
    let owner = ctx.rpc.payer().pubkey();
    let (swaps, vaults) = try_join!(
        create_swap_pools(&ctx.rpc, &tokens),
        tokens
            .iter()
            .map_async(|mint| { ctx.tokens.create_account_funded(mint, &owner, u64::MAX / 4) })
    )?;
    let vaults = tokens
        .clone()
        .into_iter()
        .zip(vaults)
        .collect::<HashMap<Pubkey, Pubkey>>();
    let pricer = TokenPricer::new(&ctx.rpc, vaults, &swaps);

    Ok((tokens, swaps, pricer))
}

/// (token_mint, balance in wallet, balance in pools)
pub async fn setup_user(
    ctx: &MarginTestContext,
    tokens: Vec<(Pubkey, u64, u64)>,
) -> Result<TestUser> {
    // Create our two user wallets, with some SOL funding to get started
    let wallet = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;

    // Create the user context helpers, which give a simple interface for executing
    // common actions on a margin account
    let user = ctx.margin.user(&wallet, 0)?;

    // Initialize the margin accounts for each user
    user.create_account().await?;

    let mut mint_to_token_account = HashMap::new();
    for (mint, in_wallet, in_pool) in tokens {
        // Create some tokens for each user to deposit
        let token_account = ctx
            .tokens
            .create_account_funded(&mint, &wallet.pubkey(), in_wallet + in_pool)
            .await?;
        mint_to_token_account.insert(mint, token_account);

        if in_pool > 0 {
            // Deposit user funds into their margin accounts
            user.deposit(&mint, &token_account, TokenChange::shift(in_pool))
                .await?;
        }

        // Verify user tokens have been deposited
        assert_eq!(in_wallet, ctx.tokens.get_balance(&token_account).await?);
    }

    let test_user = TestUser {
        ctx,
        user,
        mint_to_token_account,
    };

    // todo try to remove this and let tests do it instead only when necessary
    ctx.rpc
        .send_and_confirm_condensed(test_user.refresh_positions_with_oracles_txs().await?)
        .await?;

    Ok(test_user)
}

/// Environment where no user has a balance
pub async fn build_environment_with_no_balances<'a>(
    number_of_mints: u64,
    number_of_users: u64,
) -> Result<(&'static MarginTestContext, TestEnvironment<'a>), Error> {
    let ctx = test_context().await;
    let mut mints: Vec<Pubkey> = Vec::new();
    for _ in 0..number_of_mints {
        let mint = setup_token(ctx, 6, 1_00, 10_00, 1.0).await?;
        mints.push(mint);
    }
    let mut users: Vec<TestUser> = Vec::new();
    for _ in 0..number_of_users {
        users.push(setup_user(ctx, vec![]).await?);
    }

    Ok((
        ctx,
        TestEnvironment {
            mints,
            users,
            // liquidator,
        },
    ))
}

/// Environment where every user has 100 of every token in their wallet but no pool deposits
pub async fn build_environment_with_raw_token_balances<'a>(
    number_of_mints: u64,
    number_of_users: u64,
) -> Result<(&'static MarginTestContext, TestEnvironment<'a>), Error> {
    let ctx = test_context().await;
    // let liquidator = ctx.create_liquidator(100).await?;
    let mut mints: Vec<Pubkey> = Vec::new();
    let mut wallets: Vec<(Pubkey, u64, u64)> = Vec::new();
    for _ in 0..number_of_mints {
        let mint = setup_token(ctx, 6, 1_00, 10_00, 1.0).await?;
        mints.push(mint);
        wallets.push((mint, 100, 0));
    }
    let mut users: Vec<TestUser> = Vec::new();
    for _ in 0..number_of_users {
        users.push(setup_user(ctx, wallets.clone()).await?);
    }

    Ok((
        ctx,
        TestEnvironment {
            mints,
            users,
            // liquidator,
        },
    ))
}
