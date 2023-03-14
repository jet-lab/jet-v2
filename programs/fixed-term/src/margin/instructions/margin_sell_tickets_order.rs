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
        has_one = token_collateral @ FixedTermErrorCode::WrongTicketCollateralAccount,
    )]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub token_collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub token_collateral_mint: AccountInfo<'info>,

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
    let posted_token_value = order_summary.quote_posted(RoundingAction::PostLend)?;
    ctx.accounts.margin_user.sell_tickets(posted_token_value)?;
    ctx.mint(
        &ctx.accounts.token_collateral_mint,
        &ctx.accounts.token_collateral,
        posted_token_value,
    )?;

    ctx.accounts.inner.sell_tickets(
        info.order_tag.as_u128(),
        order_summary,
        &params,
        Some(ctx.accounts.margin_user.key()),
        OrderType::MarginSellTickets,
    )
}
