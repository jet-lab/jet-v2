use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use hosted_tests::{
    bonds::{
        BondsUser, GenerateProxy, OrderAmount, TestManager as BondsTestManager, STARTING_TOKENS,
    },
    context::test_context,
};
use jet_bonds::orderbook::state::OrderParams;
use jet_margin_sdk::{
    ix_builder::MarginIxBuilder,
    margin_integrator::{NoProxy, Proxy},
    solana::transaction::InverseSendTransactionBuilder,
    tx_builder::bonds::BondsPositionRefresher,
};
use jet_margin_sdk::{
    margin_integrator::RefreshingProxy, solana::transaction::SendTransactionBuilder,
    tx_builder::MarginTxBuilder,
};
use jet_proto_math::fixed_point::Fp32;
use jet_simulation::create_wallet;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, signer::Signer};

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
    let manager = Arc::new(BondsTestManager::full(ctx.rpc.clone()).await?);
    let client = manager.client.clone();

    // create user
    let wallet = create_wallet(&ctx.rpc.clone(), 100 * LAMPORTS_PER_SOL).await?;
    let margin = MarginIxBuilder::new(wallet.pubkey(), 0);
    client
        .send_and_confirm_1tx(&[margin.create_account()], &[&wallet])
        .await?;

    let mut bonds_position_refresher = BondsPositionRefresher {
        margin_account: margin.pubkey(),
        bond_markets: HashMap::new(),
        rpc: client.clone(),
    };
    bonds_position_refresher
        .add_bond_market(manager.ix_builder.manager())
        .await?;

    let proxy = RefreshingProxy {
        proxy: margin.clone(),
        refreshers: vec![
            Arc::new(MarginTxBuilder::new(
                client.clone(),
                None,
                wallet.pubkey(),
                0,
            )),
            Arc::new(bonds_position_refresher),
        ],
    };

    let user = BondsUser::new_with_proxy_funded(manager.clone(), wallet, proxy.clone()).await?;
    user.initialize_margin_user().await?;

    let borrower_account = user.load_margin_user().await?;
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

    // this fails checks in margin after the bonds ix completes successfully
    // FIXME:
    // - get claim registerable in margin
    // - get usdc registerable in margin
    // - register usdc position directly
    // - deposit usdc to position
    user.margin_borrow_order(borrow_params)
        .await?
        .send_and_confirm_condensed(&client)
        .await?;

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

    // borrow 100 usdc at 20% interest
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

    alice.sell_tickets_order(borrow_params).await?;

    assert_eq!(
        alice.tickets().await?,
        START_TICKETS - STAKE_AMOUNT - borrow_amount.base
    );

    let borrow_order = manager.load_orderbook().await?.asks()?[0];

    assert_eq!(borrow_order.price(), borrow_amount.price);
    assert_eq!(borrow_order.base_quantity, borrow_amount.base);
    // quote amounts of the post are a result of an fp32 mul, so we cannot directly compare
    assert_eq!(
        Fp32::upcast_fp32(borrow_order.price())
            .decimal_u64_mul(borrow_order.base_quantity)
            .unwrap(),
        Fp32::upcast_fp32(borrow_amount.price)
            .decimal_u64_mul(borrow_amount.base)
            .unwrap()
    );

    manager.pause_orders().await?;
    let bob = BondsUser::<P>::new_funded(manager.clone()).await?;

    // // lend 100 usdc at 15% interest
    let lend_amount = OrderAmount::from_amount_rate(1_000, 1_500);
    let lend_params = OrderParams {
        max_bond_ticket_qty: lend_amount.base,
        max_underlying_token_qty: lend_amount.quote,
        limit_price: lend_amount.price,
        match_limit: 1,
        post_only: false,
        post_allowed: true,
        auto_stake: true,
    };
    bob.lend_order(lend_params, vec![]).await?;

    assert_eq!(bob.tokens().await?, STARTING_TOKENS - lend_amount.quote);

    let lend_order = manager.load_orderbook().await?.bids()?[0];

    assert_eq!(lend_order.price(), lend_amount.price);
    assert_eq!(lend_order.base_quantity, lend_amount.base);
    // quote amounts of the post are a result of an fp32 mul, so we cannot directly compare
    assert_eq!(
        Fp32::upcast_fp32(lend_order.price())
            .decimal_u64_mul(lend_order.base_quantity)
            .unwrap(),
        Fp32::upcast_fp32(lend_amount.price)
            .decimal_u64_mul(lend_amount.base)
            .unwrap()
    );

    let mut eq = manager.load_event_queue().await?;
    assert!(eq.inner().iter().next().is_none());
    assert!(manager.consume_events().await.is_err());

    assert!(manager.load_orderbook_market_state().await?.pause_matching == true as u8);
    manager.resume_orders().await?;
    assert!(manager.load_orderbook_market_state().await?.pause_matching == false as u8);

    let remaining_order = manager.load_orderbook().await?.asks()?[0];

    assert_eq!(
        remaining_order.base_quantity,
        borrow_order.base_quantity - lend_order.base_quantity
    );
    assert_eq!(remaining_order.price(), borrow_order.price());

    alice.cancel_order(remaining_order.order_id()).await?;

    let mut eq = manager.load_event_queue().await?;
    assert!(eq.inner().iter().next().is_some());

    // only works on simulation right now
    // Access violation in stack frame 5 at address 0x200005ff8 of size 8 by instruction #22627
    #[cfg(not(feature = "localnet"))]
    manager.consume_events().await?;

    // assert SplitTicket

    // make an adapter

    // place and match a bunch of orders

    Ok(())
}
