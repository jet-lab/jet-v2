use anyhow::Error;

use jet_control::TokenMetadataParams;
use jet_margin_sdk::instructions::control::TokenConfiguration;
use jet_simulation::tokens::TokenPrice;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;

use hosted_tests::context::{test_context, MarginTestContext};

use jet_margin_pool::{Amount, MarginPoolConfig, PoolFlags};
use jet_metadata::TokenKind;
use jet_simulation::create_wallet;
use jet_simulation::margin::MarginPoolSetupInfo;

const ONE_USDC: u64 = 1_000_000;
const ONE_USDT: u64 = 1_000_000;
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
    usdt: Pubkey,
    tsol: Pubkey,
}

async fn setup_environment(ctx: &MarginTestContext) -> Result<TestEnv, Error> {
    let usdc = ctx.tokens.create_token(6, None, None).await?;
    let usdc_fees = ctx
        .tokens
        .create_account(&usdc, &ctx.authority.pubkey())
        .await?;
    let usdc_oracle = ctx.tokens.create_oracle(&usdc).await?;
    let usdt = ctx.tokens.create_token(6, None, None).await?;
    let usdt_fees = ctx
        .tokens
        .create_account(&usdt, &ctx.authority.pubkey())
        .await?;
    let usdt_oracle = ctx.tokens.create_oracle(&usdt).await?;
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
            token: usdt,
            fee_destination: usdt_fees,
            token_kind: TokenKind::Collateral,
            collateral_weight: 10_000,
            config: DEFAULT_POOL_CONFIG,
            oracle: usdt_oracle,
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
            &usdt,
            &TokenConfiguration {
                pyth_price: Some(usdt_oracle.price),
                pyth_product: Some(usdt_oracle.product),
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

    Ok(TestEnv { usdc, usdt, tsol })
}

/// Pool repayment test
///
/// Tests that users cannot over-pay their claims.
/// The test creates 3 users:
/// 1. Deposits Token A, borrows Token B
/// 2. Deposits Token B, borrows Token A
/// 3. Deposits Token C, borrows Tokens A and B, tries to overpay either
#[tokio::test]
async fn pool_overpayment() -> Result<(), anyhow::Error> {
    // Get the mocked runtime
    let ctx = test_context().await;

    let env = setup_environment(ctx).await?;

    // Create our two user wallets, with some SOL funding to get started
    let wallet_a = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;
    let wallet_b = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;
    let wallet_c = create_wallet(&ctx.rpc, 10 * LAMPORTS_PER_SOL).await?;

    // Create the user context helpers, which give a simple interface for executing
    // common actions on a margin account
    let user_a = ctx.margin.user(&wallet_a).await?;
    let user_b = ctx.margin.user(&wallet_b).await?;
    let user_c = ctx.margin.user(&wallet_c).await?;

    // Initialize the margin accounts for each user
    user_a.create_account().await?;
    user_b.create_account().await?;
    user_c.create_account().await?;

    // Create some tokens for each user to deposit
    let user_a_usdc_account = ctx
        .tokens
        .create_account_funded(&env.usdc, &wallet_a.pubkey(), 1_000_000 * ONE_USDC)
        .await?;
    let user_b_tsol_account = ctx
        .tokens
        .create_account_funded(&env.tsol, &wallet_b.pubkey(), 1_000 * ONE_TSOL)
        .await?;
    let user_c_usdt_account = ctx
        .tokens
        .create_account_funded(&env.usdt, &wallet_c.pubkey(), 1_000_000 * ONE_USDT)
        .await?;
    let user_c_tsol_account = ctx
        .tokens
        .create_account_funded(&env.tsol, &wallet_c.pubkey(), 500 * ONE_TSOL)
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
            // Set price to 1 USD +- 0.01
            &env.usdt,
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
        .deposit(&env.usdc, &user_a_usdc_account, 1_000_000 * ONE_USDC)
        .await?;
    user_b
        .deposit(&env.tsol, &user_b_tsol_account, 1_000 * ONE_TSOL)
        .await?;
    user_c
        .deposit(&env.usdt, &user_c_usdt_account, 1_000_000 * ONE_USDT)
        .await?;
    // User deposits TSOL which they will use to over-pay
    user_c
        .deposit(&env.tsol, &user_c_tsol_account, 500 * ONE_TSOL)
        .await?;

    // Verify user tokens have been deposited
    assert_eq!(0, ctx.tokens.get_balance(&user_a_usdc_account).await?);
    assert_eq!(0, ctx.tokens.get_balance(&user_b_tsol_account).await?);
    assert_eq!(0, ctx.tokens.get_balance(&user_c_usdt_account).await?);

    user_a.refresh_all_pool_positions().await?;
    user_b.refresh_all_pool_positions().await?;
    user_c.refresh_all_pool_positions().await?;

    // User A borrows enough TSOL so that there is sufficient liquidity when C repays
    user_a.borrow(&env.tsol, 1_000 * ONE_TSOL).await?;
    // User B borrows an irrelevant amount
    user_b.borrow(&env.usdc, 1_000 * ONE_USDC).await?;
    user_c.borrow(&env.usdc, 1_000 * ONE_USDC).await?;
    // Borrow TSOL which user will try to overpay
    user_c.borrow(&env.tsol, 100 * ONE_TSOL).await?;

    // User overpays their loan by 300 TSOL, they should only repay the maximum amount
    user_c
        .repay(&env.tsol, Amount::tokens(400 * ONE_TSOL))
        .await?;

    // TODO: We do not yet have functions for getting a pool balance,
    // we use a withdrawal to test that the overpaid tokens are still in the deposit.
    // User C has 500 (deposit) + 100 (borrow) - 100 (max repay) tokens
    user_c
        .withdraw(
            &env.tsol,
            &user_c_tsol_account,
            Amount::tokens(500 * ONE_TSOL),
        )
        .await?;

    assert_eq!(
        500 * ONE_TSOL,
        ctx.tokens.get_balance(&user_c_tsol_account).await?
    );

    // User C should be able to close all TSOL positions as loan is paid and deposit withdrawn
    user_c.close_token_positions(&env.tsol).await?;

    Ok(())
}
