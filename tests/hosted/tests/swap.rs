use anyhow::Error;

use jet_control::TokenMetadataParams;
use jet_margin_sdk::instructions::control::TokenConfiguration;
use jet_margin_swap::orca_swap_v1_metadata;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;

use hosted_tests::{
    context::{test_context, MarginTestContext},
    margin::MarginPoolSetupInfo,
    swap::SwapPool,
    tokens::TokenPrice,
};

use jet_margin_pool::{Amount, MarginPoolConfig, PoolFlags};
use jet_metadata::TokenKind;
use jet_simulation::create_wallet;

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
    management_fee_collect_threshold: 100,
    flags: PoolFlags::ALLOW_LENDING.bits(),
};

struct TestEnv {
    usdc: Pubkey,
    tsol: Pubkey,
}

async fn setup_environment(ctx: &MarginTestContext) -> Result<TestEnv, Error> {
    let usdc = ctx.tokens.create_token(6, None, None).await?;
    let usdc_fees = ctx
        .tokens
        .create_account(&usdc, &ctx.authority.pubkey())
        .await?;
    let usdc_oracle = ctx.tokens.create_oracle(&usdc).await?;
    let tsol = ctx.tokens.create_token(9, None, None).await?;
    let tsol_fees = ctx
        .tokens
        .create_account(&tsol, &ctx.authority.pubkey())
        .await?;
    let tsol_oracle = ctx.tokens.create_oracle(&tsol).await?;

    let pools = [
        MarginPoolSetupInfo {
            token: usdc,
            fee_destination: usdc_fees,
            token_kind: TokenKind::Collateral,
            collateral_weight: 10_000,
            config: DEFAULT_POOL_CONFIG,
            oracle: usdc_oracle,
        },
        MarginPoolSetupInfo {
            token: tsol,
            fee_destination: tsol_fees,
            token_kind: TokenKind::Collateral,
            collateral_weight: 9_500,
            config: DEFAULT_POOL_CONFIG,
            oracle: tsol_oracle,
        },
    ];

    for pool_info in pools {
        ctx.margin.create_pool(&pool_info).await?;
    }

    ctx.margin
        .configure_token(
            &usdc,
            &TokenConfiguration {
                pyth_price: Some(usdc_oracle.price),
                pyth_product: Some(usdc_oracle.product),
                pool_config: Some(DEFAULT_POOL_CONFIG),
                metadata: Some(TokenMetadataParams {
                    token_kind: TokenKind::Collateral,
                    collateral_weight: 10_000,
                    collateral_max_staleness: 0,
                }),
                ..Default::default()
            },
        )
        .await?;

    ctx.margin
        .configure_token(
            &tsol,
            &TokenConfiguration {
                pyth_price: Some(tsol_oracle.price),
                pyth_product: Some(tsol_oracle.product),
                pool_config: Some(DEFAULT_POOL_CONFIG),
                metadata: Some(TokenMetadataParams {
                    token_kind: TokenKind::Collateral,
                    collateral_weight: 9_500,
                    collateral_max_staleness: 0,
                }),
                ..Default::default()
            },
        )
        .await?;

    Ok(TestEnv { usdc, tsol })
}

/// Test token swaps for the official SPL token swap
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn spl_swap_test() -> Result<(), anyhow::Error> {
    swap_test_impl(spl_token_swap::id()).await
}

/// Test token swaps for orca
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn orca_swap_test() -> Result<(), anyhow::Error> {
    swap_test_impl(jet_margin_swap::orca_swap_v1_metadata::id()).await
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
    let user_a = ctx.margin.user(&wallet_a).await?;
    let user_b = ctx.margin.user(&wallet_b).await?;

    // Initialize the margin accounts for each user
    user_a.create_account().await?;
    user_b.create_account().await?;

    // Create a swap pool with sufficient liquidity
    let swap_pool = SwapPool::configure(
        &ctx.rpc,
        &swap_program_id,
        &env.usdc,
        &env.tsol,
        1_000_000 * ONE_USDC,
        10_000 * ONE_TSOL,
    )
    .await?;

    let transit_source_authority = if swap_program_id == orca_swap_v1_metadata::id() {
        &swap_pool.pool_authority
    } else {
        user_a.address()
    };
    
    let usdc_transit_source = ctx
        .tokens
        .create_account(&env.usdc, transit_source_authority)
        .await?;
    let tsol_transit_target = ctx
        .tokens
        .create_account(&env.tsol, user_a.address())
        .await?;
    let usdc_transit_target = ctx
        .tokens
        .create_account(&env.usdc, user_a.address())
        .await?;
    let tsol_transit_source = ctx
        .tokens
        .create_account(&env.tsol, transit_source_authority)
        .await?;
    
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
        .deposit(&env.usdc, &user_a_usdc_account, 1_000 * ONE_USDC)
        .await?;
    user_a
        .deposit(&env.tsol, &user_a_tsol_account, 10 * ONE_TSOL)
        .await?;
    user_b
        .deposit(&env.tsol, &user_b_tsol_account, 10 * ONE_TSOL)
        .await?;

        // // Verify user tokens have been deposited
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
            &usdc_transit_source,
            &tsol_transit_target,
            &swap_pool,
            Amount::tokens(100 * ONE_USDC),
            // we want a minimum of 0.9 SOL for 100 USDC
            Amount::tokens(ONE_TSOL / 10 * 9),
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

    // Swap in a different order
    // Now user A swaps their USDC for TSOL
    user_a
        .swap(
            &swap_program_id,
            &env.tsol,
            &env.usdc,
            &tsol_transit_source,
            &usdc_transit_target,
            &swap_pool,
            Amount::tokens(2 * ONE_TSOL),
            Amount::tokens(180 * ONE_USDC),
        )
        .await?;

    // Verify that swap has taken place in the pool
    assert!(
        1_000_100 * ONE_USDC - 180 * ONE_USDC >= ctx.tokens.get_balance(&swap_pool.token_a).await?
    );
    assert!(
        (10_000 * ONE_TSOL - 900_000_000) + 2 * ONE_TSOL
            >= ctx.tokens.get_balance(&swap_pool.token_b).await?
    );

    Ok(())
}
