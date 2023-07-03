use jet_client::{margin::MarginAccountClient, state::margin_pool::MarginPoolCacheExt, JetClient};

use hosted_tests::{
    actions::*,
    context::{TestContext, TestContextSetupInfo},
    environment::TestToken,
    test_context,
    util::assert_program_error,
};
use jet_margin_sdk::swap::openbook_swap::OpenBookMarket;

struct TestEnv {
    ctx: TestContext,
    usdc: Token,
    tsol: Token,
    msol: Token,
    usdt: Token,
    tbtc: Token,
    _users: Vec<JetClient>,
    accounts: Vec<MarginAccountClient>,
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
    };

    let ctx = TestContext::new(name, &setup_config).await.unwrap();

    // derive mints for tokens
    let usdc = Token::from_context(&ctx, "USDC");
    let tsol = Token::from_context(&ctx, "TSOL");
    let usdt = Token::from_context(&ctx, "USDT");
    let msol = Token::from_context(&ctx, "MSOL");
    let tbtc = Token::from_context(&ctx, "TBTC");

    let mut lookup_addresses = vec![
        jet_margin_sdk::jet_airspace::ID,
        jet_margin_sdk::jet_margin::ID,
        jet_margin_sdk::jet_margin_pool::ID,
        jet_margin_sdk::jet_fixed_term::ID,
        spl_token::ID,
        usdc.mint,
        tsol.mint,
    ];

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
    }

    // set token prices
    set_price(&ctx, &usdc, 1.0, 0.01).await;
    set_price(&ctx, &tsol, 10.0, 0.01).await;
    set_price(&ctx, &tbtc, 30000.0, 0.01).await;

    TestEnv {
        ctx,
        usdc,
        tsol,
        msol,
        usdt,
        tbtc,
        _users: users,
        accounts,
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
        accounts,
        ..
    } = setup_context!([("openbook", "TBTC/USDC"), ("openbook", "TSOL/USDC")]);

    let supported_mints = [usdc.mint, tsol.mint, tbtc.mint].into_iter().collect();
    let markets = OpenBookMarket::get_markets(&ctx.rpc(), &supported_mints, anchor_spl::dex::id())
        .await
        .unwrap();

    for (_, market) in &markets {
        jet_testing::openbook::market_make(
            &ctx.inner.payer(),
            ctx.inner.solana.rpc2.as_ref(),
            ctx.config.airspaces[0].lookup_registry_authority.unwrap(),
            market.market,
            anchor_spl::dex::id(),
        )
        .await
        .unwrap();
    }

    Ok(())
}
