use hosted_tests::{bonds::BondsTestManager, context::test_context};

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn full_direct() -> Result<(), anyhow::Error> {
    let ctx = test_context().await;
    // let manager = BondsTestManager::full(ctx.rpc.clone()).await?;
    Ok(())
}
