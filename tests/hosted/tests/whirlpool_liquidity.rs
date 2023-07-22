use std::ops::Deref;
use std::rc::Rc;

use anchor_lang::prelude::Pubkey;
use hosted_tests::tokens::TokenManager;
use jet_client::state::dexes::DexState;
use jet_client::JetClient;
use jet_environment::builder::WHIRLPOOL_TICK_SPACING;
use jet_instructions::orca::derive_whirlpool;
use jet_instructions::test_service::derive_whirlpool_config;
use jet_margin_sdk::tokens::TokenPrice;
use jet_solana_client::util::keypair;

use hosted_tests::actions::*;
use hosted_tests::context::TestContextSetupInfo;
use hosted_tests::environment::TestToken;
use hosted_tests::test_context;

#[tokio::test]
async fn whirlpool_liquidity_workflow() -> anyhow::Result<()> {
    let ctx = test_context! {
        setup: &TestContextSetupInfo {
            is_restricted: false,
            tokens: vec![
                TestToken::with_pool("TSOL").into(),
                TestToken::with_pool("USDC").into(),
            ],
            dexes: vec![("orca-whirlpool", "TSOL/USDC")],
            whirlpools: vec!["TSOL/USDC"],
        }
    };

    let rpc = ctx.inner.solana.rpc2.clone();

    // derive mints for default config tokens
    let usdc = Token::from_context(&ctx, "USDC");
    let tsol = Token::from_context(&ctx, "TSOL");

    let token_manager = TokenManager::new(ctx.inner.solana.clone());

    token_manager.create_oracle(&usdc.mint).await?;
    token_manager.create_oracle(&tsol.mint).await?;

    // Set oracle prices for valuing the position
    token_manager
        .set_price(
            &usdc.mint,
            &TokenPrice {
                price: 100000000,
                exponent: -8,
                confidence: 1000,
                twap: 100000000,
            },
        )
        .await?;
    token_manager
        .set_price(
            &tsol.mint,
            &TokenPrice {
                price: 2200000000,
                exponent: -8,
                confidence: 100000,
                twap: 2200000000,
            },
        )
        .await?;

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
        2,
    )
    .await?;

    // Create user wallet to add liquidity
    let margin_user = ctx.inner.create_margin_user(1_000).await.unwrap();

    let user_client = JetClient::new(
        ctx.inner.solana.rpc2.clone(),
        Rc::new(keypair::clone(&margin_user.signer)),
        ctx.config.clone(),
        &ctx.config.airspaces[0].name,
    )
    .unwrap();

    // Add some user funds to provide liquidity with
    let deposit_amount_usdc = usdc.amount(1_000.0);
    let deposit_amount_tsol = tsol.amount(1_000.0 / 22.0);
    airdrop(&user_client, &usdc, deposit_amount_usdc).await;
    airdrop(&user_client, &tsol, deposit_amount_tsol).await;

    let user_account = user_client.margin().accounts()[0].clone();
    deposit(&user_account, &usdc, deposit_amount_usdc)
        .await
        .unwrap();
    deposit(&user_account, &tsol, deposit_amount_tsol)
        .await
        .unwrap();

    user_account.send_with_refresh(&[]).await?;

    // Get a client for a whirlpool
    let whirlpool_address = user_client
        .state()
        .addresses_of::<DexState>()
        .first()
        .cloned()
        .unwrap();

    user_account.update_lookup_tables().await.unwrap();
    user_account.sync().await.unwrap();
    // Sync whirlpools before creating a client
    jet_client::state::margin_orca::sync(user_account.client().state()).await?;
    let mut orca_client = user_account.orca(&whirlpool_address)?;

    // Register a margin position
    orca_client.register_position_meta().await?;

    // Register a position in the whirlpool
    // TODO: Better to combine the instructions
    // Provide liquidity between 20.1147 and 23.9087
    let position = orca_client.open_position(30016, 31744).await?;

    // Refresh to update positions
    jet_client::state::margin_orca::sync(user_account.client().state()).await?;

    let tsol_balance = position_balance(&user_account, &tsol);
    let usdc_balance = position_balance(&user_account, &usdc);

    // Add liquidity to the position
    let liquidity_amount_usdc = usdc.amount(500.0);
    let liquidity_amount_tsol = tsol.amount(500.0 / 22.0);
    orca_client
        .add_liquidity(&position, liquidity_amount_tsol, liquidity_amount_usdc)
        .await?;

    user_account.sync().await.unwrap();
    // Sync whirlpools before creating a client
    jet_client::state::margin_orca::sync(user_account.client().state()).await?;

    let tsol_change = tsol_balance - position_balance(&user_account, &tsol);
    let usdc_change = usdc_balance - position_balance(&user_account, &usdc);

    // Refresh positions
    // TODO: this is failing because a token_config is not found,
    // howeve we have registered the config, so unclear.
    // user_account.send_with_refresh(&[]).await?;

    let state = user_account.state();
    let margin_position = state
        .positions()
        .find(|p| p.adapter == jet_margin_orca::ID)
        .expect("Position not registered");
    // The position should have been added, and should have a balance and value
    assert_eq!(margin_position.balance, 1);
    let position_value = margin_position.value().as_f64();
    assert!(position_value > 900.0);
    assert!(position_value <= 1000.0);

    // Try remove all liquidity
    orca_client
        .remove_liquidity(&position, tsol_change, usdc_change)
        .await?;

    user_account.sync().await.unwrap();
    // Sync whirlpools before creating a client
    jet_client::state::margin_orca::sync(user_account.client().state()).await?;

    let state = user_account.state();
    let margin_position = state
        .positions()
        .find(|p| p.adapter == jet_margin_orca::ID)
        .expect("Position not registered");
    // The position should have been added, and should have a balance and value
    assert_eq!(margin_position.balance, 1);
    let position_value = margin_position.value().as_f64();
    // The position should be empty
    assert!(position_value < 0.001);

    // The other positions should have their values restored, minus some dust
    for position in state.positions() {
        if position.adapter == Pubkey::default() {
            assert!(position.value().as_f64() >= 999.90);
        }
    }

    // Close the position
    orca_client.close_position(position.position_mint).await?;

    // There should be a position with a balance of 0
    user_account.sync().await.unwrap();
    // Sync whirlpools before creating a client
    jet_client::state::margin_orca::sync(user_account.client().state()).await?;

    let state = user_account.state();
    let margin_position = state
        .positions()
        .find(|p| p.adapter == jet_margin_orca::ID)
        .expect("Position not registered");
    assert_eq!(margin_position.balance, 0);

    // Close the margin position
    orca_client.close_position_meta().await?;

    // There should be only 2 positions left
    user_account.sync().await.unwrap();
    assert_eq!(user_account.state().positions().count(), 2);
    Ok(())
}
