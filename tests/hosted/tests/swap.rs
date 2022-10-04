use std::collections::HashSet;

use anyhow::Error;

use jet_margin_sdk::{spl_swap::SplSwapPool, tokens::TokenPrice, tx_builder::TokenDepositsConfig};
use jet_static_program_registry::{orca_swap_v1, orca_swap_v2, spl_token_swap_v2};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;

use hosted_tests::{
    context::{test_context, MarginTestContext},
    margin::MarginPoolSetupInfo,
    swap::SwapPoolConfig,
};

use jet_margin::TokenKind;
use jet_margin_pool::{MarginPoolConfig, PoolFlags, TokenChange};
use jet_simulation::{assert_custom_program_error, create_wallet};

const ONE_USDC: u64 = 1_000_000;
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

/// Test token swaps for the official SPL token swap
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn spl_swap_v2() -> Result<(), anyhow::Error> {
    let result = swap_test_impl(spl_token_swap_v2::id()).await;
    println!("{:#?}", &result);

    result
}

/// Test token swaps for orca v1
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn orca_swap_v1() -> Result<(), anyhow::Error> {
    swap_test_impl(orca_swap_v1::id()).await
}

/// Test token swaps for orca v2
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn orca_swap_v2() -> Result<(), anyhow::Error> {
    swap_test_impl(orca_swap_v2::id()).await
}

struct TestEnv {
    usdc: Pubkey,
    tsol: Pubkey,
}

async fn setup_environment(ctx: &MarginTestContext) -> Result<TestEnv, Error> {
    let usdc = ctx.tokens.create_token(6, None, None).await?;
    let usdc_oracle = ctx.tokens.create_oracle(&usdc).await?;
    let tsol = ctx.tokens.create_token(9, None, None).await?;
    let tsol_oracle = ctx.tokens.create_oracle(&tsol).await?;

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
            token: tsol,
            token_kind: TokenKind::Collateral,
            collateral_weight: 95,
            max_leverage: 4_00,
            config: DEFAULT_POOL_CONFIG,
            oracle: tsol_oracle,
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
                    token_kind: pool_info.token_kind,
                    collateral_weight: pool_info.collateral_weight,
                }),
            )
            .await?;
        ctx.margin.create_pool(&pool_info).await?;
    }

    Ok(TestEnv { usdc, tsol })
}

async fn swap_test_impl(swap_program_id: Pubkey) -> Result<(), anyhow::Error> {
    // Get the mocked runtime
    let ctx = test_context().await;
    let env = setup_environment(ctx).await?;

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

    // Create a swap pool with sufficient liquidity
    let swap_pool = SplSwapPool::configure(
        &ctx.rpc,
        &swap_program_id,
        &env.usdc,
        &env.tsol,
        1_000_000 * ONE_USDC,
        10_000 * ONE_TSOL,
    )
    .await?;

    // Check if the swap pool can be found
    let mut supported_mints = HashSet::new();
    supported_mints.insert(env.usdc);
    supported_mints.insert(env.tsol);

    let swap_pools = SplSwapPool::get_pools(&ctx.rpc, &supported_mints, swap_program_id)
        .await
        .unwrap();
    assert_eq!(swap_pools.len(), 1);

    for pool in swap_pools.values() {
        assert_eq!(swap_pool.pool, pool.pool);
    }

    let user_a_usdc_transit = user_a.create_deposit_position(&env.usdc).await?;
    let user_a_tsol_transit = user_a.create_deposit_position(&env.tsol).await?;

    // Create some tokens for each user to deposit
    let user_a_usdc_account = ctx
        .tokens
        .create_account_funded(&env.usdc, &wallet_a.pubkey(), 1_000 * ONE_USDC)
        .await?;
    let user_a_tsol_account = ctx
        .tokens
        .create_account_funded(&env.tsol, &wallet_a.pubkey(), 100 * ONE_TSOL)
        .await?;
    let user_b_tsol_account = ctx
        .tokens
        .create_account_funded(&env.tsol, &wallet_b.pubkey(), 10 * ONE_TSOL)
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
            &env.tsol,
            &user_a_tsol_account,
            TokenChange::shift(10 * ONE_TSOL),
        )
        .await?;
    user_b
        .deposit(
            &env.tsol,
            &user_b_tsol_account,
            TokenChange::shift(10 * ONE_TSOL),
        )
        .await?;

    // Verify user tokens have been deposited
    assert_eq!(0, ctx.tokens.get_balance(&user_a_usdc_account).await?);
    assert_eq!(
        90 * ONE_TSOL,
        ctx.tokens.get_balance(&user_a_tsol_account).await?
    );
    assert_eq!(0, ctx.tokens.get_balance(&user_b_tsol_account).await?);

    user_a.refresh_all_pool_positions().await?;
    user_b.refresh_all_pool_positions().await?;

    // Now user A swaps their USDC for TSOL
    user_a
        .swap(
            &swap_program_id,
            &env.usdc,
            &env.tsol,
            &user_a_usdc_transit,
            &user_a_tsol_transit,
            &swap_pool,
            TokenChange::shift(100 * ONE_USDC),
            // we want a minimum of 0.9 SOL for 100 USDC
            ONE_TSOL / 10 * 9,
        )
        .await?;

    // Verify that swap has taken place in the pool
    assert_eq!(
        // There was 1 million USDC as a start
        1_000_100 * ONE_USDC,
        ctx.tokens.get_balance(&swap_pool.token_a).await?
    );

    assert!(
        // Pool balance less almost 1 SOL
        10_000 * ONE_TSOL - 900_000_000 >= ctx.tokens.get_balance(&swap_pool.token_b).await?
    );

    // Trying to withdraw by setting balance > actual should return an error
    let result = user_a
        .swap(
            &swap_program_id,
            &env.usdc,
            &env.tsol,
            &user_a_usdc_transit,
            &user_a_tsol_transit,
            &swap_pool,
            TokenChange::set(2_000 * ONE_USDC),
            // Value doesn't matter
            ONE_TSOL,
        )
        .await;
    assert_custom_program_error(jet_margin_pool::ErrorCode::InvalidSetTo, result);

    // Trying to swap 0 tokens should return an error
    let result = user_a
        .swap(
            &swap_program_id,
            &env.usdc,
            &env.tsol,
            &user_a_usdc_transit,
            &user_a_tsol_transit,
            &swap_pool,
            TokenChange::set(900 * ONE_USDC),
            // Value doesn't matter
            ONE_TSOL,
        )
        .await;
    assert_custom_program_error(jet_margin_swap::ErrorCode::NoSwapTokensWithdrawn, result);

    // Swap more, setting the change to a `set(x)`
    user_a
        .swap(
            &swap_program_id,
            &env.usdc,
            &env.tsol,
            &user_a_usdc_transit,
            &user_a_tsol_transit,
            &swap_pool,
            TokenChange::set(799 * ONE_USDC),
            // we want a minimum of 0.9 SOL for 101 USDC (1000 - 100 - 799)
            ONE_TSOL / 10 * 9,
        )
        .await?;

    // Verify that swap has taken place in the pool
    assert_eq!(
        // There was 1 million USDC as a start
        1_000_201 * ONE_USDC,
        ctx.tokens.get_balance(&swap_pool.token_a).await?
    );

    // Swap in a different order
    // Now user A swaps their USDC for TSOL
    user_a
        .swap(
            &swap_program_id,
            &env.tsol,
            &env.usdc,
            &user_a_tsol_transit,
            &user_a_usdc_transit,
            &swap_pool,
            TokenChange::set(0),
            90 * 10 * ONE_USDC,
        )
        .await?;

    // Verify that swap has taken place in the pool
    assert!(
        1_000_201 * ONE_USDC - (90 * 10 * ONE_USDC)
            >= ctx.tokens.get_balance(&swap_pool.token_a).await?
    );
    assert!((10_000 + 10) * ONE_TSOL >= ctx.tokens.get_balance(&swap_pool.token_b).await?);

    Ok(())
}
