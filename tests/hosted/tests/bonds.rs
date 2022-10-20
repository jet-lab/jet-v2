use std::sync::Arc;

use anyhow::Result;
use hosted_tests::{
    bonds::{
        BondsUser, GenerateProxy, OrderAmount, TestManager as BondsTestManager, MIN_ORDER_SIZE,
        STARTING_TOKENS,
    },
    context::test_context,
    setup_helper::{setup_user, tokens},
};
use jet_bonds::orderbook::state::{CallbackInfo, OrderParams};
use jet_margin_sdk::{
    ix_builder::MarginIxBuilder,
    margin_integrator::{NoProxy, Proxy},
    tx_builder::bonds::BondsPositionRefresher,
};
use jet_margin_sdk::{
    margin_integrator::RefreshingProxy, solana::transaction::SendTransactionBuilder,
    tx_builder::MarginTxBuilder,
};
use jet_proto_math::fixed_point::Fp32;

use solana_sdk::signer::Signer;

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(feature = "localnet"), serial_test::serial)]
async fn full_direct() -> Result<(), anyhow::Error> {
    let manager = BondsTestManager::full(test_context().await.rpc.clone()).await?;
    _full_workflow::<NoProxy>(Arc::new(manager)).await
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial]
async fn full_through_margin() -> Result<()> {
    let manager = BondsTestManager::full(test_context().await.rpc.clone()).await?;
    _full_workflow::<MarginIxBuilder>(Arc::new(manager)).await
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial]
#[allow(unused_variables)] //todo remove this once fixme is addressed
async fn margin() -> Result<()> {
    let ctx = test_context().await;
    let manager = Arc::new(BondsTestManager::full(ctx.rpc.clone()).await.unwrap());
    let client = manager.client.clone();
    let ([collateral], _, pricer) = tokens(ctx).await.unwrap();

    // set up user
    let user = setup_user(ctx, vec![(collateral, 0, u64::MAX / 2)])
        .await
        .unwrap();
    let margin = user.user.tx.ix.clone();
    let wallet = user.user.signer;

    // set up proxy
    let proxy = RefreshingProxy {
        proxy: margin.clone(),
        refreshers: vec![
            Arc::new(MarginTxBuilder::new(
                client.clone(),
                None,
                wallet.pubkey(),
                0,
            )),
            Arc::new(
                BondsPositionRefresher::new(
                    margin.pubkey(),
                    client.clone(),
                    &[manager.ix_builder.manager()],
                )
                .await
                .unwrap(),
            ),
        ],
    };

    let user = BondsUser::new_with_proxy_funded(manager.clone(), wallet, proxy.clone())
        .await
        .unwrap();
    user.initialize_margin_user().await.unwrap();

    let borrower_account = user.load_margin_user().await.unwrap();
    assert_eq!(borrower_account.bond_manager, manager.ix_builder.manager());

    // place a borrow order
    let borrow_amount = OrderAmount::from_amount_rate(1_000, 2_000);
    let borrow_params = OrderParams {
        max_bond_ticket_qty: borrow_amount.base,
        max_underlying_token_qty: borrow_amount.quote,
        limit_price: borrow_amount.price,
        match_limit: 1,
        post_only: false,
        post_allowed: true,
        auto_stake: true,
    };
    let mut ixs = vec![
        pricer.set_oracle_price_tx(&collateral, 1.0).await.unwrap(),
        pricer
            .set_oracle_price_tx(&manager.ix_builder.token_mint(), 1.0)
            .await
            .unwrap(),
    ];
    ixs.extend(user.margin_borrow_order(borrow_params).await.unwrap());
    client
        .send_and_confirm_condensed_in_order(ixs)
        .await
        .unwrap();

    let borrower_account = user.load_margin_user().await.unwrap();
    assert!(borrower_account.debt.total() == borrow_params.max_bond_ticket_qty);

    Ok(())
}

async fn _full_workflow<P: Proxy + GenerateProxy>(manager: Arc<BondsTestManager>) -> Result<()> {
    let alice = BondsUser::<P>::new_funded(manager.clone()).await?;

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

    alice
        .stake_tokens(STAKE_AMOUNT, ticket_seed.clone())
        .await?;
    assert_eq!(alice.tickets().await?, START_TICKETS - STAKE_AMOUNT);

    let ticket = alice.load_claim_ticket(ticket_seed.clone()).await?;
    assert_eq!(ticket.redeemable, STAKE_AMOUNT);
    assert_eq!(ticket.bond_manager, manager.ix_builder.manager());
    assert_eq!(ticket.owner, alice.proxy.pubkey());

    manager.pause_ticket_redemption().await?;
    let bond_manager = manager.load_manager().await?;

    assert!(bond_manager.tickets_paused);
    assert!(alice.redeem_claim_ticket(ticket_seed).await.is_err());

    manager.resume_ticket_redemption().await?;

    let bond_manager = manager.load_manager().await?;
    assert!(!bond_manager.tickets_paused);

    // Scenario a: post a borrow order to an empty book
    let a_amount = OrderAmount::from_amount_rate(1_000, 2_000);
    let a_params = OrderParams {
        max_bond_ticket_qty: a_amount.base,
        max_underlying_token_qty: a_amount.quote,
        limit_price: a_amount.price,
        match_limit: 100,
        post_only: false,
        post_allowed: true,
        auto_stake: true,
    };

    // simulate
    let mut eq = manager.load_event_queue().await?;
    let mut orderbook = manager.load_orderbook().await?;
    let summary_a = orderbook.inner()?.new_order(
        a_params.as_new_order_params(
            agnostic_orderbook::state::Side::Ask,
            CallbackInfo::default(),
        ),
        &mut eq.inner(),
        MIN_ORDER_SIZE,
    )?;
    assert!(summary_a.posted_order_id.is_some());
    assert_eq!(summary_a.total_base_qty, a_params.max_bond_ticket_qty);
    assert_eq!(
        summary_a.total_quote_qty,
        Fp32::upcast_fp32(a_params.limit_price)
            .decimal_u64_mul(a_params.max_bond_ticket_qty)
            .unwrap()
    );
    assert_eq!(
        summary_a.total_base_qty_posted,
        a_params.max_bond_ticket_qty
    );

    // send to validator
    alice.sell_tickets_order(a_params).await?;

    assert_eq!(
        alice.tickets().await?,
        START_TICKETS - STAKE_AMOUNT - a_amount.base
    );

    let borrow_order = manager.load_orderbook().await?.asks()?[0];

    assert_eq!(borrow_order.price(), a_amount.price);
    assert_eq!(borrow_order.base_quantity, a_amount.base);
    // quote amounts of the post are a result of an fp32 mul, so we cannot directly compare
    assert_eq!(
        Fp32::upcast_fp32(borrow_order.price())
            .decimal_u64_mul(borrow_order.base_quantity)
            .unwrap(),
        Fp32::upcast_fp32(a_amount.price)
            .decimal_u64_mul(a_amount.base)
            .unwrap()
    );

    // Scenario b: post a lend order that partially fills the borrow order and does not post remaining
    let b_amount = OrderAmount::from_amount_rate(500, 1_500);
    let b_params = OrderParams {
        max_bond_ticket_qty: b_amount.base,
        max_underlying_token_qty: b_amount.quote,
        limit_price: b_amount.price,
        match_limit: 100,
        post_only: false,
        post_allowed: true,
        auto_stake: true,
    };

    // simulate
    let mut eq = manager.load_event_queue().await?;
    let mut orderbook = manager.load_orderbook().await?;
    let summary_b = orderbook.inner()?.new_order(
        b_params.as_new_order_params(
            agnostic_orderbook::state::Side::Bid,
            CallbackInfo::default(),
        ),
        &mut eq.inner(),
        MIN_ORDER_SIZE,
    )?;

    let trade_price = Fp32::upcast_fp32(borrow_order.price());
    let base_trade_qty = borrow_order
        .base_quantity
        .min(b_params.max_bond_ticket_qty)
        .min(
            (Fp32::from(b_params.max_underlying_token_qty) / trade_price)
                .as_decimal_u64()
                .unwrap(),
        );
    let quote_maker_qty = (trade_price * base_trade_qty)
        .as_decimal_u64_ceil()
        .unwrap();

    assert!(summary_b.posted_order_id.is_none());
    assert_eq!(summary_b.total_base_qty, b_params.max_bond_ticket_qty);
    assert_eq!(summary_b.total_quote_qty, quote_maker_qty);

    // send to validator
    let bob = BondsUser::<P>::new_funded(manager.clone()).await?;
    bob.lend_order(b_params, vec![0]).await?;

    let split_ticket_b = bob.load_split_ticket(vec![0]).await?;
    dbg!(split_ticket_b);

    assert_eq!(
        bob.tokens().await?,
        STARTING_TOKENS - summary_b.total_quote_qty
    );

    // scenario c: post a lend order that fills the remaining borrow and makes a new post with the remaining
    let c_amount = OrderAmount::from_amount_rate(1_500, 1_500);
    let c_params = OrderParams {
        max_bond_ticket_qty: c_amount.base,
        max_underlying_token_qty: c_amount.quote,
        limit_price: c_amount.price,
        match_limit: 100,
        post_only: false,
        post_allowed: true,
        auto_stake: true,
    };

    // simulate
    let mut eq = manager.load_event_queue().await?;
    let mut orderbook = manager.load_orderbook().await?;
    let summary_c = orderbook.inner()?.new_order(
        c_params.as_new_order_params(
            agnostic_orderbook::state::Side::Bid,
            CallbackInfo::default(),
        ),
        &mut eq.inner(),
        MIN_ORDER_SIZE,
    )?;

    let existing_order = manager.load_orderbook().await?.asks()?[0];

    let trade_price = Fp32::upcast_fp32(existing_order.price());
    let base_trade_qty = existing_order
        .base_quantity
        .min(c_params.max_bond_ticket_qty)
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
        c_params.max_bond_ticket_qty - base_trade_qty,
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
    bob.lend_order(c_params, vec![1]).await?;

    let split_ticket_c = bob.load_split_ticket(vec![1]).await?;
    dbg!(split_ticket_c);

    assert_eq!(
        bob.tokens().await?,
        STARTING_TOKENS - summary_b.total_quote_qty - summary_c.total_quote_qty
    );

    // let lend_order = manager.load_orderbook().await?.bids()?[0];

    // assert_eq!(lend_order.price(), lend_amount.price);
    // assert_eq!(lend_order.base_quantity, lend_amount.base);
    // // quote amounts of the post are a result of an fp32 mul, so we cannot directly compare
    // assert_eq!(
    //     Fp32::upcast_fp32(lend_order.price())
    //         .decimal_u64_mul(lend_order.base_quantity)
    //         .unwrap(),
    //     Fp32::upcast_fp32(lend_amount.price)
    //         .decimal_u64_mul(lend_amount.base)
    //         .unwrap()
    // );

    // manager.pause_orders().await?;

    // // // lend 100 usdc at 15% interest
    // let lend_amount = OrderAmount::from_amount_rate(1_000, 1_500);
    // let lend_params = OrderParams {
    //     max_bond_ticket_qty: lend_amount.base,
    //     max_underlying_token_qty: lend_amount.quote,
    //     limit_price: lend_amount.price,
    //     match_limit: 1,
    //     post_only: false,
    //     post_allowed: true,
    //     auto_stake: true,
    // };
    // bob.lend_order(lend_params, vec![]).await?;

    // let mut eq = manager.load_event_queue().await?;
    // assert!(eq.inner().iter().next().is_none());
    // assert!(manager.consume_events().await.is_err());

    // assert!(manager.load_orderbook_market_state().await?.pause_matching == true as u8);
    // manager.resume_orders().await?;
    // assert!(manager.load_orderbook_market_state().await?.pause_matching == false as u8);

    // let remaining_order = manager.load_orderbook().await?.asks()?[0];

    // assert_eq!(
    //     remaining_order.base_quantity,
    //     borrow_order.base_quantity - lend_order.base_quantity
    // );
    // assert_eq!(remaining_order.price(), borrow_order.price());

    // alice.cancel_order(remaining_order.order_id()).await?;

    // let mut eq = manager.load_event_queue().await?;
    // assert!(eq.inner().iter().next().is_some());

    // only works on simulation right now
    // Access violation in stack frame 5 at address 0x200005ff8 of size 8 by instruction #22627
    #[cfg(not(feature = "localnet"))]
    manager.consume_events().await?;

    // assert SplitTicket

    // make an adapter

    // place and match a bunch of orders

    Ok(())
}
