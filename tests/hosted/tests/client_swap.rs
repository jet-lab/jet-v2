use std::collections::HashMap;

use jet_client::{margin::MarginAccountClient, state::dexes::DexState, swaps::SwapStep};

use hosted_tests::{
    actions::*,
    context::{TestContext, TestContextSetupInfo},
    environment::TestToken,
};
use jet_margin_sdk::swap::openbook_swap::OpenBookMarket;
use jet_program_common::programs::OPENBOOK;

struct TestEnv {
    ctx: TestContext,
    usdc: Token,
    tsol: Token,
    tbtc: Token,
    margin_accounts: Vec<MarginAccountClient>,
}

async fn setup_context(
    name: &str,
    dexes: impl IntoIterator<Item = (&'static str, &'static str)>,
) -> TestEnv {
    let setup_config = TestContextSetupInfo {
        is_restricted: false,
        tokens: vec![
            TestToken::new("TSOL").into(),
            TestToken::new("MSOL").into(),
            TestToken::new("USDC").into(),
            TestToken::new("USDT").into(),
            TestToken::new("TBTC").into(),
        ],
        dexes: dexes.into_iter().collect(),
        whirlpools: vec![],
    };

    let ctx = TestContext::new(name, &setup_config).await.unwrap();

    // derive mints for tokens
    let usdc = Token::from_context(&ctx, "USDC");
    let tsol = Token::from_context(&ctx, "TSOL");
    let tbtc = Token::from_context(&ctx, "TBTC");

    // create users
    let users = vec![
        ctx.create_user().await.unwrap(),
        ctx.create_user().await.unwrap(),
        ctx.create_user().await.unwrap(),
    ];

    // sync the client state
    for user in &users {
        user.state().sync_all().await.unwrap();
    }

    // Get the client for each user's account
    let accounts = users
        .iter()
        .map(|c| c.margin().accounts()[0].clone())
        .collect::<Vec<_>>();

    // setup some base user funds for each wallet
    for user in &users {
        airdrop(user, &usdc, usdc.amount(10_000_000.0)).await;
        airdrop(user, &tsol, tsol.amount(1_000_000.0)).await;
        airdrop(user, &tbtc, tsol.amount(1_000.0)).await;
    }

    // set token prices
    set_price(&ctx, &usdc, 1.0, 0.01).await;
    set_price(&ctx, &tsol, 10.0, 0.01).await;
    set_price(&ctx, &tbtc, 30000.0, 0.01).await;

    TestEnv {
        ctx,
        usdc,
        tsol,
        tbtc,
        margin_accounts: accounts,
    }
}

macro_rules! setup_context {
    ($dexes:expr) => {{
        let name = hosted_tests::fn_name_and_try_num!();
        setup_context(&name, $dexes).await
    }};
}

#[tokio::test]
async fn can_openbook_swap() -> anyhow::Result<()> {
    let TestEnv {
        ctx,
        usdc,
        tsol,
        tbtc,
        margin_accounts,
        ..
    } = setup_context!([("openbook", "TSOL/USDC")]);

    let supported_mints = [usdc.mint, tsol.mint, tbtc.mint].into_iter().collect();
    let markets = OpenBookMarket::get_markets(ctx.rpc(), &supported_mints, anchor_spl::dex::id())
        .await
        .unwrap();

    for market in markets.values() {
        jet_testing::openbook::market_make(
            ctx.inner.payer(),
            ctx.inner.solana.rpc2.as_ref(),
            ctx.config.airspaces[0].lookup_registry_authority.unwrap(),
            market.market,
            anchor_spl::dex::id(),
        )
        .await
        .unwrap();
    }

    let deposit_amount = tsol.amount(3_100.0);
    let margin_account = &margin_accounts[0];

    deposit(margin_account, &tsol, deposit_amount)
        .await
        .unwrap();

    // Swap TSOL for TBTC
    let markets = margin_accounts[0]
        .client()
        .state()
        .get_all::<DexState>()
        .into_iter()
        .map(|(state_addr, ds)| (ds.token_a, state_addr))
        .collect::<HashMap<_, _>>();

    let swap_steps = [SwapStep {
        from_token: tsol.mint,
        to_token: usdc.mint,
        program: OPENBOOK,
        swap_pool: markets.get(&tsol.mint).cloned().unwrap(),
    }];

    margin_account.update_lookup_tables().await.unwrap();
    margin_account
        .swaps()
        .route_swap(&swap_steps, deposit_amount, 1)
        .await
        .unwrap();
    margin_account.sync().await.unwrap();

    // FIXME: wrong balance
    //assert!(balance > usdc.amount(30_000.0));

    Ok(())
}
