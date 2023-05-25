use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::{accessor::mint, Mint, Token, TokenAccount};
use jet_margin::MarginAccount;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    events::OrderType,
    margin::state::MarginUser,
    market_token_manager::MarketTokenManager,
    orderbook::{instructions::sell_tickets_order::*, state::*},
    serialization::RemainingAccounts,
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct MarginSellTicketsOrder<'info> {
    /// The account tracking borrower debts
    #[account(mut,
        has_one = margin_account @ FixedTermErrorCode::WrongMarginUserAuthority,
        has_one = ticket_collateral @ FixedTermErrorCode::WrongTicketCollateralAccount,
        constraint = margin_user.market == orderbook_mut.market.key() @ FixedTermErrorCode::UserNotInMarket,
    )]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub ticket_collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    /// CHECK: instruction logic
    #[account(mut)]
    pub ticket_collateral_mint: AccountInfo<'info>,

    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// Account containing the tickets being sold
    #[account(mut, constraint =
        mint(&user_ticket_vault.to_account_info()).unwrap()
        == ticket_mint.key() @ FixedTermErrorCode::WrongTicketMint
    )]
    pub user_ticket_vault: Account<'info, TokenAccount>,

    /// The account to receive the matched tokens
    #[account(mut, constraint =
        mint(&user_token_vault.to_account_info()).unwrap()
        == orderbook_mut.market.load().unwrap().underlying_token_mint.key() @ FixedTermErrorCode::WrongUnderlyingTokenMint
    )]
    pub user_token_vault: Account<'info, TokenAccount>,

    #[market]
    pub orderbook_mut: OrderbookMut<'info>,

    /// The ticket mint
    #[account(mut, address = orderbook_mut.market.load().unwrap().ticket_mint.key() @ FixedTermErrorCode::WrongTicketMint)]
    pub ticket_mint: Account<'info, Mint>,

    /// The token vault holding the underlying token of the ticket
    #[account(mut, address = orderbook_mut.market.load().unwrap().underlying_token_vault.key() @ FixedTermErrorCode::WrongTicketMint)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<MarginSellTicketsOrder>, params: OrderParams) -> Result<()> {
    let (info, order_summary) = ctx.accounts.orderbook_mut.place_margin_order(
        Side::Ask,
        params,
        ctx.accounts.margin_account.key(),
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

    let user = ctx.accounts.margin_user.key();
    let a = ctx.accounts;
    SellTicketsAccounts {
        authority: a.margin_account.as_ref(),
        user_ticket_vault: a.user_ticket_vault.as_ref(),
        user_token_vault: a.user_token_vault.as_ref(),
        orderbook_mut: &a.orderbook_mut,
        ticket_mint: a.ticket_mint.as_ref(),
        underlying_token_vault: a.underlying_token_vault.as_ref(),
        token_program: &a.token_program,
    }
    .sell_tickets(
        info.order_tag.as_u128(),
        order_summary,
        &params,
        Some(user),
        OrderType::MarginSellTickets,
    )
}
