use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    events::OrderType,
    margin::state::MarginUser,
    market_token_manager::MarketTokenManager,
    orderbook::{
        instructions::sell_tickets_order::*,
        state::{CallbackFlags, OrderParams, RoundingAction},
    },
    serialization::RemainingAccounts,
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct MarginSellTicketsOrder<'info> {
    /// The account tracking borrower debts
    #[account(mut,
        constraint = margin_user.margin_account == inner.authority.key() @ FixedTermErrorCode::UnauthorizedCaller,
        has_one = ticket_collateral @ FixedTermErrorCode::WrongTicketCollateralAccount,
    )]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub ticket_collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    /// CHECK: instruction logic
    #[account(mut)]
    pub ticket_collateral_mint: AccountInfo<'info>,

    #[market(orderbook_mut)]
    #[token_program]
    pub inner: SellTicketsOrder<'info>,
}

pub fn handler(ctx: Context<MarginSellTicketsOrder>, params: OrderParams) -> Result<()> {
    let (info, order_summary) = ctx.accounts.inner.orderbook_mut.place_margin_order(
        Side::Ask,
        params,
        ctx.accounts.inner.authority.key(),
        ctx.accounts.margin_user.key(),
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
        CallbackFlags::MARGIN,
    )?;

    // collateral accounting
    // The order might settle to either tickets or underlying. To be completely safe,
    // it needs to be priced as the less valuable one (tickets)
    // and counted as the less numerous one (underlying).
    let posted_value = order_summary.quote_posted(RoundingAction::PostBorrow.direction())?;
    ctx.accounts.margin_user.sell_tickets(posted_value)?;
    ctx.mint(
        &ctx.accounts.ticket_collateral_mint,
        &ctx.accounts.ticket_collateral,
        posted_value,
    )?;

    ctx.accounts.inner.sell_tickets(
        info.order_tag.as_u128(),
        order_summary,
        &params,
        Some(ctx.accounts.margin_user.key()),
        OrderType::MarginSellTickets,
    )
}
