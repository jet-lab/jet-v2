use hosted_tests::{
    actions::*,
    context::{default_test_setup, TestContext},
    util::assert_program_error,
};
use jet_client::state::margin_pool::MarginPoolCacheExt;

#[tokio::test]
async fn simple_pool_lend_borrow_workflow() -> anyhow::Result<()> {
    let ctx = TestContext::new("simple-pool-lend-borrow", &default_test_setup()).await;

    // derive mints for default config tokens
    let usdc = Token::from_context(&ctx, "USDC");
    let tsol = Token::from_context(&ctx, "TSOL");

    // Create two user wallets to get started
    let user_a = ctx.create_user().await?;
    let user_b = ctx.create_user().await?;

    // Create margin accounts for each user
    user_a.margin().create_account().await?;
    user_b.margin().create_account().await?;

    // Get some tokens for each user to deposit
    airdrop(&user_a, &usdc, usdc.amount(1_000_000.0)).await;
    airdrop(&user_a, &tsol, tsol.amount(1.0)).await;
    airdrop(&user_b, &tsol, tsol.amount(1_000.0)).await;
    airdrop(&user_b, &usdc, usdc.amount(2_000.0)).await;

    // Set the prices for each token
    set_price(&ctx, &usdc, 1.0, 0.01).await;
    set_price(&ctx, &tsol, 10.0, 0.01).await;

    // Get the client for each user's account
    let account_a = user_a.margin().accounts()[0].clone();
    let account_b = user_b.margin().accounts()[0].clone();

    let deposit_amount_a = usdc.amount(1_000_000.0);
    let deposit_amount_b = tsol.amount(1_000.0);

    // Deposit user funds into their margin accounts
    deposit(&account_a, &usdc, deposit_amount_a).await.unwrap();
    deposit(&account_b, &tsol, deposit_amount_b).await.unwrap();

    // Verify user tokens have been deposited
    assert_eq!(deposit_amount_a, position_balance(&account_a, &usdc));
    assert_eq!(deposit_amount_b, position_balance(&account_b, &tsol));

    // Lend user tokens
    pool_lend(&account_a, &usdc, deposit_amount_a)
        .await
        .unwrap();
    pool_lend(&account_b, &tsol, deposit_amount_b)
        .await
        .unwrap();

    // Each user borrows the other's funds
    let borrow_amount_b = usdc.amount(1_000.0);
    let borrow_amount_a = tsol.amount(10.0);

    pool_borrow(&account_a, &tsol, borrow_amount_a)
        .await
        .unwrap();
    pool_borrow(&account_b, &usdc, borrow_amount_b)
        .await
        .unwrap();

    // User should not be able to borrow more than what's in the pool
    let excess_borrow_result = pool_borrow(&account_a, &tsol, tsol.amount(5_000.0)).await;

    assert_program_error(
        jet_margin_pool::ErrorCode::InsufficientLiquidity,
        excess_borrow_result,
    );

    // withdraw borrowed tokens to wallet
    withdraw(&account_a, &tsol, borrow_amount_a).await.unwrap();
    withdraw(&account_b, &usdc, borrow_amount_b).await.unwrap();

    // Users repay their loans
    account_a.pool(&tsol.mint).deposit_repay(None, None).await?;

    deposit(&account_b, &usdc, usdc.amount(2_000.0))
        .await
        .unwrap();
    pool_repay(&account_b, &usdc, None).await.unwrap();

    // Verify accounting
    let usdc_pool = user_a.state().get_pool(&usdc.mint).unwrap();
    let tsol_pool = user_a.state().get_pool(&tsol.mint).unwrap();

    assert_eq!(0, usdc_pool.loan_notes);
    assert_eq!(0, tsol_pool.loan_notes);

    // Users withdraw their funds
    pool_withdraw(&account_a, &usdc, None).await?;
    pool_withdraw(&account_b, &tsol, None).await?;

    // Verify users got their tokens back
    assert_eq!(deposit_amount_a, wallet_balance(&user_a, &usdc));
    assert_eq!(deposit_amount_b, wallet_balance(&user_b, &tsol));

    Ok(())
}

#[tokio::test]
async fn max_pool_util_ratio_after_borrow() -> anyhow::Result<()> {
    let ctx = TestContext::new("max-pool-util-ratio-after-borrow", &default_test_setup()).await;

    // derive mints for default config tokens
    let usdc = Token::from_context(&ctx, "USDC");
    let tsol = Token::from_context(&ctx, "TSOL");

    // Create two user wallets to get started
    let user_a = ctx.create_user().await?;
    let user_b = ctx.create_user().await?;

    // Create margin accounts for each user
    user_a.margin().create_account().await?;
    user_b.margin().create_account().await?;

    // Get some tokens for each user to deposit
    airdrop(&user_a, &usdc, usdc.amount(1_000_000.0)).await;
    airdrop(&user_a, &tsol, tsol.amount(1.0)).await;
    airdrop(&user_b, &tsol, tsol.amount(1_000.0)).await;
    airdrop(&user_b, &usdc, usdc.amount(2_000.0)).await;

    // Set the prices for each token
    set_price(&ctx, &usdc, 1.0, 0.01).await;
    set_price(&ctx, &tsol, 10.0, 0.01).await;

    // Get the client for each user's account
    let account_a = user_a.margin().accounts()[0].clone();
    let account_b = user_b.margin().accounts()[0].clone();

    let deposit_amount_a = usdc.amount(1_000_000.0);
    let deposit_amount_b = tsol.amount(1_000.0);

    // Deposit user funds into their margin accounts
    deposit(&account_a, &usdc, deposit_amount_a).await.unwrap();
    deposit(&account_b, &tsol, deposit_amount_b).await.unwrap();

    // Verify user tokens have been deposited
    assert_eq!(deposit_amount_a, position_balance(&account_a, &usdc));
    assert_eq!(deposit_amount_b, position_balance(&account_b, &tsol));

    // Lend user tokens
    pool_lend(&account_a, &usdc, deposit_amount_a)
        .await
        .unwrap();
    pool_lend(&account_b, &tsol, deposit_amount_b)
        .await
        .unwrap();

    // Each user borrows the other's funds
    let borrow_amount_b = usdc.amount(1_000.0);
    let borrow_amount_a = tsol.amount(10.0);

    pool_borrow(&account_a, &tsol, borrow_amount_a)
        .await
        .unwrap();
    pool_borrow(&account_b, &usdc, borrow_amount_b)
        .await
        .unwrap();

    // User should not be able to borrow beyond the bound of 95% utilisation
    let excess_borrow_result =
        pool_borrow(&account_a, &tsol, tsol.amount((1_000.0 - 10.0) * 0.951)).await;

    assert_program_error(
        jet_margin_pool::ErrorCode::ExceedsMaxBorrowUtilRatio,
        excess_borrow_result,
    );

    // But this should be okay (technically 0.95 should work, but rounding)
    pool_borrow(&account_a, &tsol, tsol.amount((1_000.0 - 10.0) * 0.949))
        .await
        .unwrap();

    Ok(())
}
