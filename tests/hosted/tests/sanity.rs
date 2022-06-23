use anyhow::Error;

use jet_margin::PositionKind;
use jet_margin_pool::{Amount, MarginPoolConfig, PoolFlags};
use jet_metadata::TokenKind;
use jet_simulation::{assert_custom_program_error, create_wallet};

use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;

use hosted_tests::{
    context::{test_context, MarginTestContext},
    margin::MarginPoolSetupInfo,
    tokens::TokenPrice,
};

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
            collateral_weight: 1_00,
            max_leverage: 10_00,
            config: DEFAULT_POOL_CONFIG,
            oracle: usdc_oracle,
        },
        MarginPoolSetupInfo {
            token: tsol,
            fee_destination: tsol_fees,
            token_kind: TokenKind::Collateral,
            collateral_weight: 95,
            max_leverage: 4_00,
            config: DEFAULT_POOL_CONFIG,
            oracle: tsol_oracle,
        },
    ];

    for pool_info in pools {
        ctx.margin.create_pool(&pool_info).await?;
    }

    Ok(TestEnv { usdc, tsol })
}

/// Sanity test for the margin system
///
/// This serves as an example for writing mocked integration tests for the
/// margin system. This particular test will create two users which execute
/// a series of deposit/borrow/repay/withdraw actions onto the margin pools
/// via their margin accounts.
#[tokio::test(flavor = "multi_thread")]
async fn sanity_test() -> Result<(), anyhow::Error> {
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

    // Create some tokens for each user to deposit
    let user_a_usdc_account = ctx
        .tokens
        .create_account_funded(&env.usdc, &wallet_a.pubkey(), 1_000_000 * ONE_USDC)
        .await?;
    let user_b_tsol_account = ctx
        .tokens
        .create_account_funded(&env.tsol, &wallet_b.pubkey(), 1_000 * ONE_TSOL)
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
        .deposit(&env.usdc, &user_a_usdc_account, 1_000_000 * ONE_USDC)
        .await?;
    user_b
        .deposit(&env.tsol, &user_b_tsol_account, 1_000 * ONE_TSOL)
        .await?;

    // Verify user tokens have been deposited
    assert_eq!(0, ctx.tokens.get_balance(&user_a_usdc_account).await?);
    assert_eq!(0, ctx.tokens.get_balance(&user_b_tsol_account).await?);

    user_a.refresh_all_pool_positions().await?;
    user_b.refresh_all_pool_positions().await?;

    // Have each user borrow the other's funds
    user_a.borrow(&env.tsol, 10 * ONE_TSOL).await?;
    user_b.borrow(&env.usdc, 1_000 * ONE_USDC).await?;

    // User should not be able to borrow more than what's in the pool
    let excess_borrow_result = user_a.borrow(&env.tsol, 5_000 * ONE_TSOL).await;

    assert_custom_program_error(
        jet_margin_pool::ErrorCode::InsufficientLiquidity,
        excess_borrow_result,
    );

    // Users repay their loans
    user_a
        .repay(&env.tsol, Amount::tokens(10 * ONE_TSOL))
        .await?;
    user_b
        .repay(&env.usdc, Amount::tokens(1_000 * ONE_USDC))
        .await?;

    // Users withdraw their funds
    user_a
        .withdraw(
            &env.usdc,
            &user_a_usdc_account,
            Amount::tokens(1_000_000 * ONE_USDC),
        )
        .await?;
    user_b
        .withdraw(
            &env.tsol,
            &user_b_tsol_account,
            Amount::tokens(1_000 * ONE_TSOL),
        )
        .await?;

    // Now verify that the users got all their tokens back
    assert_eq!(
        1_000_000 * ONE_USDC,
        ctx.tokens.get_balance(&user_a_usdc_account).await?
    );
    assert_eq!(
        1_000 * ONE_TSOL,
        ctx.tokens.get_balance(&user_b_tsol_account).await?
    );

    // Close a specific position
    user_a
        .close_token_position(&env.tsol, PositionKind::Deposit)
        .await?;

    // Close all User A empty accounts
    user_a.close_empty_positions().await?;

    // Close User A's margin account
    user_a.close_account().await?;

    // User B only had a TSOL deposit, they should not be able to close
    // a non-existent loan position by closing both deposit and loan.
    let b_close_tsol_result = user_b.close_token_positions(&env.tsol).await;
    // Error ref: https://github.com/project-serum/anchor/blob/v0.23.0/lang/src/error.rs#L171
    assert_custom_program_error(
        anchor_lang::error::ErrorCode::AccountNotInitialized,
        b_close_tsol_result,
    );

    // NOTE: due to how the simulator works, the deposit will be closed
    // as the state gets mutated regardless of an error.
    // So the user should have 3 positions left, but they have 2

    // It should not be possible to close User B account as it is not empty
    let b_close_acc_result = user_b.close_account().await;
    assert_custom_program_error(jet_margin::ErrorCode::AccountNotEmpty, b_close_acc_result);

    // User B had a USDC loan which created a corresponding deposit.
    // They should be able to close all USDC positions
    user_b.close_token_positions(&env.usdc).await?;

    // Close User B's account
    user_b.close_account().await?;

    Ok(())
}
