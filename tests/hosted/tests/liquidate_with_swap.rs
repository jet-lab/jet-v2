use anyhow::Result;

use hosted_tests::{
    margin_test_context,
    setup_helper::{liquidators, tokens, users},
};

use jet_margin_pool::TokenChange;

/// This was moved out to a separate file since it was very flaky when run
/// concurrently with the other liquidate tests on a localnet. cargo test will
/// never run this concurrently with other files because it is in a test folder
/// where each file is treated as a separate crate.
///
/// Also, this test works very differently from the other liquidate tests since
/// it isn't using the scenario in that file and instead uses these helper
/// methods to do additional and more generic setup, plus executes an actual
/// swap, which the other liquidate tests are not equipped to do.
#[tokio::test(flavor = "multi_thread")]
async fn liquidate_with_swap() -> Result<()> {
    let ctx = margin_test_context!();
    let ([usdc, sol], swaps, pricer) = tokens(&ctx).await.unwrap();
    let [liquidator] = liquidators(&ctx).await.unwrap();
    let [user0, user1] = users(&ctx).await.unwrap();
    user0.deposit(&usdc, 1_000).await.unwrap();
    user1.deposit(&sol, 1_000).await.unwrap();
    user1.borrow_to_wallet(&usdc, 800).await.unwrap();
    pricer.set_price(&sol, 0.9).await.unwrap();
    liquidator
        .liquidate(
            &user1.user,
            &swaps,
            &sol,
            &usdc,
            TokenChange::shift(800),
            700,
        )
        .await
        .unwrap();
    user1.borrow_to_wallet(&usdc, 5).await.unwrap();

    Ok(())
}
