use std::collections::HashSet;
use std::ops::Deref;
use std::rc::Rc;

use jet_client::JetClient;
use jet_environment::builder::WHIRLPOOL_TICK_SPACING;
use jet_instructions::margin_swap::{MarginSwapRouteIxBuilder, SwapContext};
use jet_instructions::orca::derive_whirlpool;
use jet_instructions::test_service::derive_whirlpool_config;
use jet_margin_pool::TokenChange;
use jet_margin_sdk::swap::whirlpool::WhirlpoolSwap;
use jet_solana_client::util::keypair;

use hosted_tests::actions::*;
use hosted_tests::context::TestContextSetupInfo;
use hosted_tests::environment::TestToken;
use hosted_tests::test_context;

#[tokio::test]
async fn whirlpool_swap_workflow() -> anyhow::Result<()> {
    let ctx = test_context! {
        setup: &TestContextSetupInfo {
            is_restricted: false,
            tokens: vec![
                TestToken::with_pool("TSOL").into(),
                TestToken::with_pool("USDC").into(),
            ],
            dexes: vec![("orca-whirlpool", "TSOL/USDC")],
        }
    };

    let rpc = ctx.inner.solana.rpc2.clone();

    // derive mints for default config tokens
    let usdc = Token::from_context(&ctx, "USDC");
    let tsol = Token::from_context(&ctx, "TSOL");

    let (base_real, quote_real) = (
        std::cmp::min(tsol.mint, usdc.mint),
        std::cmp::max(tsol.mint, usdc.mint),
    );

    let target_pool_price = if base_real == tsol.mint {
        22.0
    } else {
        1.0 / 22.0
    };

    // Add liquidity
    let whirlpool = derive_whirlpool(
        &derive_whirlpool_config(),
        &base_real,
        &quote_real,
        WHIRLPOOL_TICK_SPACING,
    )
    .0;
    jet_testing::whirlpool::set_liquidity(
        ctx.rpc().payer(),
        rpc.deref(),
        whirlpool,
        target_pool_price,
        9,
    )
    .await?;

    // Create user wallet to try swap
    let margin_user = ctx.inner.create_margin_user(1_000).await.unwrap();

    let user_client = JetClient::new(
        ctx.inner.solana.rpc2.clone(),
        Rc::new(keypair::clone(&margin_user.signer)),
        ctx.config.clone(),
        &ctx.config.airspaces[0].name,
    )
    .unwrap();

    // Add some user funds to swap with
    let deposit_amount = usdc.amount(1_000_000.0);
    airdrop(&user_client, &usdc, deposit_amount).await;
    airdrop(&user_client, &tsol, 100).await;

    let user_account = user_client.margin().accounts()[0].clone();
    deposit(&user_account, &usdc, deposit_amount).await.unwrap();
    deposit(&user_account, &tsol, 1).await.unwrap();

    // Swap USDC for TSOL
    let mut swap_builder = MarginSwapRouteIxBuilder::try_new(
        SwapContext::MarginPositions,
        user_account.address(),
        usdc.mint,
        tsol.mint,
        TokenChange::shift(deposit_amount),
        1, // Get at least 1 token back
    )
    .unwrap();

    let mut supported_mints = HashSet::new();
    supported_mints.insert(usdc.mint);
    supported_mints.insert(tsol.mint);

    let whirlpool_swaps = WhirlpoolSwap::get_pools(rpc.deref(), &supported_mints)
        .await
        .unwrap();
    let (_, whirlpool_swap) = whirlpool_swaps.into_iter().next().unwrap();

    if base_real == tsol.mint {
        swap_builder
            .add_swap_leg(&whirlpool_swap.swap_b_to_a(), 0)
            .unwrap();
    } else {
        swap_builder
            .add_swap_leg(&whirlpool_swap.swap_a_to_b(), 0)
            .unwrap();
    };

    swap_builder.finalize().unwrap();

    margin_user.route_swap(&swap_builder, &[]).await.unwrap();

    user_account.sync().await.unwrap();
    let balance = position_balance(&user_account, &tsol);

    assert!(balance > tsol.amount(45_000.0));

    jet_testing::whirlpool::set_liquidity(
        ctx.rpc().payer(),
        rpc.deref(),
        whirlpool,
        target_pool_price * 100.0,
        9,
    )
    .await?;

    Ok(())
}
