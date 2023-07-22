use std::ops::Deref;
use std::rc::Rc;

use jet_client::state::dexes::DexState;
use jet_client::swaps::SwapStep;
use jet_client::JetClient;
use jet_environment::builder::WHIRLPOOL_TICK_SPACING;
use jet_instructions::orca::derive_whirlpool;
use jet_instructions::test_service::derive_whirlpool_config;
use jet_program_common::programs::ORCA_WHIRLPOOL;
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
            whirlpools: vec![],
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
    let whirlpool_address = user_client
        .state()
        .addresses_of::<DexState>()
        .first()
        .cloned()
        .unwrap();

    let swap_steps = [SwapStep {
        from_token: usdc.mint,
        to_token: tsol.mint,
        program: ORCA_WHIRLPOOL,
        swap_pool: whirlpool_address,
    }];

    user_account.update_lookup_tables().await.unwrap();
    user_account
        .swaps()
        .route_swap(&swap_steps, deposit_amount, 1)
        .await
        .unwrap();
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
