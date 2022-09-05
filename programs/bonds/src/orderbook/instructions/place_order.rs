use std::collections::BTreeMap;

use agnostic_orderbook::{
    instruction::new_order,
    state::{OrderSummary, SelfTradeBehavior, Side},
};
use anchor_lang::prelude::*;
use jet_proto_math::traits::TrySubAssign;

use crate::{
    control::state::BondManager,
    events::OrderPlaced,
    orderbook::state::{user::OrderbookUser, CallbackFlags, CallbackInfo, OrderParams, OrderSide},
    BondsError,
};

#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    /// The account tracking information related to this particular user
    #[account(
        mut,
        has_one = bond_manager @ BondsError::UserNotInMarket,
        has_one = user @ BondsError::UserDoesNotOwnAccount,
    )]
    pub orderbook_user_account: Account<'info, OrderbookUser>,

    /// The signing authority for this user account
    pub user: Signer<'info>,

    /// The `BondManager` account tracks global information related to this particular bond market
    #[account(
        has_one = orderbook_market_state @ BondsError::WrongMarketState,
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,

    // aaob accounts
    /// CHECK: handled by aaob
    #[account(
        mut,
        owner = crate::ID @ BondsError::MarketStateNotProgramOwned
    )]
    pub orderbook_market_state: AccountInfo<'info>,
    /// CHECK: handled by aaob
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    /// CHECK: handled by aaob
    #[account(mut)]
    pub bids: AccountInfo<'info>,
    /// CHECK: handled by aaob
    #[account(mut)]
    pub asks: AccountInfo<'info>,
}

pub fn handler<'a, 'b, 'info>(
    ctx: Context<'a, 'b, 'b, 'info, PlaceOrder<'info>>,
    side: OrderSide,
    params: OrderParams,
) -> Result<()> {
    let order_summary = place_order_blindly(
        Context::new(
            ctx.program_id,
            ctx.accounts,
            ctx.remaining_accounts,
            BTreeMap::new(),
        ),
        side,
        params,
        CallbackFlags::empty(),
    )?;

    match side {
        OrderSide::Borrow => {
            ctx.accounts
                .orderbook_user_account
                .bond_tickets_stored
                .try_sub_assign(order_summary.total_base_qty)?;
        }
        OrderSide::Lend => {
            ctx.accounts
                .orderbook_user_account
                .underlying_token_stored
                .try_sub_assign(order_summary.total_quote_qty)?;
        }
    }

    emit!(OrderPlaced {
        bond_manager: ctx.accounts.bond_manager.key(),
        orderbook_user: ctx.accounts.orderbook_user_account.key(),
        side,
        order_summary,
    });

    Ok(())
}

/// Places the order with the assumption that there are sufficient balances to cover the order
/// The caller is responsible for ensuring the order can be covered somehow (underlying/tickets/collateral/etc)
pub fn place_order_blindly(
    ctx: Context<PlaceOrder>,
    side: OrderSide,
    params: OrderParams,
    flags: CallbackFlags,
) -> Result<OrderSummary> {
    let OrderParams {
        max_bond_ticket_qty,
        max_underlying_token_qty,
        limit_price,
        match_limit,
        post_only,
        post_allowed,
        auto_stake: _,
    } = params;

    let orderbook_account_key = ctx.accounts.orderbook_user_account.key();
    let nonce = ctx.accounts.orderbook_user_account.nonce;
    let callback_info = match ctx.accounts.orderbook_user_account.event_adapter {
        key if key != Pubkey::default() => CallbackInfo::new_with_adapter(
            ctx.accounts.bond_manager.key(),
            orderbook_account_key,
            key,
            params.callback_flags() | flags,
            nonce,
        ),
        _ => CallbackInfo::new(
            ctx.accounts.bond_manager.key(),
            orderbook_account_key,
            params.callback_flags() | flags,
            nonce,
        ),
    };

    let orderbook_side = match side {
        OrderSide::Lend => Side::Bid,
        OrderSide::Borrow => Side::Ask,
    };

    ctx.accounts.orderbook_user_account.nonce += 1;

    // post to the orderbook
    let orderbook_accounts = new_order::Accounts {
        market: &ctx.accounts.orderbook_market_state.to_account_info(),
        event_queue: &ctx.accounts.event_queue.to_account_info(),
        bids: &ctx.accounts.bids.to_account_info(),
        asks: &ctx.accounts.asks.to_account_info(),
    };
    let orderbook_params = new_order::Params {
        max_base_qty: max_bond_ticket_qty,
        max_quote_qty: max_underlying_token_qty,
        limit_price,
        match_limit,
        side: orderbook_side,
        callback_info,
        post_only,
        post_allowed,
        self_trade_behavior: SelfTradeBehavior::AbortTransaction,
    };
    let order_summary = agnostic_orderbook::instruction::new_order::process::<CallbackInfo>(
        ctx.program_id,
        orderbook_accounts,
        orderbook_params,
    )?;

    Ok(order_summary)
}
