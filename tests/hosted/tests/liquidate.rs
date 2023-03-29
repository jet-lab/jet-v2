use std::sync::Arc;

use anyhow::Result;

use hosted_tests::{
    context::MarginTestContext,
    margin::MarginUser,
    margin_test_context,
    setup_helper::{setup_token, setup_user},
    test_user::TestLiquidator,
};
use jet_margin::ErrorCode;
use jet_margin_sdk::{solana::transaction::InverseSendTransactionBuilder, tokens::TokenPrice};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;

use jet_margin_pool::TokenChange;
use jet_simulation::assert_custom_program_error;

const ONE_USDC: u64 = 1_000_000;
const ONE_TSOL: u64 = LAMPORTS_PER_SOL;

struct Scenario1 {
    usdc: Pubkey,
    user_a: MarginUser,
    user_b: MarginUser,
    liquidator: TestLiquidator,
}

macro_rules! scenario1 {
    () => {{
        let ctx = margin_test_context!();
        scenario1_with_ctx(&ctx).await.map(|scen| (ctx, scen))
    }};
}
use scenario1;

/// User A deposited 5'000'000 USD worth, borrowed 800'000 USD worth
/// User B deposited 1'000'000 USD worth, borrowed 3'500'000 USD worth
/// TSOL collateral counts 95%
/// Total collateral = 3'500'000 + 1'000'000 * 95% = 4'450'000
/// Total claims = 3'500'000
/// C ratio = 127%
#[allow(clippy::erasing_op)]
async fn scenario1_with_ctx(ctx: &Arc<MarginTestContext>) -> Result<Scenario1> {
    let usdc = setup_token(ctx, 6, 1_00, 4_00, 1.0).await?;
    let tsol = setup_token(ctx, 9, 95, 4_00, 100.0).await?;

    // Create wallet for the liquidator
    let user_a = setup_user(
        ctx,
        vec![(usdc, 5_000_000 * ONE_USDC, 5_000_000 * ONE_USDC)],
    )
    .await?;
    let user_b = setup_user(ctx, vec![(tsol, 0, 10_000 * ONE_TSOL)]).await?;

    // Have each user borrow the other's funds

    vec![
        ctx.tokens().refresh_to_same_price_tx(&tsol).await.unwrap(),
        user_a
            .user
            .tx
            .borrow(&tsol, TokenChange::shift(8000 * ONE_TSOL))
            .await
            .unwrap(),
    ]
    .send_and_confirm_condensed_in_order(&ctx.rpc())
    .await
    .unwrap();

    vec![
        ctx.tokens().refresh_to_same_price_tx(&usdc).await?,
        user_b
            .user
            .tx
            .borrow(&usdc, TokenChange::shift(3_500_000 * ONE_USDC))
            .await?,
    ]
    .send_and_confirm_condensed_in_order(&ctx.rpc())
    .await
    .unwrap();

    // User A deposited 5'000'000 USD worth, borrowed 800'000 USD worth
    // User B deposited 1'000'000 USD worth, borrowed 3'500'000 USD worth
    // TSOL collateral counts 95%
    // Total collateral = 3'500'000 + 1'000'000 * 95% = 4'450'000
    // Total claims = 3'500'000
    // C ratio = 127%

    ctx.tokens()
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
        user_a: user_a.user.clone(),
        user_b: user_b.user.clone(),
        usdc,
        liquidator: TestLiquidator::new(ctx).await?,
    })
}

/// Account liquidations
///
/// This test creates 2 users who deposit collateral and take loans in the
/// margin account. The price of the loan token moves adversely, leading to
/// liquidations. One user borrowed conservatively, and is not subject to
/// liquidation, while the other user gets liquidated.
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn cannot_liquidate_healthy_user() -> Result<()> {
    let scen = scenario1!()?.1;

    // A liquidator tries to liquidate User A, it should not be able to
    let result = scen.liquidator.begin(&scen.user_a, true).await;
    assert_custom_program_error(ErrorCode::Healthy, result);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn cannot_end_nonexistent_liquidation() -> Result<()> {
    let scen = scenario1!()?.1;

    // A liquidator should not be able to end liquidation of an account that is
    // not being liquidated
    let result = scen
        .liquidator
        .for_user(&scen.user_a)
        .unwrap()
        .liquidate_end(None)
        .await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn cannot_transact_when_being_liquidated() -> Result<()> {
    let scen = scenario1!().unwrap().1;

    // A liquidator tries to liquidate User B, it should be able to
    scen.liquidator.begin(&scen.user_b, false).await.unwrap();

    // When User B is being liquidated, they should be unable to transact
    let result = scen
        .user_b
        .margin_repay(&scen.usdc, TokenChange::shift(1_000_000 * ONE_USDC))
        .await;
    assert_custom_program_error(ErrorCode::Liquidating, result);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn liquidator_can_repay_from_unhealthy_to_healthy_state() -> Result<()> {
    let scen = scenario1!().unwrap().1;

    let liq = scen.liquidator.begin(&scen.user_b, true).await.unwrap();
    liq.verify_healthy().await.err().unwrap();

    // Execute a repayment on behalf of the user
    liq.margin_repay(&scen.usdc, 1_000_000 * ONE_USDC)
        .await
        .unwrap();

    // User B now has
    // Collateral (800'000 * 0.95) + 2'500'000 = 1'260'000
    // Claim 2'500'000
    // C ratio = .unwrap()
    scen.user_b.verify_healthy().await.unwrap();

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn liquidator_can_end_liquidation_when_unhealthy() -> Result<()> {
    let scen = scenario1!().unwrap().1;

    let liq = scen.liquidator.begin(&scen.user_b, true).await.unwrap();
    liq.verify_healthy().await.err().unwrap();
    liq.liquidate_end(None).await.unwrap();

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn no_one_else_can_liquidate_after_liquidate_begin() -> Result<()> {
    let (ctx, scen) = scenario1!().unwrap();

    // A liquidator tries to liquidate User B, it should be able to
    scen.liquidator.begin(&scen.user_b, false).await.unwrap();

    // If an account is still being liquidated, another liquidator should not
    // be able to begin or stop liquidating it
    let rogue_liquidator = ctx.create_liquidator(100).await.unwrap();
    let user_b_rliq = ctx
        .margin_client()
        .liquidator(&rogue_liquidator, scen.user_b.owner(), scen.user_b.seed())
        .unwrap();

    // Should fail to begin liquidation
    assert_custom_program_error(
        ErrorCode::Liquidating,
        user_b_rliq.liquidate_begin(true).await,
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn liquidation_completes() -> Result<()> {
    let scen = scenario1!().unwrap().1;

    // A liquidator tries to liquidate User B, it should be able to
    let user_b_liq = scen.liquidator.begin(&scen.user_b, false).await.unwrap();

    // Execute a repayment on behalf of the user
    user_b_liq
        .margin_repay(&scen.usdc, 1_000_000 * ONE_USDC)
        .await
        .unwrap();

    // The liquidator should be able to end liquidation after liquidating
    user_b_liq.liquidate_end(None).await.unwrap();

    // User B should now be able to transact again
    scen.user_b
        .margin_repay(&scen.usdc, TokenChange::shift(200_000 * ONE_USDC))
        .await
        .unwrap();

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn cannot_withdraw_too_much_during_liquidation() -> Result<()> {
    let scen = scenario1!().unwrap().1;

    let user_b_liq = scen.liquidator.begin(&scen.user_b, true).await.unwrap();

    let result = user_b_liq.withdraw(&scen.usdc, 200_000 * ONE_USDC).await;

    assert_custom_program_error(ErrorCode::LiquidationLostValue, result);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn can_withdraw_some_during_liquidation() -> Result<()> {
    let scen = scenario1!().unwrap().1;

    let user_b_liq = scen.liquidator.begin(&scen.user_b, true).await.unwrap();
    user_b_liq
        .withdraw(&scen.usdc, 40 * ONE_USDC)
        .await
        .unwrap();

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
#[ignore = "ignored while there is no constraint on borrowing"]
async fn cannot_borrow_too_much_during_liquidation() -> Result<()> {
    let scen = scenario1!().unwrap().1;

    let user_b_liq = scen.liquidator.begin(&scen.user_b, false).await.unwrap();

    let result = user_b_liq.borrow(&scen.usdc, 500_000 * ONE_USDC).await;
    assert_custom_program_error(ErrorCode::LiquidationLostValue, result);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn can_borrow_some_during_liquidation() -> Result<()> {
    let scen = scenario1!().unwrap().1;

    let user_b_liq = scen.liquidator.begin(&scen.user_b, false).await.unwrap();
    user_b_liq
        .borrow(&scen.usdc, 5_000 * ONE_USDC)
        .await
        .unwrap();

    Ok(())
}

/// The owner is provided as the authority and signs
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn owner_cannot_end_liquidation_before_timeout() -> Result<()> {
    let scen = scenario1!().unwrap().1;

    scen.liquidator.begin(&scen.user_b, false).await.unwrap();

    let result = scen
        .user_b
        .liquidate_end(Some(scen.liquidator.wallet.pubkey()))
        .await;
    assert_custom_program_error(ErrorCode::UnauthorizedLiquidator, result);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
#[cfg(not(feature = "localnet"))]
async fn owner_can_end_liquidation_after_timeout() -> Result<()> {
    let (ctx, scen) = scenario1!().unwrap();

    scen.liquidator.begin(&scen.user_b, false).await.unwrap();

    let mut clock = ctx.rpc().get_clock().await.unwrap();
    clock.unix_timestamp += 61;
    ctx.rpc().set_clock(clock).await.unwrap();

    scen.user_b
        .liquidate_end(Some(scen.liquidator.wallet.pubkey()))
        .await
        .unwrap();

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn liquidator_permission_is_removable() -> Result<()> {
    let (ctx, scen) = scenario1!().unwrap();

    ctx.margin_client()
        .set_liquidator_metadata(scen.liquidator.wallet.pubkey(), false)
        .await
        .unwrap();

    // A liquidator tries to liquidate User B, it should no longer have authority to do that
    let result = scen.liquidator.begin(&scen.user_b, false).await;

    assert_custom_program_error(anchor_lang::error::ErrorCode::AccountNotInitialized, result);

    Ok(())
}
