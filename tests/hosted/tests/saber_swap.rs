#![cfg_attr(not(feature = "localnet"), allow(unused))]

use std::collections::HashSet;

use anyhow::Error;

use jet_margin_sdk::{
    swap::saber_swap::SaberSwapPool, tokens::TokenPrice, tx_builder::TokenDepositsConfig,
};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;

use hosted_tests::{
    context::MarginTestContext, margin::MarginPoolSetupInfo, margin_test_context,
    saber_swap::SaberSwapPoolConfig,
};

use jet_margin::TokenKind;
use jet_margin_pool::{MarginPoolConfig, PoolFlags, TokenChange};
use jet_simulation::{assert_custom_program_error, create_wallet};

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
    msol: Pubkey,
    tsol: Pubkey,
}

async fn setup_environment(ctx: &MarginTestContext) -> Result<TestEnv, Error> {
    let msol = ctx.tokens.create_token(9, None, None).await?;
    let msol_oracle = ctx.tokens.create_oracle(&msol).await?;
    let tsol = ctx.tokens.create_token(9, None, None).await?;
    let tsol_oracle = ctx.tokens.create_oracle(&tsol).await?;

    let pools = [
        MarginPoolSetupInfo {
            token: msol,
            token_kind: TokenKind::Collateral,
            collateral_weight: 90,
            max_leverage: 3_00,
            config: DEFAULT_POOL_CONFIG,
            oracle: msol_oracle,
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
                    collateral_weight: pool_info.collateral_weight,
                }),
            )
            .await?;
        ctx.margin.create_pool(&pool_info).await?;
    }

    Ok(TestEnv { msol, tsol })
}
// TODO: this should pass locally
#[cfg(feature = "localnet")]
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn saber_swap() -> Result<(), anyhow::Error> {
    let swap_program_id = saber_client::id();
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

    // Create a swap pool with sufficient liquidity
    let swap_pool = SaberSwapPool::configure(
        &ctx.solana,
        &env.msol,
        &env.tsol,
        // Set a 1.06 rate
        10_000 * ONE_MSOL,
        10_600 * ONE_TSOL,
    )
    .await?;

    // Check if the swap pool can be found
    let mut supported_mints = HashSet::new();
    supported_mints.insert(env.msol);
    supported_mints.insert(env.tsol);

    let swap_pools = SaberSwapPool::get_pools(&ctx.rpc, &supported_mints)
        .await
        .unwrap();
    assert_eq!(swap_pools.len(), 1);

    for pool in swap_pools.values() {
        assert_eq!(swap_pool.pool, pool.pool);
    }

    let user_a_msol_transit = user_a.create_deposit_position(&env.msol).await?;
    let user_a_tsol_transit = user_a.create_deposit_position(&env.tsol).await?;

    // Create some tokens for each user to deposit
    let user_a_msol_account = ctx
        .tokens
        .create_account_funded(&env.msol, &wallet_a.pubkey(), 100 * ONE_MSOL)
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
            &env.msol,
            &user_a_msol_account,
            TokenChange::shift(100 * ONE_MSOL),
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
    assert_eq!(0, ctx.tokens.get_balance(&user_a_msol_account).await?);
    assert_eq!(
        90 * ONE_TSOL,
        ctx.tokens.get_balance(&user_a_tsol_account).await?
    );
    assert_eq!(0, ctx.tokens.get_balance(&user_b_tsol_account).await?);

    user_a.refresh_all_pool_positions().await?;
    user_b.refresh_all_pool_positions().await?;

    // Now user A swaps their MSOL for TSOL
    user_a
        .saber_swap(
            &swap_program_id,
            &env.msol,
            &env.tsol,
            &user_a_msol_transit,
            &user_a_tsol_transit,
            &swap_pool,
            TokenChange::shift(ONE_MSOL),
            // we want a minimum of 0.9 SOL for 1 MSOL
            ONE_TSOL * 99 / 106,
        )
        .await?;

    // Verify that swap has taken place in the pool
    assert_eq!(
        // There was 10'000 MSOL as a start
        10_001 * ONE_MSOL,
        ctx.tokens.get_balance(&swap_pool.token_a).await?
    );

    // assert!(
    //     // Pool balance less almost 1 SOL
    //     10_000 * ONE_TSOL - (ONE_TSOL * 100 / 106) >= ctx.tokens.get_balance(&swap_pool.token_b).await?
    // );

    // Trying to withdraw by setting balance > actual should return an error
    let result = user_a
        .saber_swap(
            &swap_program_id,
            &env.msol,
            &env.tsol,
            &user_a_msol_transit,
            &user_a_tsol_transit,
            &swap_pool,
            TokenChange::set(200 * ONE_MSOL),
            // Value doesn't matter
            ONE_TSOL,
        )
        .await;
    assert_custom_program_error(jet_margin_pool::ErrorCode::InvalidSetTo, result);

    // Trying to swap 0 tokens should return an error
    let result = user_a
        .saber_swap(
            &swap_program_id,
            &env.msol,
            &env.tsol,
            &user_a_msol_transit,
            &user_a_tsol_transit,
            &swap_pool,
            TokenChange::set(99 * ONE_MSOL),
            // Value doesn't matter
            ONE_TSOL,
        )
        .await;
    assert_custom_program_error(jet_margin_swap::ErrorCode::NoSwapTokensWithdrawn, result);

    // Swap more, setting the change to a `set(x)`
    user_a
        .saber_swap(
            &swap_program_id,
            &env.msol,
            &env.tsol,
            &user_a_msol_transit,
            &user_a_tsol_transit,
            &swap_pool,
            TokenChange::set(79 * ONE_MSOL),
            // we want a minimum of 0.9 SOL for 20 MSOL (100 - 1 - 79)
            20 * ONE_TSOL * 99 / 106,
        )
        .await?;

    // Verify that swap has taken place in the pool
    assert_eq!(
        // There was 1 million MSOL as a start
        10_021 * ONE_MSOL,
        ctx.tokens.get_balance(&swap_pool.token_a).await?
    );

    // Swap in a different order
    // Now user A swaps their MSOL for TSOL
    user_a
        .saber_swap(
            &swap_program_id,
            &env.tsol,
            &env.msol,
            &user_a_tsol_transit,
            &user_a_msol_transit,
            &swap_pool,
            TokenChange::set(0),
            20 * ONE_MSOL * 99 / 100,
        )
        .await?;

    // Verify that swap has taken place in the pool
    assert!(
        10_021 * ONE_MSOL - (20 * ONE_MSOL * 99 / 100)
            >= ctx.tokens.get_balance(&swap_pool.token_a).await?
    );
    let balance = ctx.tokens.get_balance(&swap_pool.token_b).await?;
    assert!((10_600 + 10) * ONE_TSOL >= balance);

    Ok(())
}
