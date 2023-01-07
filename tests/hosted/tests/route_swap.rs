#![cfg_attr(not(feature = "localnet"), allow(unused))]

use std::{collections::HashSet, sync::Arc, time::Duration};

use anyhow::Error;

use jet_margin_sdk::{
    ix_builder::{MarginPoolIxBuilder, MarginSwapRouteIxBuilder, SwapAccounts, SwapContext},
    lookup_tables::LookupTable,
    margin_integrator::PositionRefresher,
    swap::{saber_swap::SaberSwapPool, spl_swap::SplSwapPool},
    tokens::TokenPrice,
    tx_builder::TokenDepositsConfig,
};
use jet_static_program_registry::spl_token_swap_v2;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;

use hosted_tests::{
    context::MarginTestContext, margin::MarginPoolSetupInfo, margin_test_context,
    saber_swap::SaberSwapPoolConfig, spl_swap::SwapPoolConfig,
};

use jet_margin::TokenKind;
use jet_margin_pool::{MarginPoolConfig, PoolFlags, TokenChange};
use jet_simulation::create_wallet;

const ONE_USDC: u64 = 1_000_000;
const ONE_USDT: u64 = 1_000_000;
const ONE_MSOL: u64 = LAMPORTS_PER_SOL;
const ONE_TSOL: u64 = LAMPORTS_PER_SOL;

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

struct TestEnv {
    usdc: Pubkey,
    usdt: Pubkey,
    tsol: Pubkey,
    msol: Pubkey,
}

async fn setup_environment(ctx: &MarginTestContext) -> Result<TestEnv, Error> {
    let usdc = ctx.tokens.create_token(6, None, None).await?;
    let usdc_oracle = ctx.tokens.create_oracle(&usdc).await?;
    let usdt = ctx.tokens.create_token(6, None, None).await?;
    let usdt_oracle = ctx.tokens.create_oracle(&usdt).await?;
    let tsol = ctx.tokens.create_token(9, None, None).await?;
    let tsol_oracle = ctx.tokens.create_oracle(&tsol).await?;
    let msol = ctx.tokens.create_token(9, None, None).await?;
    let msol_oracle = ctx.tokens.create_oracle(&msol).await?;

    let pools = [
        MarginPoolSetupInfo {
            token: usdc,
            token_kind: TokenKind::Collateral,
            collateral_weight: 1_00,
            max_leverage: 10_00,
            config: DEFAULT_POOL_CONFIG,
            oracle: usdc_oracle,
        },
        MarginPoolSetupInfo {
            token: usdt,
            token_kind: TokenKind::Collateral,
            collateral_weight: 1_00,
            max_leverage: 10_00,
            config: DEFAULT_POOL_CONFIG,
            oracle: usdt_oracle,
        },
        MarginPoolSetupInfo {
            token: tsol,
            token_kind: TokenKind::Collateral,
            collateral_weight: 95,
            max_leverage: 4_00,
            config: DEFAULT_POOL_CONFIG,
            oracle: tsol_oracle,
        },
        MarginPoolSetupInfo {
            token: msol,
            token_kind: TokenKind::Collateral,
            collateral_weight: 90,
            max_leverage: 3_00,
            config: DEFAULT_POOL_CONFIG,
            oracle: msol_oracle,
        },
    ];

    for pool_info in pools {
        ctx.margin
            .configure_token_deposits(
                &pool_info.token,
                Some(&TokenDepositsConfig {
                    oracle: jet_margin::TokenOracle::Pyth {
                        price: pool_info.oracle.price,
                        product: pool_info.oracle.product,
                    },
                    collateral_weight: pool_info.collateral_weight,
                }),
            )
            .await?;
        ctx.margin.create_pool(&pool_info).await?;
    }

    Ok(TestEnv {
        usdc,
        tsol,
        usdt,
        msol,
    })
}

#[cfg(feature = "localnet")]
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn route_swap() -> Result<(), anyhow::Error> {
    let swap_program_id = spl_token_swap_v2::id();
    // Get the mocked runtime
    let ctx = margin_test_context!();
    let env = setup_environment(&ctx).await?;

    // Create our two user wallets, with some SOL funding to get started
    let wallet_a = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;
    let wallet_b = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;

    // Create the user context helpers, which give a simple interface for executing
    // common actions on a margin account
    let user_a = ctx.margin.user(&wallet_a, 0)?;
    let user_b = ctx.margin.user(&wallet_b, 0)?;

    // Initialize the margin accounts for each user
    user_a.create_account().await?;
    user_b.create_account().await?;

    // Create swap pools with some liquidity
    let swap_pool_spl_usdc_tsol = SplSwapPool::configure(
        &ctx.solana,
        &swap_program_id,
        &env.usdc,
        &env.tsol,
        1_000_000 * ONE_USDC,
        10_000 * ONE_TSOL,
    )
    .await?;

    // TOOD: replace with a different pool type
    let swap_pool_spl_msol_usdt = SplSwapPool::configure(
        &ctx.solana,
        &swap_program_id,
        &env.msol,
        &env.usdt,
        // Set a 106 price relative to SOL at 100
        1_060_000 * ONE_USDT,
        10_000 * ONE_MSOL,
    )
    .await?;

    // Check if the swap pool can be found
    let mut supported_mints = HashSet::new();
    supported_mints.insert(env.usdc);
    supported_mints.insert(env.usdt);
    supported_mints.insert(env.msol);
    supported_mints.insert(env.tsol);

    let swap_pools = SplSwapPool::get_pools(&ctx.rpc, &supported_mints, swap_program_id)
        .await
        .unwrap();
    assert_eq!(swap_pools.len(), 2);

    // Add Saber swap pool
    // Create a swap pool with sufficient liquidity
    let swap_pool_sbr_msol_tsol = SaberSwapPool::configure(
        &ctx.solana,
        &env.msol,
        &env.tsol,
        // Set a 1.06 rate
        10_000 * ONE_MSOL,
        10_600 * ONE_TSOL,
    )
    .await?;

    // Create some tokens for each user to deposit
    let user_a_usdc_account = ctx
        .tokens
        .create_account_funded(&env.usdc, &wallet_a.pubkey(), 1_000 * ONE_USDC)
        .await?;
    let user_a_msol_account = ctx
        .tokens
        .create_account_funded(&env.msol, &wallet_a.pubkey(), 100 * ONE_MSOL)
        .await?;
    let user_b_tsol_account = ctx
        .tokens
        .create_account_funded(&env.tsol, &wallet_b.pubkey(), 10 * ONE_TSOL)
        .await?;
    let user_b_msol_account = ctx
        .tokens
        .create_account_funded(&env.msol, &wallet_b.pubkey(), 10 * ONE_MSOL)
        .await?;

    // Set the prices for each token
    ctx.tokens
        .set_price(
            // Set price to 1 USD +- 0.01
            &env.usdc,
            &TokenPrice {
                exponent: -8,
                price: 100_000_000,
                confidence: 1_000_000,
                twap: 100_000_000,
            },
        )
        .await?;
    ctx.tokens
        .set_price(
            // Set price to 100 USD +- 1
            &env.tsol,
            &TokenPrice {
                exponent: -8,
                price: 10_000_000_000,
                confidence: 100_000_000,
                twap: 10_000_000_000,
            },
        )
        .await?;
    ctx.tokens
        .set_price(
            // Set price to 106 USD +- 1
            &env.msol,
            &TokenPrice {
                exponent: -8,
                price: 10_600_000_000,
                confidence: 100_000_000,
                twap: 10_600_000_000,
            },
        )
        .await?;

    // Deposit user funds into their margin accounts
    user_a
        .deposit(
            &env.usdc,
            &user_a_usdc_account,
            TokenChange::shift(1_000 * ONE_USDC),
        )
        .await?;
    user_a
        .deposit(
            &env.msol,
            &user_a_msol_account,
            TokenChange::shift(ONE_MSOL),
        )
        .await?;
    user_b
        .deposit(
            &env.tsol,
            &user_b_tsol_account,
            TokenChange::shift(10 * ONE_TSOL),
        )
        .await?;
    user_b
        .deposit(
            &env.msol,
            &user_b_msol_account,
            TokenChange::shift(10 * ONE_MSOL),
        )
        .await?;

    user_a.refresh_all_pool_positions().await?;
    user_b.refresh_all_pool_positions().await?;

    // Add a lookup table for the swap route
    let table = LookupTable::create_lookup_table(&ctx.rpc, None)
        .await
        .unwrap();

    // Add accounts to the lookup table
    let usdc_pool = MarginPoolIxBuilder::new(env.usdc);
    let tsol_pool = MarginPoolIxBuilder::new(env.tsol);
    let usdt_pool = MarginPoolIxBuilder::new(env.tsol);
    let accounts = &[
        // Programs
        jet_margin::id(),
        jet_margin_pool::id(),
        jet_margin_swap::id(),
        spl_token::id(),
        // Pools
        usdc_pool.token_mint,
        usdc_pool.address,
        usdc_pool.vault,
        usdc_pool.deposit_note_mint,
        usdc_pool.loan_note_mint,
        tsol_pool.token_mint,
        tsol_pool.address,
        tsol_pool.vault,
        tsol_pool.deposit_note_mint,
        tsol_pool.loan_note_mint,
        usdt_pool.token_mint,
        usdt_pool.address,
        usdt_pool.vault,
        usdt_pool.deposit_note_mint,
        usdt_pool.loan_note_mint,
        // SPL swap pools
        swap_pool_spl_usdc_tsol.pool,
        swap_pool_spl_usdc_tsol.pool_mint,
        swap_pool_spl_usdc_tsol.token_a,
        swap_pool_spl_usdc_tsol.token_b,
        swap_pool_spl_usdc_tsol.fee_account,
        swap_pool_spl_msol_usdt.pool,
        swap_pool_spl_msol_usdt.pool_mint,
        swap_pool_spl_msol_usdt.token_a,
        swap_pool_spl_msol_usdt.token_b,
        swap_pool_spl_msol_usdt.fee_account,
        swap_pool_spl_msol_usdt.program,
        // Saber swap pools
        swap_pool_sbr_msol_tsol.pool,
        swap_pool_sbr_msol_tsol.pool_authority,
        swap_pool_sbr_msol_tsol.pool_mint,
        swap_pool_sbr_msol_tsol.token_a,
        swap_pool_sbr_msol_tsol.token_b,
        swap_pool_sbr_msol_tsol.fee_a,
        swap_pool_sbr_msol_tsol.fee_b,
        swap_pool_sbr_msol_tsol.program,
    ];

    LookupTable::extend_lookup_table(&ctx.rpc, table, None, accounts)
        .await
        .unwrap();

    // Wait a bit before starting to use lookup table
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Create a swap route and execute it
    let mut swap_builder = MarginSwapRouteIxBuilder::try_new(
        SwapContext::MarginPool,
        *user_a.address(),
        env.msol,
        env.usdc,
        TokenChange::shift(ONE_MSOL),
        106 * ONE_USDC * 92 / 100,
    )?;

    // Split the route 60/40 to emulate a split even if going to the same venue
    swap_builder.add_swap_leg(&swap_pool_sbr_msol_tsol, 60)?;
    swap_builder.add_swap_leg(&swap_pool_sbr_msol_tsol, 0)?;

    swap_builder.add_swap_leg(&swap_pool_spl_usdc_tsol, 0)?;

    // Adding a disconnected swap should fail
    let result = swap_builder.add_swap_leg(&swap_pool_sbr_msol_tsol, 0);
    assert!(result.is_err());

    swap_builder.finalize().unwrap();

    // Now user A swaps their USDC for TSOL
    user_a.route_swap(&swap_builder, &[table]).await.unwrap();

    Ok(())
}

async fn single_leg_swap_margin(
    ctx: &Arc<MarginTestContext>,
    env: &TestEnv,
    pool: impl SwapAccounts,
) -> Result<(), anyhow::Error> {
    // Create our two user wallets, with some SOL funding to get started
    let wallet_a = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;
    let wallet_b = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;

    // Create the user context helpers, which give a simple interface for executing
    // common actions on a margin account
    let user_a = ctx.margin.user(&wallet_a, 0)?;
    let user_b = ctx.margin.user(&wallet_b, 0)?;

    // Initialize the margin accounts for each user
    user_a.create_account().await?;
    user_b.create_account().await?;

    let user_a_msol_account = ctx
        .tokens
        .create_account_funded(&env.msol, &wallet_a.pubkey(), 100 * ONE_MSOL)
        .await?;
    let user_b_tsol_account = ctx
        .tokens
        .create_account_funded(&env.tsol, &wallet_b.pubkey(), 10 * ONE_TSOL)
        .await?;
    let user_b_msol_account = ctx
        .tokens
        .create_account_funded(&env.msol, &wallet_b.pubkey(), 10 * ONE_MSOL)
        .await?;

    ctx.tokens
        .set_price(
            // Set price to 100 USD +- 1
            &env.tsol,
            &TokenPrice {
                exponent: -8,
                price: 10_000_000_000,
                confidence: 100_000_000,
                twap: 10_000_000_000,
            },
        )
        .await?;
    ctx.tokens
        .set_price(
            // Set price to 106 USD +- 1
            &env.msol,
            &TokenPrice {
                exponent: -8,
                price: 10_600_000_000,
                confidence: 100_000_000,
                twap: 10_600_000_000,
            },
        )
        .await?;

    // Deposit user funds into their margin accounts
    user_a
        .deposit(
            &env.msol,
            &user_a_msol_account,
            TokenChange::shift(ONE_MSOL),
        )
        .await?;
    user_b
        .deposit(
            &env.tsol,
            &user_b_tsol_account,
            TokenChange::shift(10 * ONE_TSOL),
        )
        .await?;
    user_b
        .deposit(
            &env.msol,
            &user_b_msol_account,
            TokenChange::shift(10 * ONE_MSOL),
        )
        .await?;

    user_a.refresh_all_pool_positions().await?;
    user_b.refresh_all_pool_positions().await?;

    // Create a swap route and execute it
    let mut swap_builder = MarginSwapRouteIxBuilder::try_new(
        SwapContext::MarginPool,
        *user_a.address(),
        env.msol,
        env.tsol,
        TokenChange::shift(ONE_MSOL),
        1, // Get at least 1 token back
    )?;

    // Can't finalize if there are no routes
    assert!(swap_builder.finalize().is_err());

    swap_builder.add_swap_leg(&pool, 60)?;
    swap_builder.add_swap_leg(&pool, 0)?;

    // // Adding a disconnected swap should fail
    // let result = swap_builder.add_swap_leg(&pool, 90);
    // assert!(result.is_err());

    swap_builder.finalize()?;

    // Can't finalize if already finalized
    assert!(swap_builder.finalize().is_err());

    user_a.route_swap(&swap_builder, &[]).await?;

    Ok(())
}

async fn single_leg_swap(
    ctx: &Arc<MarginTestContext>,
    env: &TestEnv,
    pool: impl SwapAccounts,
) -> Result<(), anyhow::Error> {
    let wallet_a = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;
    let user_a = ctx.margin.user(&wallet_a, 0)?;
    user_a.create_account().await?;

    ctx.tokens
        .set_price(
            // Set price to 100 USD +- 1
            &env.tsol,
            &TokenPrice {
                exponent: -8,
                price: 10_000_000_000,
                confidence: 100_000_000,
                twap: 10_000_000_000,
            },
        )
        .await?;
    ctx.tokens
        .set_price(
            // Set price to 106 USD +- 1
            &env.msol,
            &TokenPrice {
                exponent: -8,
                price: 10_600_000_000,
                confidence: 100_000_000,
                twap: 10_600_000_000,
            },
        )
        .await?;

    // Create a deposit accounts for the user
    let user_a_msol = user_a.create_deposit_position(&env.msol).await?;
    let user_a_tsol = user_a.create_deposit_position(&env.tsol).await?;

    // Fund one margin position
    ctx.tokens
        .mint(&env.msol, &user_a_msol, 10 * ONE_MSOL)
        .await?;

    user_a.tx.refresh_positions().await?;

    // Create a swap route and execute it
    let mut swap_builder = MarginSwapRouteIxBuilder::try_new(
        SwapContext::MarginPositions,
        *user_a.address(),
        env.msol,
        env.tsol,
        TokenChange::shift(ONE_MSOL),
        1, // Get at least 1 token back
    )?;
    swap_builder.add_swap_leg(&pool, 60)?;
    swap_builder.add_swap_leg(&pool, 0)?;
    swap_builder.finalize().unwrap();

    user_a.route_swap(&swap_builder, &[]).await?;

    // There should be tokens in the user's destination account
    let tsol_balance = ctx.tokens.get_balance(&user_a_tsol).await?;
    assert!(tsol_balance > ONE_TSOL / 2);

    Ok(())
}

// The tests create duplicate accounts, causing failures in localnet.
// They are however useful for coverage and testing logic, so we run them on the sim.
#[cfg(not(feature = "localnet"))]
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn route_spl_swap() -> Result<(), anyhow::Error> {
    let ctx = margin_test_context!();
    let env = setup_environment(&ctx).await?;

    let swap_program_id = spl_token_swap_v2::id();

    // TOOD: replace with a different pool type
    let pool = SplSwapPool::configure(
        &ctx.solana,
        &swap_program_id,
        &env.msol,
        &env.tsol,
        // Set a 106 price relative to SOL at 100
        1_060_000 * ONE_MSOL,
        1_000_000 * ONE_TSOL,
    )
    .await?;

    // Check that we can get the pool
    // Check if the swap pool can be found
    let mut supported_mints = HashSet::new();
    supported_mints.insert(env.usdc);
    supported_mints.insert(env.usdt);
    supported_mints.insert(env.msol);
    supported_mints.insert(env.tsol);

    let swap_pools = SplSwapPool::get_pools(&ctx.rpc, &supported_mints, swap_program_id)
        .await
        .unwrap();
    assert_eq!(swap_pools.len(), 1);

    single_leg_swap_margin(&ctx, &env, pool).await?;
    single_leg_swap(&ctx, &env, pool).await?;
    Ok(())
}

// The tests create duplicate accounts, causing failures in localnet.
// They are however useful for coverage and testing logic, so we run them on the sim.
#[cfg(not(feature = "localnet"))]
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn route_saber_swap() -> Result<(), anyhow::Error> {
    let ctx = margin_test_context!();
    let env = setup_environment(&ctx).await?;

    // Add Saber swap pool
    // Create a swap pool with sufficient liquidity
    let pool = SaberSwapPool::configure(
        &ctx.solana,
        &env.msol,
        &env.tsol,
        // Set a 1.06 rate
        10_000 * ONE_MSOL,
        10_600 * ONE_TSOL,
    )
    .await?;

    // Check that we can get the pool
    // Check if the swap pool can be found
    let mut supported_mints = HashSet::new();
    supported_mints.insert(env.usdc);
    supported_mints.insert(env.usdt);
    supported_mints.insert(env.msol);
    supported_mints.insert(env.tsol);

    let swap_pools = SaberSwapPool::get_pools(&ctx.rpc, &supported_mints)
        .await
        .unwrap();
    assert_eq!(swap_pools.len(), 1);

    single_leg_swap_margin(&ctx, &env, pool).await?;
    single_leg_swap(&ctx, &env, pool).await?;

    Ok(())
}
