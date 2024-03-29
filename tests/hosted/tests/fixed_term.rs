use std::sync::Arc;

use agnostic_orderbook::state::event_queue::EventRef;
use anchor_lang::AccountDeserialize;
use anyhow::Result;
use futures::{future::join_all, join};
use hosted_tests::{
    context::MarginTestContext,
    fixed_term::{
        create_and_fund_fixed_term_market_margin_user, FixedTermUser, GenerateProxy, OrderAmount,
        TestManager as FixedTermTestManager, MIN_ORDER_SIZE, STARTING_TOKENS,
    },
    margin_test_context,
    setup_helper::{setup_user, tokens},
    test_default,
};
use jet_fixed_term::{
    margin::state::{BorrowAutoRollConfig, LendAutoRollConfig, TermLoan},
    orderbook::state::{
        CallbackFlags, MarginCallbackInfo, OrderParams, RoundingAction, SensibleOrderSummary,
    },
    tickets::state::TermDeposit,
};
use jet_margin_sdk::{
    fixed_term::{auto_roll_servicer::AutoRollServicer, settler::SETTLES_PER_TX},
    margin_integrator::{NoProxy, Proxy},
    solana::transaction::{
        InverseSendTransactionBuilder, SendTransactionBuilder, TransactionBuilderExt, WithSigner,
    },
    tx_builder::invoke_into::{InvokeEachInto, InvokeInto},
};
use jet_margin_sdk::{margin_integrator::RefreshingProxy, refresh::canonical_position_refresher};
use jet_program_common::{
    interest_pricing::{InterestPricer, PricerImpl},
    Fp32,
};
use jet_solana_client::{rpc::AccountFilter, transactions, util::keypair::KeypairExt};

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn non_margin_orders() -> Result<(), anyhow::Error> {
    let ctx = margin_test_context!();
    let manager = Arc::new(FixedTermTestManager::full(&ctx).await.unwrap());
    non_margin_orders_for_proxy::<NoProxy>(ctx, manager).await
}

async fn non_margin_orders_for_proxy<P: Proxy + GenerateProxy>(
    ctx: Arc<MarginTestContext>,
    manager: Arc<FixedTermTestManager>,
) -> Result<()> {
    let alice = FixedTermUser::<P>::generate_funded(ctx.clone(), manager.clone()).await?;

    const START_TICKETS: u64 = 1_000_000;
    alice.convert_tokens(START_TICKETS).await?;

    assert_eq!(alice.tickets().await?, START_TICKETS);
    assert_eq!(alice.tokens().await?, STARTING_TOKENS - START_TICKETS);
    assert_eq!(
        manager.load_manager_token_vault().await?.amount,
        START_TICKETS
    );

    const STAKE_AMOUNT: u64 = 10_000;
    let ticket_seed = vec![];

    alice.stake_tokens(STAKE_AMOUNT, &ticket_seed).await?;
    assert_eq!(alice.tickets().await?, START_TICKETS - STAKE_AMOUNT);

    let deposit = alice
        .load_term_deposit(jet_fixed_term::seeds::USER.as_ref())
        .await?;
    assert_eq!(deposit.amount, STAKE_AMOUNT);
    assert_eq!(deposit.market, manager.ix_builder.market());
    assert_eq!(deposit.owner, alice.proxy.pubkey());

    manager.pause_ticket_redemption().await?;
    let market = manager.load_market().await?;

    assert!(market.tickets_paused.as_bool());
    assert!(alice.redeem_claim_ticket(&ticket_seed).await.is_err());

    manager.resume_ticket_redemption().await?;

    let market = manager.load_market().await?;
    assert!(!market.tickets_paused.as_bool());

    // Scenario a: post a borrow order to an empty book
    let a_amount = OrderAmount::from_quote_amount_rate(1_000, 2_000);
    let a_params = OrderParams {
        max_ticket_qty: a_amount.base,
        max_underlying_token_qty: a_amount.quote,
        limit_price: a_amount.price,
        match_limit: 100,
        post_only: false,
        post_allowed: true,
        auto_stake: true,
        auto_roll: false,
    };
    let order_a_expected_base_to_post = quote_to_base(1_000, 2_000);

    // simulate
    let summary_a = manager
        .simulate_new_order(a_params, agnostic_orderbook::state::Side::Ask)
        .await?;
    assert!(summary_a.posted_order_id.is_some());
    assert_eq!(summary_a.total_base_qty, order_a_expected_base_to_post);
    assert_eq!(
        summary_a.total_quote_qty,
        Fp32::upcast_fp32(a_params.limit_price)
            .decimal_u64_mul(order_a_expected_base_to_post)
            .unwrap()
    );
    assert_eq!(
        summary_a.total_base_qty_posted,
        order_a_expected_base_to_post
    );

    // send to validator
    alice.sell_tickets_order(a_params).await?;

    assert_eq!(
        alice.tickets().await?,
        START_TICKETS - STAKE_AMOUNT - order_a_expected_base_to_post
    );

    let borrow_order = manager.load_orderbook().await?.asks()?[0];

    assert_eq!(borrow_order.price(), a_amount.price);
    assert_eq!(borrow_order.base_quantity, order_a_expected_base_to_post);
    // quote amounts of the post are a result of an fp32 mul, so we cannot directly compare
    assert_eq!(
        Fp32::upcast_fp32(borrow_order.price())
            .decimal_u64_mul(borrow_order.base_quantity)
            .unwrap(),
        Fp32::upcast_fp32(a_amount.price)
            .decimal_u64_mul(order_a_expected_base_to_post)
            .unwrap()
    );

    // Cannot self trade
    let crossing_amount = OrderAmount::from_quote_amount_rate(500, 1_500);
    let crossing_params = OrderParams {
        max_ticket_qty: crossing_amount.base,
        max_underlying_token_qty: crossing_amount.quote,
        limit_price: crossing_amount.price,
        match_limit: 100,
        post_only: false,
        post_allowed: true,
        auto_stake: true,
        auto_roll: false,
    };
    assert!(alice.lend_order(crossing_params, &[]).await.is_err());

    // Scenario b: post a lend order that partially fills the borrow order and does not post remaining
    let b_amount = OrderAmount::from_quote_amount_rate(500, 1_500);
    let b_params = OrderParams {
        max_ticket_qty: b_amount.base,
        max_underlying_token_qty: b_amount.quote,
        limit_price: b_amount.price,
        match_limit: 100,
        post_only: false,
        post_allowed: true,
        auto_stake: true,
        auto_roll: false,
    };
    let order_b_expected_base_to_fill = quote_to_base(500, 2000);

    // simulate
    let summary_b = manager
        .simulate_new_order(b_params, agnostic_orderbook::state::Side::Bid)
        .await?;
    assert!(summary_b.posted_order_id.is_none());
    assert_eq!(summary_b.total_base_qty, order_b_expected_base_to_fill);
    assert_eq!(summary_b.total_quote_qty, 500);

    // send to validator
    let bob = FixedTermUser::<P>::generate_funded(ctx.clone(), manager.clone()).await?;
    bob.lend_order(b_params, &[0]).await?;

    assert_eq!(
        bob.tokens().await?,
        STARTING_TOKENS - summary_b.total_quote_qty
    );

    // scenario c: post a lend order that fills the remaining borrow and makes a new post with the remaining
    let c_amount = OrderAmount::from_quote_amount_rate(1_500, 1_500);
    let c_params = OrderParams {
        max_ticket_qty: c_amount.base,
        max_underlying_token_qty: c_amount.quote,
        limit_price: c_amount.price,
        match_limit: 100,
        post_only: false,
        post_allowed: true,
        auto_stake: true,
        auto_roll: false,
    };

    // simulate
    let summary_c = manager
        .simulate_new_order(c_params, agnostic_orderbook::state::Side::Bid)
        .await?;

    let existing_order = manager.load_orderbook().await?.asks()?[0];

    let trade_price = Fp32::upcast_fp32(existing_order.price());
    let base_trade_qty = existing_order
        .base_quantity
        .min(c_params.max_ticket_qty)
        .min(
            (Fp32::from(c_params.max_underlying_token_qty) / trade_price)
                .as_decimal_u64()
                .unwrap(),
        );
    let quote_maker_qty = (trade_price * base_trade_qty)
        .as_decimal_u64_ceil()
        .unwrap();

    let base_qty_to_post = std::cmp::min(
        (Fp32::from(c_params.max_underlying_token_qty - quote_maker_qty)
            / Fp32::upcast_fp32(c_params.limit_price))
        .as_decimal_u64()
        .unwrap_or(u64::MAX),
        c_params.max_ticket_qty - base_trade_qty,
    );
    let quote_qty_to_post = (Fp32::upcast_fp32(c_params.limit_price) * base_qty_to_post)
        .as_decimal_u64_ceil()
        .unwrap();

    assert!(summary_c.posted_order_id.is_some());
    assert_eq!(
        summary_c.total_base_qty,
        existing_order.base_quantity + base_qty_to_post
    );
    assert_eq!(
        summary_c.total_quote_qty,
        quote_maker_qty + quote_qty_to_post
    );
    assert_eq!(summary_c.total_base_qty_posted, base_qty_to_post);

    // run on validator
    bob.lend_order(c_params, &[1]).await?;

    // TODO: assertions
    let _split_ticket_c = bob.load_term_deposit(&[1]).await?;

    assert_eq!(
        bob.tokens().await?,
        STARTING_TOKENS - summary_b.total_quote_qty - summary_c.total_quote_qty
    );

    manager.consume_events().await?;

    assert!(manager
        .load_event_queue()
        .await?
        .inner()?
        .iter()
        .next()
        .is_none());

    // order cancelling
    let order = manager.load_orderbook().await?.bids()?[0];
    bob.cancel_order(order.key).await?;

    let mut eq = manager.load_event_queue().await?;
    let local_eq = eq.inner()?;
    let cancel_event = match local_eq.iter().next().unwrap() {
        EventRef::Out(out) => out,
        _ => panic!("expected an out event"),
    };

    manager.consume_events().await?;

    assert!(manager.load_orderbook().await?.bids()?.first().is_none());
    assert_eq!(order.base_quantity, cancel_event.event.base_size);
    assert_eq!(cancel_event.callback_info.owner(), bob.proxy.pubkey());
    assert_eq!(cancel_event.event.order_id, order.key);

    // test order pausing
    manager.pause_orders().await?;

    alice.sell_tickets_order(a_params).await?;
    bob.lend_order(b_params, &[2]).await?;

    assert!(manager
        .load_event_queue()
        .await?
        .inner()?
        .iter()
        .next()
        .is_none());

    manager.resume_orders().await?;

    assert!(manager
        .load_event_queue()
        .await?
        .inner()?
        .iter()
        .next()
        .is_some());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial]
async fn margin_repay() -> Result<()> {
    let ctx = margin_test_context!();
    let manager = Arc::new(FixedTermTestManager::full(&ctx).await.unwrap());
    let client = manager.client.clone();
    let ([collateral], _, pricer) = tokens(&ctx).await.unwrap();

    // set up user
    let user = setup_user(&ctx, vec![(collateral, 0, u64::MAX / 2)])
        .await
        .unwrap();
    let margin = user.user.tx.ix.clone();
    let wallet = user.user.signer;

    // set up proxy
    let proxy = RefreshingProxy {
        proxy: margin.clone(),
        refresher: canonical_position_refresher(client.clone()).for_address(margin.address),
    };

    // set a lend order on the book
    let lender = FixedTermUser::<NoProxy>::generate_funded(ctx.clone(), manager.clone()).await?;
    let lend_params = OrderAmount::params_from_quote_amount_rate(500, 1_500);

    lender.lend_order(lend_params, &[]).await?;
    let posted_lend = manager.load_orderbook().await?.bids()?[0];

    let user = FixedTermUser::new_funded(manager.clone(), wallet, proxy.clone())
        .await
        .unwrap();
    user.initialize_margin_user().await.unwrap();

    let margin_user = user.load_margin_user().await.unwrap();
    assert_eq!(margin_user.market, manager.ix_builder.market());

    // place a borrow order
    let borrow_params = OrderAmount::params_from_quote_amount_rate(1_000, 2_000);
    let mut ixs = vec![
        pricer.set_oracle_price_tx(&collateral, 1.0).await.unwrap(),
        pricer
            .set_oracle_price_tx(&manager.ix_builder.ticket_mint(), 1.0)
            .await
            .unwrap(),
        pricer
            .set_oracle_price_tx(&manager.ix_builder.token_mint(), 1.0)
            .await
            .unwrap(),
    ];
    ixs.extend(
        user.refresh_and_margin_borrow_order(borrow_params)
            .await
            .unwrap(),
    );
    client
        .send_and_confirm_condensed_in_order(ixs)
        .await
        .unwrap();

    let term_loan = user.load_term_loan(0).await?;
    assert_eq!(
        term_loan.margin_user,
        manager.ix_builder.margin_user_account(user.proxy.pubkey())
    );
    assert_eq!(term_loan.balance, posted_lend.base_quantity);

    let margin_user = user.load_margin_user().await.unwrap();
    let posted_order = manager.load_orderbook().await?.asks()?[0];
    assert_eq!(margin_user.debt().pending(), posted_order.base_quantity,);
    assert_eq!(
        margin_user.debt().total(),
        posted_order.base_quantity + term_loan.balance
    );

    // user.settle().await?;
    // let assets = user.load_margin_user().await?.assets;
    // assert_eq!(assets.entitled_tickets + assets.entitled_tokens, 0);
    // TODO: assert balances on claims and user wallet

    let pre_repayment_term_loan = user.load_term_loan(0).await?;
    let pre_repayment_user = user.load_margin_user().await?;
    let repayment = 400;
    user.repay(0, repayment).await?;

    let post_repayment_term_loan = user.load_term_loan(0).await?;
    let post_repayment_user = user.load_margin_user().await?;
    assert_eq!(
        pre_repayment_term_loan.balance - repayment,
        post_repayment_term_loan.balance
    );
    assert_eq!(
        pre_repayment_user.debt().committed() - repayment,
        post_repayment_user.debt().committed()
    );

    user.repay(0, post_repayment_term_loan.balance).await?;

    let margin_user = user.load_margin_user().await?;
    assert_eq!(margin_user.debt().total(), margin_user.debt().pending());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial]
async fn can_consume_lots_of_events() -> Result<()> {
    let ctx = margin_test_context!();
    let manager = Arc::new(FixedTermTestManager::full(&ctx).await.unwrap());

    // make and fund users
    let alice = FixedTermUser::<NoProxy>::generate_funded(ctx.clone(), manager.clone()).await?;
    let bob = FixedTermUser::<NoProxy>::generate_funded(ctx.clone(), manager.clone()).await?;
    alice.convert_tokens(1_000_000).await?;

    let borrow_params = OrderAmount::params_from_quote_amount_rate(1_000, 1_000);
    let lend_params = OrderAmount::params_from_quote_amount_rate(1_000, 900);

    for i in 0..100 {
        alice.sell_tickets_order(borrow_params).await?;
        bob.lend_order(lend_params, &[i]).await?;
    }

    manager.consume_events().await?;
    // these are not margin users so there are no expected accounts to settle
    manager
        .expect_and_execute_settlement::<NoProxy>(&[])
        .await?;

    assert!(manager.load_event_queue().await?.is_empty()?);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial]
async fn settle_many_margin_accounts() -> Result<()> {
    let ctx = margin_test_context!();
    let manager = Arc::new(FixedTermTestManager::full(&ctx).await.unwrap());
    let client = manager.client.clone();
    let ([collateral], _, pricer) = tokens(&ctx).await.unwrap();
    let set_prices = vec![
        pricer.set_oracle_price_tx(&collateral, 1.0).await.unwrap(),
        pricer
            .set_oracle_price_tx(&manager.ix_builder.ticket_mint(), 1.0)
            .await
            .unwrap(),
        pricer
            .set_oracle_price_tx(&manager.ix_builder.token_mint(), 1.0)
            .await
            .unwrap(),
    ]
    .send_and_confirm_condensed(&client);

    let mut trades = vec![];

    // TODO: find exact value
    let n_trades = SETTLES_PER_TX;

    for _ in 0..n_trades {
        trades.push(async {
            let (lender, borrower) = join!(
                create_and_fund_fixed_term_market_margin_user(&ctx, manager.clone(), vec![]),
                create_and_fund_fixed_term_market_margin_user(
                    &ctx,
                    manager.clone(),
                    vec![(collateral, 0, u64::MAX / 1_000)],
                )
            );
            transactions! {
                lender.proxy.refresh().await.unwrap(),
                borrower.proxy.refresh().await.unwrap(),
                lender.margin_lend_order(underlying(1_001, 2_000)).await.unwrap(),
                borrower.margin_borrow_order(underlying(1_000, 2_000)).await.unwrap()
            }
            .send_and_confirm_condensed_in_order(&client)
            .await
            .unwrap();

            lender
        });
    }

    set_prices.await.unwrap();
    let users_to_settle = join_all(trades).await;

    manager.consume_events().await?;
    manager
        .expect_and_execute_settlement(&users_to_settle.iter().collect::<Vec<_>>())
        .await?;

    assert!(manager.load_event_queue().await?.is_empty()?);

    Ok(())
}

#[cfg_attr(feature = "localnet", ignore = "does not run on localnet")]
#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial]
async fn auto_roll_many_trades() -> Result<()> {
    let ctx = margin_test_context!();
    let manager = Arc::new(FixedTermTestManager::full(&ctx).await.unwrap());
    let client = manager.client.clone();
    let ([collateral], _, pricer) = tokens(&ctx).await.unwrap();
    let set_prices = vec![
        pricer.set_oracle_price_tx(&collateral, 1.0).await.unwrap(),
        pricer
            .set_oracle_price_tx(&manager.ix_builder.ticket_mint(), 1.0)
            .await
            .unwrap(),
        pricer
            .set_oracle_price_tx(&manager.ix_builder.token_mint(), 1.0)
            .await
            .unwrap(),
    ]
    .send_and_confirm_condensed(&client);

    let (lender, borrower) = join!(
        create_and_fund_fixed_term_market_margin_user(&ctx, manager.clone(), vec![]),
        create_and_fund_fixed_term_market_margin_user(
            &ctx,
            manager.clone(),
            vec![(collateral, 0, u64::MAX / 1_000)],
        ),
    );
    let lend_params = OrderParams {
        auto_roll: true,
        ..underlying(1_001, 2_000)
    };
    let borrow_params = OrderParams {
        auto_roll: true,
        ..underlying(1_000, 2_000)
    };
    lender
        .set_lend_roll_config(LendAutoRollConfig {
            limit_price: OrderAmount::rate_to_price(1_000),
        })
        .await
        .unwrap();
    borrower
        .set_borrow_roll_config(BorrowAutoRollConfig {
            limit_price: OrderAmount::rate_to_price(10_000),
            roll_tenor: 1,
        })
        .await
        .unwrap();

    let mut trades = vec![];

    // TODO: find exact value
    let n_trades = SETTLES_PER_TX;
    for _ in 0..n_trades {
        trades.push(async {
            transactions! {
                lender.proxy.refresh().await.unwrap(),
                borrower.proxy.refresh().await.unwrap(),
                lender.margin_lend_order(lend_params).await.unwrap(),
                borrower.margin_borrow_order(borrow_params).await.unwrap()
            }
            .send_and_confirm_condensed_in_order(&client)
            .await
        });
    }
    set_prices.await.unwrap();
    join_all(trades)
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
    manager.consume_events().await?;

    #[cfg(not(feature = "localnet"))]
    {
        let mut clock = manager.client.get_clock().await?;
        clock.unix_timestamp += 1;
        manager.client.set_clock(clock).await?;
    }
    // #[cfg(feature = "localnet")]
    // {
    //     std::thread::sleep(std::time::Duration::from_secs(1 as u64));
    // }

    // provide some liquidity
    transactions! {
        lender.proxy.refresh().await.unwrap(),
        borrower.proxy.refresh().await.unwrap(),
        lender.margin_lend_order(underlying(10_000, 3_000)).await.unwrap(),
        borrower.margin_borrow_order(underlying(10_000, 2_000)).await.unwrap()
    }
    .send_and_confirm_condensed_in_order(&client)
    .await?;

    let servicer = AutoRollServicer::new(
        manager.client.clone(),
        manager.ix_builder.clone(),
        MIN_ORDER_SIZE,
    );
    servicer.service_all().await;

    #[cfg(not(feature = "localnet"))]
    {
        let mut clock = manager.client.get_clock().await?;
        clock.unix_timestamp += hosted_tests::fixed_term::LEND_TENOR as i64;
        manager.client.set_clock(clock).await?;
    }
    // #[cfg(feature = "localnet")]
    // {
    //     std::thread::sleep(std::time::Duration::from_secs(hosted_tests::fixed_term::LEND_TENOR as u64));
    // }
    servicer.service_all().await;

    let mut loans = manager
        .client
        .get_program_accounts(
            &jet_fixed_term::ID,
            vec![AccountFilter::DataSize(std::mem::size_of::<TermLoan>() + 8)],
        )
        .await?
        .into_iter()
        .filter_map(
            |(_, a)| match TermLoan::try_deserialize(&mut a.data.as_ref()) {
                Err(_) => None,
                Ok(loan) => Some(loan),
            },
        )
        .collect::<Vec<_>>();
    loans.sort_by(|a, b| a.sequence_number.partial_cmp(&b.sequence_number).unwrap());

    let mut deposits = manager
        .client
        .get_program_accounts(
            &jet_fixed_term::ID,
            vec![AccountFilter::DataSize(
                std::mem::size_of::<TermDeposit>() + 8,
            )],
        )
        .await?
        .into_iter()
        .filter_map(
            |(_, a)| match TermDeposit::try_deserialize(&mut a.data.as_ref()) {
                Err(_) => None,
                Ok(deposit) => Some(deposit),
            },
        )
        .collect::<Vec<_>>();
    deposits.sort_by(|a, b| a.sequence_number.partial_cmp(&b.sequence_number).unwrap());

    // should fully roll, keeping total number of accounts
    assert_eq!(loans.len(), 3);
    assert_eq!(deposits.len(), 3);

    // a total of 3 rolls occurred, leaving the total number of loans/deposits created at 6 per account
    assert_eq!(deposits.last().unwrap().sequence_number, 5);
    assert_eq!(loans.last().unwrap().sequence_number, 5);

    // FIXME: exact numbers should be tested against W.R.T. principal and interest
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn margin_borrow() -> Result<()> {
    let ctx = margin_test_context!();
    let manager = Arc::new(FixedTermTestManager::full(&ctx).await.unwrap());
    let client = manager.client.clone();
    let ([collateral], _, pricer) = tokens(&ctx).await.unwrap();

    let user = create_and_fund_fixed_term_market_margin_user(
        &ctx,
        manager.clone(),
        vec![(collateral, 0, u64::MAX / 2)],
    )
    .await;

    transactions! {
        pricer.set_oracle_price_tx(&collateral, 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.ticket_mint(), 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.token_mint(), 1.0).await?,
        user.refresh_and_margin_borrow_order(underlying(1_000, 2_000)).await?,
    }
    .send_and_confirm_condensed_in_order(&client)
    .await?;

    assert_eq!(STARTING_TOKENS, user.tokens().await?);
    assert_eq!(0, user.tickets().await?);
    assert_eq!(1_000, user.underlying_collateral().await?);
    assert_eq!(1_201, user.claims().await?);

    let margin_user = user.load_margin_user().await.unwrap();
    let posted_order = manager.load_orderbook().await?.asks()?[0];
    assert_eq!(margin_user.debt().total(), posted_order.base_quantity,);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn margin_borrow_fails_without_collateral() -> Result<()> {
    let ctx = margin_test_context!();
    let manager = Arc::new(FixedTermTestManager::full(&ctx).await.unwrap());
    let client = manager.client.clone();
    let ([collateral], _, pricer) = tokens(&ctx).await.unwrap();

    let user = create_and_fund_fixed_term_market_margin_user(&ctx, manager.clone(), vec![]).await;

    let result = transactions! {
        pricer.set_oracle_price_tx(&collateral, 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.ticket_mint(), 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.token_mint(), 1.0).await?,
        user.refresh_and_margin_borrow_order(underlying(1_000, 2_000)).await?,
    }
    .send_and_confirm_condensed_in_order(&client)
    .await;

    assert!(result.is_err());

    #[cfg(feature = "localnet")] // sim can't rollback
    {
        assert_eq!(STARTING_TOKENS, user.tokens().await?);
        assert_eq!(0, user.tickets().await?);
        assert_eq!(0, user.ticket_collateral().await?);
        assert_eq!(0, user.claims().await?);
        let asks = manager.load_orderbook().await?.asks()?;
        assert_eq!(0, asks.len());
        let margin_user = user.load_margin_user().await.unwrap();
        assert_eq!(0, margin_user.debt().total());
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn margin_lend() -> Result<()> {
    let ctx = margin_test_context!();
    let manager = Arc::new(FixedTermTestManager::full(&ctx).await.unwrap());
    let client = manager.client.clone();
    let ([collateral], _, pricer) = tokens(&ctx).await.unwrap();

    let user = create_and_fund_fixed_term_market_margin_user(&ctx, manager.clone(), vec![]).await;

    transactions! {
        pricer.set_oracle_price_tx(&collateral, 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.ticket_mint(), 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.token_mint(), 1.0).await?,
        user.refresh_and_margin_lend_order(underlying(1_000, 2_000)).await?,
    }
    .send_and_confirm_condensed_in_order(&client)
    .await?;

    assert_eq!(STARTING_TOKENS - 1_000, user.tokens().await?);
    assert_eq!(0, user.tickets().await?);
    assert_eq!(1_200, user.ticket_collateral().await?);
    assert_eq!(0, user.claims().await?);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn margin_borrow_then_margin_lend() -> Result<()> {
    let ctx = margin_test_context!();
    let manager = Arc::new(FixedTermTestManager::full(&ctx).await.unwrap());
    let client = manager.client.clone();
    let ([collateral], _, pricer) = tokens(&ctx).await.unwrap();

    let borrower = create_and_fund_fixed_term_market_margin_user(
        &ctx,
        manager.clone(),
        vec![(collateral, 0, u64::MAX / 2)],
    )
    .await;
    let mint = manager.ix_builder.token_mint();

    let lender = create_and_fund_fixed_term_market_margin_user(&ctx, manager.clone(), vec![]).await;

    transactions! {
        pricer.set_oracle_price_tx(&collateral, 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.ticket_mint(), 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.token_mint(), 1.0).await?,
        borrower.refresh_and_margin_borrow_order(underlying(1_000, 2_000)).await?,
    }
    .send_and_confirm_condensed_in_order(&client)
    .await?;

    assert_eq!(STARTING_TOKENS, borrower.tokens().await?);
    assert_eq!(0, borrower.tickets().await?);
    assert_eq!(1_000, borrower.underlying_collateral().await?);
    assert_eq!(1_201, borrower.claims().await?);
    // No tokens have been disbursed, so this should be 0
    assert_eq!(0, manager.collected_fees().await?);

    lender
        .refresh_and_margin_lend_order(underlying(1_001, 2_000))
        .await?
        .send_and_confirm_condensed_in_order(&client)
        .await?;

    assert_eq!(STARTING_TOKENS - 1_001, lender.tokens().await?);
    assert_eq!(0, lender.tickets().await?);
    assert_eq!(1_201, lender.ticket_collateral().await?);
    assert_eq!(0, lender.claims().await?);

    manager.consume_events().await?;
    borrower
        .proxy
        .proxy
        .create_deposit_position(mint)
        .with_signer(&borrower.owner)
        .send_and_confirm(&ctx.rpc())
        .await?;
    manager.expect_and_execute_settlement(&[&borrower]).await?;

    assert_eq!(STARTING_TOKENS + 999, borrower.tokens().await?);
    assert_eq!(0, borrower.tickets().await?);
    assert_eq!(0, borrower.ticket_collateral().await?);
    assert_eq!(1_201, borrower.claims().await?);

    assert_eq!(STARTING_TOKENS - 1_001, lender.tokens().await?);
    assert_eq!(0, lender.tickets().await?);
    assert_eq!(1_201, lender.ticket_collateral().await?);
    assert_eq!(0, lender.claims().await?);

    // FIXME: an exact number would be nice
    assert!(manager.collected_fees().await? > 0);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn margin_lend_then_margin_borrow() -> Result<()> {
    let ctx = margin_test_context!();
    let manager = Arc::new(FixedTermTestManager::full(&ctx).await.unwrap());
    let client = manager.client.clone();
    let ([collateral], _, pricer) = tokens(&ctx).await.unwrap();

    let borrower = create_and_fund_fixed_term_market_margin_user(
        &ctx,
        manager.clone(),
        vec![(collateral, 0, u64::MAX / 2)],
    )
    .await;
    let lender = create_and_fund_fixed_term_market_margin_user(&ctx, manager.clone(), vec![]).await;

    let lend_params = underlying(1_001, 2_000);
    transactions! {
        pricer.set_oracle_price_tx(&collateral, 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.ticket_mint(), 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.token_mint(), 1.0).await?,
        lender.refresh_and_margin_lend_order(lend_params).await?
    }
    .send_and_confirm_condensed_in_order(&client)
    .await?;

    assert_eq!(STARTING_TOKENS - 1_001, lender.tokens().await?);
    assert_eq!(0, lender.tickets().await?);
    assert_eq!(1_201, lender.ticket_collateral().await?);
    assert_eq!(0, lender.claims().await?);

    let borrow_params = underlying(1_000, 2_000);
    let simulated_order = manager
        .simulate_new_order_with_fees(borrow_params, agnostic_orderbook::state::Side::Ask)
        .await?;
    transactions! {
        pricer.set_oracle_price_tx(&collateral, 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.ticket_mint(), 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.token_mint(), 1.0).await?,
        borrower.refresh_and_margin_borrow_order(borrow_params).await?,
    }
    .send_and_confirm_condensed_in_order(&client)
    .await?;

    assert_eq!(STARTING_TOKENS + 999, borrower.tokens().await?);
    assert_eq!(0, borrower.tickets().await?);
    assert_eq!(0, borrower.ticket_collateral().await?);
    assert_eq!(1_201, borrower.claims().await?);

    // FIXME: an exact number would be nice
    assert!(manager.collected_fees().await? > 0);

    let loan = borrower.load_term_loan(0).await?;
    let expected_tenor = manager.load_market().await?.borrow_tenor;
    // FIXME
    // to avoid subtle rounding issues, we calculate expected price manually, this is not as good as
    // using the limit price of the order directly
    let expected_price = {
        let summary = SensibleOrderSummary::new(borrow_params.limit_price, simulated_order);
        let price = Fp32::from(summary.quote_filled(RoundingAction::FillBorrow.direction())?)
            / summary.base_filled();
        price.downcast_u64().unwrap()
    };
    let expected_rate =
        PricerImpl::price_fp32_to_bps_yearly_interest(expected_price, expected_tenor);

    assert_eq!(loan.tenor()?, expected_tenor);
    assert_eq!(loan.price()?.downcast_u64().unwrap(), expected_price);
    assert_eq!(loan.rate()?, expected_rate);

    manager.consume_events().await?;
    // assert!(false);
    manager.expect_and_execute_settlement(&[&lender]).await?;

    // todo improve the rounding situation to make this 1_000
    assert_eq!(STARTING_TOKENS + 999, borrower.tokens().await?);
    assert_eq!(0, borrower.tickets().await?);
    assert_eq!(0, borrower.ticket_collateral().await?);
    assert_eq!(1_201, borrower.claims().await?);

    assert_eq!(STARTING_TOKENS - 1_001, lender.tokens().await?);
    assert_eq!(0, lender.tickets().await?);
    assert_eq!(1_201, lender.ticket_collateral().await?);
    assert_eq!(0, lender.claims().await?);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn margin_sell_tickets() -> Result<()> {
    let ctx = margin_test_context!();
    let manager = Arc::new(FixedTermTestManager::full(&ctx).await.unwrap());
    let client = manager.client.clone();
    let ([], _, pricer) = tokens(&ctx).await.unwrap();

    let user = create_and_fund_fixed_term_market_margin_user(&ctx, manager.clone(), vec![]).await;
    ctx.issue_permit(user.proxy.pubkey()).await?;
    user.convert_tokens(10_000).await.unwrap();

    transactions! {
        pricer.set_oracle_price_tx(&manager.ix_builder.ticket_mint(), 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.token_mint(), 1.0).await?,
        user.margin_sell_tickets_order(tickets(1_200, 2_000)).await?,
    }
    .send_and_confirm_condensed_in_order(&client)
    .await?;

    assert_eq!(STARTING_TOKENS - 10_000, user.tokens().await?);
    assert_eq!(8_800, user.tickets().await?);
    assert_eq!(999, user.ticket_collateral().await?);
    assert_eq!(0, user.claims().await?);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn auto_roll_settings_are_correct() -> Result<()> {
    let ctx = margin_test_context!();
    let manager = Arc::new(FixedTermTestManager::full(&ctx).await.unwrap());
    let ([collateral], _, _) = tokens(&ctx).await?;

    let user = create_and_fund_fixed_term_market_margin_user(
        &ctx,
        manager.clone(),
        vec![(collateral, 0, u64::MAX / 2)],
    )
    .await;

    // can properly set config
    let market_tenor = manager.load_market().await?.borrow_tenor;
    let lend_price = OrderAmount::from_base_amount_rate(1_000, 1_000).price;
    let borrow_price = OrderAmount::from_base_amount_rate(1_000, 900).price;
    let borrow_roll_tenor = market_tenor - 1;
    user.set_lend_roll_config(LendAutoRollConfig {
        limit_price: lend_price,
    })
    .await?;
    user.set_borrow_roll_config(BorrowAutoRollConfig {
        limit_price: borrow_price,
        roll_tenor: borrow_roll_tenor,
    })
    .await?;

    let margin_user = user.load_margin_user().await?;
    let borrow_roll_config = margin_user.borrow_roll_config.as_ref().unwrap();
    let lend_roll_config = margin_user.lend_roll_config.as_ref().unwrap();

    assert_eq!(lend_roll_config.limit_price, lend_price);
    assert_eq!(borrow_roll_config.limit_price, borrow_price);
    assert_eq!(borrow_roll_config.roll_tenor, borrow_roll_tenor);

    // cannot set a bad config
    assert!(user
        .set_lend_roll_config(LendAutoRollConfig { limit_price: 0 })
        .await
        .is_err());
    assert!(user
        .set_lend_roll_config(LendAutoRollConfig {
            limit_price: jet_program_common::FP32_ONE as u64 + 1
        })
        .await
        .is_err());
    assert!(user
        .set_borrow_roll_config(BorrowAutoRollConfig {
            limit_price: borrow_price,
            roll_tenor: market_tenor + 1,
        })
        .await
        .is_err());
    assert!(user
        .set_borrow_roll_config(BorrowAutoRollConfig {
            limit_price: borrow_price,
            roll_tenor: 0,
        })
        .await
        .is_err());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn auto_roll_flags() -> Result<()> {
    let ctx = margin_test_context!();
    let manager = Arc::new(FixedTermTestManager::full(&ctx).await.unwrap());
    let client = manager.client.clone();
    let ([collateral], _, pricer) = tokens(&ctx).await.unwrap();

    let user = create_and_fund_fixed_term_market_margin_user(
        &ctx,
        manager.clone(),
        vec![(collateral, 0, u64::MAX / 2)],
    )
    .await;

    let mut params = underlying(1_000, 2_000);
    params.auto_roll = true;
    let borrow_order = transactions! {
        pricer.set_oracle_price_tx(&collateral, 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.token_mint(), 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.ticket_mint(), 1.0).await?,
        user.refresh_and_margin_borrow_order(params).await?
    };

    // TODO: assert proper failure
    // This fails due to an unconfigured auto_roll setting in the margin_user account
    let res = borrow_order
        .clone()
        .send_and_confirm_condensed_in_order(&client)
        .await;
    assert!(res.is_err());

    user.set_borrow_roll_config(BorrowAutoRollConfig {
        limit_price: params.limit_price,
        roll_tenor: manager.load_market().await?.borrow_tenor - 1,
    })
    .await?;

    borrow_order
        .send_and_confirm_condensed_in_order(&client)
        .await?;

    let posted_info = manager.load_orderbook().await?.asks_order_callback(0)?;
    assert!(posted_info.flags().contains(CallbackFlags::AUTO_ROLL));

    params.auto_roll = false;
    transactions! {
        pricer.set_oracle_price_tx(&collateral, 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.token_mint(), 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.ticket_mint(), 1.0).await?,
        user.refresh_and_margin_borrow_order(params).await?
    }
    .send_and_confirm_condensed_in_order(&client)
    .await?;

    let posted_info = manager.load_orderbook().await?.asks_order_callback(1)?;
    assert!(!posted_info.flags().contains(CallbackFlags::AUTO_ROLL));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn auto_roll_lend_order_is_correct() -> Result<()> {
    let ctx = margin_test_context!();
    let manager = Arc::new(FixedTermTestManager::full(&ctx).await.unwrap());
    let client = manager.client.clone();
    let ([collateral], _, pricer) = tokens(&ctx).await.unwrap();

    let borrower = create_and_fund_fixed_term_market_margin_user(
        &ctx,
        manager.clone(),
        vec![(collateral, 0, u64::MAX / 2)],
    )
    .await;
    let lender = create_and_fund_fixed_term_market_margin_user(&ctx, manager.clone(), vec![]).await;
    lender
        .set_lend_roll_config(LendAutoRollConfig {
            limit_price: underlying(1_001, 2_000).limit_price,
        })
        .await?;
    let mut lend_params = underlying(1_001, 2_000);
    lend_params.auto_roll = true;

    transactions! {
        pricer.set_oracle_price_tx(&collateral, 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.ticket_mint(), 1.0).await?,
        pricer.set_oracle_price_tx(&manager.ix_builder.token_mint(), 1.0).await?,
        lender.refresh_and_margin_lend_order(lend_params).await?,
        borrower.refresh_and_margin_borrow_order(underlying(1_000, 2_000)).await?,
    }
    .send_and_confirm_condensed_in_order(&client)
    .await?;

    manager.consume_events().await?;
    manager.expect_and_execute_settlement(&[&lender]).await?;

    // let the `TermDeposit` mature
    #[cfg(not(feature = "localnet"))]
    {
        let mut clock = manager.client.get_clock().await?;
        clock.unix_timestamp += hosted_tests::fixed_term::LEND_TENOR as i64 + 1;
        manager.client.set_clock(clock).await?;
    }
    #[cfg(feature = "localnet")]
    {
        std::thread::sleep(std::time::Duration::from_secs(
            hosted_tests::fixed_term::LEND_TENOR as u64,
        ));
    }

    // repay the loan
    borrower.try_repay_all().await?;

    let market_balance_pre = manager.load_manager_token_vault().await?.amount;
    manager
        .auto_roll_term_deposits(&lender.proxy.pubkey())
        .await?;
    let market_balance_post = manager.load_manager_token_vault().await?.amount;

    // no tokens should have leaked
    assert_eq!(market_balance_pre, market_balance_post);

    let order_info = manager.load_orderbook().await?.bids_order_callback(0)?;
    assert_eq!(order_info.owner(), lender.proxy.pubkey());
    assert_eq!(
        MarginCallbackInfo::from(order_info).margin_user,
        manager
            .ix_builder
            .margin_user(lender.proxy.pubkey())
            .address
    );

    Ok(())
}

/// This mirrors the setup for the direct_repay_fixed_term_loan test in the liquidator
#[tokio::test]
async fn fixed_term_borrow_becomes_unhealthy_without_collateral() -> Result<(), anyhow::Error> {
    let ctx = margin_test_context!();
    let (usdc, usdc_description) = ctx.basic_token(1.0).await?;
    let tsol = ctx.basic_token(10.0).await?.0;
    let mkt = ctx
        .create_fixed_term_market(usdc_description, test_default())
        .await?;

    // Users
    let lender = ctx.create_margin_user(100).await?;
    let borrower = ctx.create_margin_user(100).await?;
    let params = OrderAmount::from_base_amount_rate(usdc.amount(100.0), 10).default_order_params();

    transactions! {
        // collateral positions
        ctx.margin_airdrop(usdc.mint, lender.auth(), usdc.amount(100.0)),
        ctx.margin_airdrop(tsol.mint, borrower.auth(), tsol.amount(100.0)),
        ctx.register_deposit_position(usdc.mint, borrower.auth()),

        // add liquidity, so a borrow is possible
        vec![
            mkt.initialize_margin_user(*lender.address()),
            mkt.margin_lend_order(*lender.address(), None, params, 0),
        ].invoke_each_into(&lender.ctx())
         .with_signer(lender.signer.clone()),

        // borrow with fill
        ctx.refresh_deposit(tsol.mint, *borrower.address()),
        mkt.initialize_margin_user(*borrower.address())
            .invoke_into(&borrower.ctx())
            .with_signer(borrower.signer.clone()),
        vec![
            mkt.refresh_position(*borrower.address(), true),
            mkt.margin_borrow_order(*borrower.address(), params, 0)
        ].invoke_into(&borrower.ctx())
         .with_signer(borrower.signer.clone()),

        // make user unhealthy
        ctx.set_price(tsol.mint, 0.01),
        ctx.refresh_deposit(tsol.mint, *borrower.address()),
        ctx.refresh_deposit(usdc.mint, *borrower.address()),
    }
    .send_and_confirm_condensed_in_order(&ctx.rpc())
    .await?;

    assert!(borrower.verify_healthy().await.is_err());
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn auto_roll_borrow_order_is_correct() -> Result<()> {
    let ctx = margin_test_context!();
    let manager = Arc::new(FixedTermTestManager::full(&ctx).await.unwrap());
    let client = manager.client.clone();
    let ([collateral], _, pricer) = tokens(&ctx).await.unwrap();

    let borrower = create_and_fund_fixed_term_market_margin_user(
        &ctx,
        manager.clone(),
        vec![(collateral, 0, u64::MAX / 2)],
    )
    .await;
    let lender = create_and_fund_fixed_term_market_margin_user(&ctx, manager.clone(), vec![]).await;

    let borrow_params = OrderParams {
        auto_roll: true,
        ..underlying(1_000, 2_000)
    };
    let lend_params = underlying(1_000_000, 2000);

    let roll_tenor = 1;
    borrower
        .set_borrow_roll_config(BorrowAutoRollConfig {
            limit_price: borrow_params.limit_price,
            roll_tenor,
        })
        .await?;

    transactions! {
        pricer.set_oracle_price_tx(&collateral, 1.0).await.unwrap(),
        pricer.set_oracle_price_tx(&manager.ix_builder.ticket_mint(), 1.0).await.unwrap(),
        pricer.set_oracle_price_tx(&manager.ix_builder.token_mint(), 1.0).await?,
        lender.refresh_and_margin_lend_order(lend_params).await?,
        borrower.refresh_and_margin_borrow_order(borrow_params).await?,
    }
    .send_and_confirm_condensed_in_order(&client)
    .await?;

    manager.consume_events().await?;

    // let the `TermDeposit` mature
    #[cfg(not(feature = "localnet"))]
    {
        let mut clock = manager.client.get_clock().await?;
        clock.unix_timestamp += roll_tenor as i64;
        manager.client.set_clock(clock).await?;
    }
    #[cfg(feature = "localnet")]
    {
        std::thread::sleep(std::time::Duration::from_secs(roll_tenor));
    }

    let pre_roll_loan = borrower.get_active_term_loans().await?[0].clone();
    manager
        .auto_roll_term_loans(&borrower.proxy.pubkey())
        .await?;
    let post_roll_loans = borrower.get_active_term_loans().await?;

    // we had enough liquidity, so the first loan should be fully repaid, leaving only one
    assert!(post_roll_loans.len() < 2);

    // FIXME: add the fee calculation to get an exact number
    // The principal of the new loan is the balance of the previous, plus an originiation fee
    assert!(pre_roll_loan.balance < post_roll_loans[0].principal);

    Ok(())
}

fn quote_to_base(quote: u64, rate_bps: u64) -> u64 {
    quote + quote * rate_bps / 10_000
}

fn underlying(quote: u64, rate_bps: u64) -> OrderParams {
    let borrow_amount = OrderAmount::from_quote_amount_rate(quote, rate_bps);
    OrderParams {
        max_ticket_qty: borrow_amount.base,
        max_underlying_token_qty: borrow_amount.quote,
        limit_price: borrow_amount.price,
        match_limit: 1,
        post_only: false,
        post_allowed: true,
        auto_stake: true,
        auto_roll: false,
    }
}

fn tickets(base: u64, rate_bps: u64) -> OrderParams {
    let borrow_amount = OrderAmount::from_base_amount_rate(base, rate_bps);
    OrderParams {
        max_ticket_qty: borrow_amount.base,
        max_underlying_token_qty: borrow_amount.quote,
        limit_price: borrow_amount.price,
        match_limit: 1,
        post_only: false,
        post_allowed: true,
        auto_stake: true,
        auto_roll: false,
    }
}
