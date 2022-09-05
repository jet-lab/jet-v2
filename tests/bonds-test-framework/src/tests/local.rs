use std::sync::Arc;

use anyhow::Result;
use jet_bonds::orderbook::state::OrderParams;
use jet_bonds_lib::utils::{Fp32, OrderAmount};
use solana_sdk::signer::Signer;

use crate::{
    emulated_client, localhost_client, BondsUser, TestManager, CONVERSION_DECIMALS, ONE_TOKEN,
    STARTING_TOKENS,
};

async fn _full_workflow(manager: Arc<TestManager>) -> Result<()> {
    let alice = BondsUser::new_funded(manager.clone()).await?;
    const START_TICKETS: u64 = 1_000_000 * ONE_TOKEN;
    alice.convert_tokens(START_TICKETS).await?;

    const STAKE_AMOUNT: u64 = 1_000 * ONE_TOKEN;
    let stake_amount = 1_000 * ONE_TOKEN;
    let claim_amount = match CONVERSION_DECIMALS {
        decimals if decimals > 0 => stake_amount * 10u64.pow(decimals as u32),
        #[allow(clippy::cast_abs_to_unsigned)]
        decimals if decimals < 0 => stake_amount / 10u64.pow(decimals.abs() as u32),
        _ => stake_amount,
    };
    let ticket_seed = 1337_u64;

    alice.stake_tokens(stake_amount, ticket_seed).await?;
    assert_eq!(alice.tickets().await?, START_TICKETS - STAKE_AMOUNT);

    let ticket = alice.load_claim_ticket(ticket_seed).await?;

    assert_eq!(ticket.redeemable, claim_amount);
    assert_eq!(ticket.bond_manager, manager.ix_builder.manager());
    assert_eq!(ticket.owner, alice.kp.pubkey());

    manager.pause_ticket_redemption().await?;
    let bond_manager = manager.load_manager().await?;

    assert!(bond_manager.tickets_paused);
    assert!(alice.redeem_claim_ticket(ticket_seed).await.is_err());

    manager.resume_ticket_redemption().await?;

    let bond_manager = manager.load_manager().await?;
    assert!(!bond_manager.tickets_paused);

    // borrow 100 usdc at 20% interest
    let borrow_amount = OrderAmount::new(100 * ONE_TOKEN, 2_000).unwrap();
    let borrow_params = OrderParams {
        max_bond_ticket_qty: borrow_amount.base,
        max_underlying_token_qty: borrow_amount.quote,
        limit_price: borrow_amount.price,
        match_limit: 1,
        post_only: false,
        post_allowed: true,
        auto_stake: true,
    };

    alice.borrow_order(borrow_params).await?;

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
            .u64_mul(borrow_order.base_quantity)
            .unwrap(),
        Fp32::upcast_fp32(borrow_amount.price)
            .u64_mul(borrow_amount.base)
            .unwrap()
    );

    manager.pause_orders().await?;
    let bob = BondsUser::new_funded(manager.clone()).await?;

    // // lend 100 usdc at 15% interest
    let lend_amount = OrderAmount::new(100 * ONE_TOKEN, 1_500).unwrap();
    let lend_params = OrderParams {
        max_bond_ticket_qty: lend_amount.base,
        max_underlying_token_qty: lend_amount.quote,
        limit_price: lend_amount.price,
        match_limit: 1,
        post_only: false,
        post_allowed: true,
        auto_stake: true,
    };
    bob.lend_order(lend_params, 0).await?;

    assert_eq!(bob.tokens().await?, STARTING_TOKENS - lend_amount.quote);

    let lend_order = manager.load_orderbook().await?.bids()?[0];

    assert_eq!(lend_order.price(), lend_amount.price);
    assert_eq!(lend_order.base_quantity, lend_amount.base);
    // quote amounts of the post are a result of an fp32 mul, so we cannot directly compare
    assert_eq!(
        Fp32::upcast_fp32(lend_order.price())
            .u64_mul(lend_order.base_quantity)
            .unwrap(),
        Fp32::upcast_fp32(lend_amount.price)
            .u64_mul(lend_amount.base)
            .unwrap()
    );

    let mut eq = manager.load_event_queue().await?;
    assert!(eq.inner().iter().next().is_none());
    assert!(manager.consume_events().await.is_err());

    manager.resume_orders().await?;

    let remaining_order = manager.load_orderbook().await?.asks()?[0];

    assert_eq!(
        remaining_order.base_quantity,
        borrow_order.base_quantity - lend_order.base_quantity
    );
    assert_eq!(remaining_order.price(), borrow_order.price());

    let mut eq = manager.load_event_queue().await?;
    assert!(eq.inner().iter().next().is_some());

    // manager.consume_events().await?;

    // assert SplitTicket

    // make an adapter

    // place and match a bunch of orders

    Ok(())
}

#[tokio::test]
async fn emulated_full() -> Result<()> {
    let client = emulated_client();
    let manager = TestManager::new(Arc::new(client))
        .await?
        .with_bonds()
        .await?
        .with_crank()
        .await?;

    _full_workflow(Arc::new(manager)).await
}

#[tokio::test(flavor = "multi_thread")]
async fn localhost_full() -> Result<()> {
    let client = localhost_client();
    let manager = TestManager::new(Arc::new(client))
        .await?
        .with_bonds()
        .await?
        .with_crank()
        .await?;

    _full_workflow(Arc::new(manager)).await
}
