use anyhow::Result;

use hosted_tests::{
    context::test_context,
    margin::MarginUser,
    setup_helper::{setup_token, setup_user},
    tokens::TokenPrice,
};
use jet_margin::ErrorCode;
use serial_test::serial;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;

use jet_margin_pool::Amount;
use jet_simulation::{assert_program_error, assert_program_error_code};

const ONE_USDC: u64 = 1_000_000;
const ONE_TSOL: u64 = LAMPORTS_PER_SOL;

struct Scenario1 {
    usdc: Pubkey,
    user_b: MarginUser,
    user_a_liq: MarginUser,
    user_b_liq: MarginUser,
    liquidator: Pubkey,
}

/// User A deposited 5'000'000 USD worth, borrowed 800'000 USD worth
/// User B deposited 1'000'000 USD worth, borrowed 3'500'000 USD worth
/// TSOL collateral counts 95%
/// Total collateral = 3'500'000 + 1'000'000 * 95% = 4'450'000
/// Total claims = 3'500'000
/// C ratio = 127%
#[allow(clippy::erasing_op)]
async fn scenario1() -> Result<Scenario1> {
    let ctx = test_context().await;
    let usdc = setup_token(ctx, 6, 10_000, 1).await?;
    let tsol = setup_token(ctx, 9, 9_500, 100).await?;

    // Create wallet for the liquidator
    let liquidator_wallet = ctx.create_liquidator(100).await?;
    let user_a = setup_user(
        ctx,
        &liquidator_wallet,
        vec![(usdc, 5_000_000 * ONE_USDC, 5_000_000 * ONE_USDC)],
    )
    .await?;
    let user_b = setup_user(ctx, &liquidator_wallet, vec![(tsol, 0, 10_000 * ONE_TSOL)]).await?;

    // Have each user borrow the other's funds
    user_a.user.borrow(&tsol, 8000 * ONE_TSOL).await?;
    user_b.user.borrow(&usdc, 3_500_000 * ONE_USDC).await?;

    // User A deposited 5'000'000 USD worth, borrowed 800'000 USD worth
    // User B deposited 1'000'000 USD worth, borrowed 3'500'000 USD worth
    // TSOL collateral counts 95%
    // Total collateral = 3'500'000 + 1'000'000 * 95% = 4'450'000
    // Total claims = 3'500'000
    // C ratio = 127%

    ctx.tokens
        .set_price(
            // Set price to 80 USD +- 1
            &tsol,
            &TokenPrice {
                exponent: -8,
                price: 8_000_000_000,
                confidence: 100_000_000,
                twap: 8_000_000_000,
            },
        )
        .await?;

    user_a.user.refresh_all_pool_positions().await?;
    user_b.user.refresh_all_pool_positions().await?;

    Ok(Scenario1 {
        user_b: user_b.user,
        user_a_liq: user_a.liquidator,
        user_b_liq: user_b.liquidator,
        usdc,
        liquidator: liquidator_wallet.pubkey(),
    })
}

/// Account liquidations
///
/// This test creates 2 users who deposit collateral and take loans in the
/// margin account. The price of the loan token moves adversely, leading to
/// liquidations. One user borrowed conservatively, and is not subject to
/// liquidation, while the other user gets liquidated.
#[tokio::test]
#[serial]
async fn cannot_liquidate_healthy_user() -> Result<()> {
    let scen = scenario1().await?;

    // A liquidator tries to liquidate User A, it should not be able to
    let result = scen.user_a_liq.liquidate_begin().await;
    assert_program_error!(ErrorCode::Healthy, result);

    Ok(())
}

#[tokio::test]
#[serial]
async fn cannot_end_nonexistent_liquidation() -> Result<()> {
    let scen = scenario1().await?;

    // A liquidator should not be able to end liquidation of an account that is
    // not being liquidated
    let result = scen.user_a_liq.liquidate_end(None).await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
#[serial]
async fn cannot_transact_when_being_liquidated() -> Result<()> {
    let scen = scenario1().await?;

    // A liquidator tries to liquidate User B, it should be able to
    scen.user_b_liq.liquidate_begin().await?;

    // When User B is being liquidated, they should be unable to transact
    let result = scen
        .user_b
        .repay(&scen.usdc, Amount::tokens(1_000_000 * ONE_USDC))
        .await;
    assert_program_error!(ErrorCode::Liquidating, result);

    Ok(())
}

#[tokio::test]
#[serial]
async fn liquidator_cannot_over_repay() -> Result<()> {
    let scen = scenario1().await?;

    scen.user_b_liq.liquidate_begin().await?;

    // Fail a repayment on behalf of the user because it repays too much
    // User B would have
    // Collateral (800'000 * 0.95) + 500'000 = 1'260'000
    // Claim 500'000
    // C ratio = 2.52
    let result = scen
        .user_b_liq
        .repay(&scen.usdc, Amount::tokens(3_000_000 * ONE_USDC))
        .await;
    assert_program_error!(ErrorCode::LiquidationTooHealthy, result);

    Ok(())
}

#[tokio::test]
#[serial]
async fn liquidator_can_repay_from_unhealthy_to_healthy_state() -> Result<()> {
    let scen = scenario1().await?;

    scen.user_b_liq.liquidate_begin().await?;
    scen.user_b_liq.verify_healthy().await.err().unwrap();

    // Execute a repayment on behalf of the user
    scen.user_b_liq
        .repay(&scen.usdc, Amount::tokens(1_000_000 * ONE_USDC))
        .await?;

    // User B now has
    // Collateral (800'000 * 0.95) + 2'500'000 = 1'260'000
    // Claim 2'500'000
    // C ratio = ?
    scen.user_b.verify_healthy().await?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn liquidator_can_end_liquidation_when_unhealthy() -> Result<()> {
    let scen = scenario1().await?;

    scen.user_b_liq.liquidate_begin().await?;

    scen.user_b_liq.verify_healthy().await.err().unwrap();
    scen.user_b_liq.liquidate_end(None).await?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn no_one_else_can_liquidate_after_liquidate_begin() -> Result<()> {
    let ctx = test_context().await;
    let scen = scenario1().await?;

    // A liquidator tries to liquidate User B, it should be able to
    scen.user_b_liq.liquidate_begin().await?;

    // If an account is still being liquidated, another liquidator should not
    // be able to begin or stop liquidating it
    let rogue_liquidator = ctx.create_liquidator(100).await?;
    let user_b_rliq = ctx
        .margin
        .liquidator(&rogue_liquidator, scen.user_b.owner())
        .await?;

    // Should fail to begin liquidation
    assert_program_error!(ErrorCode::Liquidating, user_b_rliq.liquidate_begin().await);

    Ok(())
}

#[tokio::test]
#[serial]
async fn liquidation_completes() -> Result<()> {
    let scen = scenario1().await?;

    // A liquidator tries to liquidate User B, it should be able to
    scen.user_b_liq.liquidate_begin().await?;

    // Execute a repayment on behalf of the user
    scen.user_b_liq
        .repay(&scen.usdc, Amount::tokens(1_000_000 * ONE_USDC))
        .await?;

    // The liquidator should be able to end liquidation after liquidating
    scen.user_b_liq.liquidate_end(None).await?;

    // User B should now be able to transact again
    scen.user_b
        .repay(&scen.usdc, Amount::tokens(200_000 * ONE_USDC))
        .await?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn cannot_withdraw_too_much_during_liquidation() -> Result<()> {
    let ctx = test_context().await;
    let scen = scenario1().await?;

    scen.user_b_liq.liquidate_begin().await?;

    let liquidator_usdc_account = ctx
        .tokens
        .create_account_funded(&scen.usdc, &scen.liquidator, 0)
        .await?;

    let result = scen
        .user_b_liq
        .withdraw(
            &scen.usdc,
            &liquidator_usdc_account,
            Amount::tokens(50000 * ONE_USDC),
        )
        .await;

    assert_program_error!(ErrorCode::LiquidationLostValue, result);

    Ok(())
}

#[tokio::test]
#[serial]
async fn can_withdraw_some_during_liquidation() -> Result<()> {
    let ctx = test_context().await;
    let scen = scenario1().await?;

    let liquidator_usdc_account = ctx
        .tokens
        .create_account_funded(&scen.usdc, &scen.liquidator, 0)
        .await?;

    scen.user_b_liq.liquidate_begin().await?;
    scen.user_b_liq
        .withdraw(
            &scen.usdc,
            &liquidator_usdc_account,
            Amount::tokens(40000 * ONE_USDC),
        )
        .await?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn cannot_borrow_too_much_during_liquidation() -> Result<()> {
    let scen = scenario1().await?;

    scen.user_b_liq.liquidate_begin().await?;

    let result = scen
        .user_b_liq
        .borrow(&scen.usdc, 5_000_000 * ONE_USDC)
        .await;
    assert_program_error!(ErrorCode::LiquidationUnhealthy, result);

    Ok(())
}

#[tokio::test]
#[serial]
async fn can_borrow_some_during_liquidation() -> Result<()> {
    let scen = scenario1().await?;

    scen.user_b_liq.liquidate_begin().await?;
    scen.user_b_liq
        .borrow(&scen.usdc, 500_000 * ONE_USDC)
        .await?;

    Ok(())
}

/// The owner is provided as the authority and signs
#[tokio::test]
#[serial]
async fn owner_cannot_end_liquidation_before_timeout() -> Result<()> {
    let scen = scenario1().await?;

    scen.user_b_liq.liquidate_begin().await?;

    let result = scen
        .user_b
        .liquidate_end(Some(scen.user_b_liq.signer()))
        .await;
    assert_program_error!(ErrorCode::UnauthorizedLiquidator, result);

    Ok(())
}

#[tokio::test]
#[serial]
async fn liquidator_permission_is_removable() -> Result<()> {
    let ctx = test_context().await;
    let scen = scenario1().await?;

    ctx.margin
        .set_liquidator_metadata(scen.liquidator, false)
        .await?;

    // A liquidator tries to liquidate User B, it should no longer have authority to do that
    let result = scen.user_b_liq.liquidate_begin().await;
    assert_program_error_code!(
        anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch as u32,
        result
    );

    Ok(())
}

// todo enable this test when test runtime clock works
// #[tokio::test]
// fn owner_can_end_liquidation_after_timeout(ctx: SyncContext) {
//     let (_, user_b, liquidator) = scenario1(&ctx);
//     user_b.liquidate_begin().await.unwrap();
//     let mut clock = ctx.runtime.get_clock();
//     clock.unix_timestamp += 61;
//     ctx.runtime.set_clock(clock);
//     user_b.liquidate_end(None).unwrap();
// }

// todo enable this test when test runtime clock works
// #[tokio::test]
// fn arbitrary_user_can_end_liquidation_after_timeout(ctx: SyncContext) {
//     let (_, user_b, liquidator) = scenario1(&ctx);
//     user_b.liquidate_begin().await.unwrap();
//     let mut clock = ctx.runtime.get_clock();
//     clock.unix_timestamp += 61;
//     ctx.runtime.set_clock(clock);
//     user_b.liquidate_end_arbitrary(&Pubkey::default(), &Pubkey::default()).unwrap();
// }
