use std::time::SystemTime;

use jet_client::fixed_term::MarketInfo;
use jet_client::margin::MarginAccountClient;
use jet_client::JetClient;
use jet_instructions::fixed_term::derive;

use hosted_tests::environment::TestToken;
use hosted_tests::util::assert_program_error;

use hosted_tests::actions::*;
use hosted_tests::context::{TestContext, TestContextSetupInfo};

struct TestEnv {
    ctx: TestContext,
    usdc: Token,
    tsol: Token,
    _users: Vec<JetClient>,
    accounts: Vec<MarginAccountClient>,
    market: MarketInfo,
}

async fn setup_context(name: &str, tenor: u64) -> TestEnv {
    let setup_config = TestContextSetupInfo {
        is_restricted: false,
        tokens: vec![
            TestToken::new("TSOL").into(),
            TestToken {
                name: "USDC".to_string(),
                margin_pool: true,
                fixed_term_tenors: vec![tenor],
            }
            .into(),
        ],
        dexes: vec![("spl-swap", "TSOL/USDC")],
    };

    let ctx = TestContext::new(name, &setup_config).await.unwrap();

    // derive mints for tokens
    let usdc = Token::from_context(&ctx, "USDC");
    let tsol = Token::from_context(&ctx, "TSOL");

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

    // Get the market
    let market = users[0].fixed_term().markets().pop().unwrap();
    let ticket = Token {
        mint: market.ticket,
        decimals: 6,
    };

    // set token prices
    set_price(&ctx, &usdc, 1.0, 0.01).await;
    set_price(&ctx, &tsol, 10.0, 0.01).await;
    set_price(&ctx, &ticket, 1.0, 0.01).await;

    TestEnv {
        ctx,
        usdc,
        tsol,
        _users: users,
        accounts,
        market,
    }
}

macro_rules! setup_context {
    ($tenor:expr) => {{
        let name = hosted_tests::fn_name_and_try_num!();
        setup_context(&name, $tenor).await
    }};
}

#[tokio::test]
async fn can_place_orders() -> anyhow::Result<()> {
    let TestEnv {
        usdc,
        tsol,
        accounts,
        market,
        ..
    } = setup_context!(3600);

    // Deposit user funds into the margin account for collateral
    deposit(&accounts[0], &usdc, usdc.amount(1_000.0))
        .await
        .unwrap();
    deposit(&accounts[0], &tsol, tsol.amount(100_000.0))
        .await
        .unwrap();

    // Place a borrow order
    request_loan(&accounts[0], &market, usdc.amount(1_000.0), 5.0)
        .await
        .unwrap();

    // Place a lending order
    offer_loan(&accounts[0], &market, usdc.amount(1_000.0), 10.0)
        .await
        .unwrap();

    // Verify we can see both orders
    let orders = accounts[0].fixed_term(&market.address).unwrap().orders();

    assert_eq!(2, orders.len());
    assert_eq!(5_00, orders[0].rate());
    assert_eq!(10_00, orders[1].rate());

    Ok(())
}

#[tokio::test]
async fn lend_then_borrow_now() -> anyhow::Result<()> {
    let TestEnv {
        usdc,
        tsol,
        accounts,
        market,
        ..
    } = setup_context!(3600);

    // Deposit user funds into the margin account
    deposit(&accounts[0], &usdc, usdc.amount(1_000_000.0))
        .await
        .unwrap();
    deposit(&accounts[1], &tsol, tsol.amount(100_000.0))
        .await
        .unwrap();

    // user A lends in fixed-term market
    offer_loan(&accounts[0], &market, usdc.amount(1_000.0), 1.0)
        .await
        .unwrap();

    // user B borrows from A with immediate fill
    request_loan(&accounts[1], &market, usdc.amount(100_000.0), 100.0)
        .await
        .unwrap();

    // Verify the loan was disbursed
    let b_usdc_amount = position_balance(&accounts[1], &usdc);
    assert_eq!(usdc.amount(999.999999), b_usdc_amount);

    let loans = accounts[1].fixed_term(&market.address).unwrap().loans();
    assert_eq!(1, loans.len());
    assert_eq!(usdc.amount(1_000.001141), loans[0].balance);

    Ok(())
}

#[tokio::test]
async fn borrow_then_lend_now() -> anyhow::Result<()> {
    let TestEnv {
        usdc,
        tsol,
        accounts,
        market,
        ..
    } = setup_context!(3600);

    // Deposit user funds into the margin account
    deposit(&accounts[0], &usdc, usdc.amount(1_000_000.0))
        .await
        .unwrap();
    deposit(&accounts[1], &tsol, tsol.amount(100_000.0))
        .await
        .unwrap();

    // user B requests to borrow
    request_loan(&accounts[1], &market, usdc.amount(100_000.0), 1.0)
        .await
        .unwrap();

    // user A lends to B with immediate fill
    offer_loan(&accounts[0], &market, usdc.amount(1_000.0), 0.0)
        .await
        .unwrap();

    // Verify the term deposit
    let deposits = accounts[0].fixed_term(&market.address).unwrap().deposits();
    assert_eq!(1, deposits.len());
    assert_eq!(usdc.amount(1_000.001141), deposits[0].amount);

    Ok(())
}

#[tokio::test]
async fn simple_lend_borrow_repay() -> anyhow::Result<()> {
    let TestEnv {
        usdc,
        tsol,
        accounts,
        market,
        ..
    } = setup_context!(3600);

    // Deposit user funds into the margin account
    deposit(&accounts[0], &usdc, usdc.amount(1_000_000.0))
        .await
        .unwrap();
    deposit(&accounts[1], &tsol, tsol.amount(100_000.0))
        .await
        .unwrap();

    // user A lends in fixed-term market
    offer_loan(&accounts[0], &market, usdc.amount(1_000.0), 1.0)
        .await
        .unwrap();

    // user B borrows from A
    request_loan(&accounts[1], &market, usdc.amount(100_000.0), 1.0)
        .await
        .unwrap();

    // Verify the loan was disbursed
    let b_usdc_amount = position_balance(&accounts[1], &usdc);
    assert_eq!(usdc.amount(999.999999), b_usdc_amount);

    let loans = accounts[1].fixed_term(&market.address).unwrap().loans();
    assert_eq!(1, loans.len());
    assert_eq!(usdc.amount(1_000.001141), loans[0].balance);

    // user B repays their active loan
    deposit(&accounts[1], &usdc, usdc.amount(1.0))
        .await
        .unwrap();
    repay_term_loan(&accounts[1], &market, u64::MAX)
        .await
        .unwrap();

    Ok(())
}

#[tokio::test]
async fn borrow_fails_without_collateral() -> anyhow::Result<()> {
    let TestEnv {
        usdc,
        accounts,
        market,
        ..
    } = setup_context!(3600);

    let borrow_result = request_loan(&accounts[0], &market, usdc.amount(1_000.0), 1.0).await;

    assert_program_error(jet_margin::ErrorCode::Unhealthy, borrow_result);

    Ok(())
}

#[tokio::test]
async fn margin_can_sell_tickets() -> anyhow::Result<()> {
    let TestEnv {
        usdc,
        tsol,
        accounts,
        market,
        ..
    } = setup_context!(3600);

    // Deposit user funds into the margin account
    deposit(&accounts[0], &usdc, usdc.amount(1_000.0))
        .await
        .unwrap();
    deposit(&accounts[1], &usdc, usdc.amount(1_000_000.0))
        .await
        .unwrap();
    deposit(&accounts[1], &tsol, tsol.amount(100_000.0))
        .await
        .unwrap();

    // user B requests a loan
    request_loan(&accounts[1], &market, usdc.amount(100_000.0), 1.0)
        .await
        .unwrap();

    // user A lends in market without auto stake, should immediately fill from B's request
    offer_loan_no_auto_stake(&accounts[0], &market, usdc.amount(1_000.0), 1.0)
        .await
        .unwrap();

    // A should now have tickets in their account
    let ticket_balance = accounts[0].balance(&market.ticket);
    assert_eq!(1_000_001_141, ticket_balance);

    // B offers to lend at a higher rate
    offer_loan(&accounts[1], &market, usdc.amount(100_000.0), 2.0)
        .await
        .unwrap();

    // A can sell their tickets to B
    sell_tickets(&accounts[0], &market, ticket_balance, 0.999997)
        .await
        .unwrap();

    // A should now have USDC in their account
    assert_eq!(0, accounts[0].balance(&market.ticket));
    assert_eq!(
        usdc.amount(999.998857),
        position_balance(&accounts[0], &usdc)
    );

    Ok(())
}

#[tokio::test]
async fn can_cancel_orders() -> anyhow::Result<()> {
    let TestEnv {
        ctx,
        usdc,
        tsol,
        accounts,
        market,
        ..
    } = setup_context!(3600);

    // Deposit user funds into the margin account
    deposit(&accounts[0], &usdc, usdc.amount(1_000.0))
        .await
        .unwrap();
    deposit(&accounts[0], &tsol, tsol.amount(100_000.0))
        .await
        .unwrap();

    // user places an order to lend in the market
    offer_loan(&accounts[0], &market, usdc.amount(1_000.0), 5.0)
        .await
        .unwrap();

    // user places an order to borrow in the market
    request_loan(&accounts[0], &market, usdc.amount(100_000.0), 1.0)
        .await
        .unwrap();

    let orders = accounts[0].fixed_term(&market.address).unwrap().orders();
    let claims_token = Token {
        mint: derive::claims_mint(&market.address),
        decimals: usdc.decimals,
    };

    assert_eq!(2, orders.len());
    assert_eq!(0, position_balance(&accounts[0], &usdc));
    assert_eq!(
        usdc.amount(100_000.114156),
        position_balance(&accounts[0], &claims_token)
    );

    for order in orders {
        cancel_order(&accounts[0], &market, order.order_id)
            .await
            .unwrap();
    }

    consume_events(&ctx, &market).await;
    market_settle(&accounts[0], &market).await.unwrap();

    let orders = accounts[0].fixed_term(&market.address).unwrap().orders();
    assert_eq!(0, orders.len());
    assert_eq!(
        usdc.amount(999.999999),
        position_balance(&accounts[0], &usdc)
    );
    assert_eq!(0, position_balance(&accounts[0], &claims_token));

    Ok(())
}

#[tokio::test]
async fn can_redeem_deposit() -> anyhow::Result<()> {
    let TestEnv {
        ctx,
        usdc,
        tsol,
        accounts,
        market,
        ..
    } = setup_context!(0);

    // Deposit user funds into the margin account
    deposit(&accounts[0], &usdc, usdc.amount(1_000_000.0))
        .await
        .unwrap();
    deposit(&accounts[1], &tsol, tsol.amount(100_000.0))
        .await
        .unwrap();

    // user B requests to borrow
    request_loan(&accounts[1], &market, usdc.amount(100_000.0), 1.0)
        .await
        .unwrap();

    // user A lends to B with immediate fill
    offer_loan(&accounts[0], &market, usdc.amount(1_000.0), 0.0)
        .await
        .unwrap();

    // Advance time
    let mut clock = ctx.rpc().get_clock().await.unwrap();
    clock.unix_timestamp = 2 + SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    ctx.rpc().set_clock(clock).await.unwrap();

    // redeem deposit
    let balance_before_redemption = position_balance(&accounts[0], &usdc);
    redeem_term_deposits(&accounts[0], &market).await.unwrap();
    let balance_change = position_balance(&accounts[0], &usdc) - balance_before_redemption;

    assert_eq!(usdc.amount(1_000.0), balance_change);

    Ok(())
}
