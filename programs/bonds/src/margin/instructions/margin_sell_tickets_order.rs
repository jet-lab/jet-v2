use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use jet_program_proc_macros::BondTokenManager;

use crate::{
    bond_token_manager::BondTokenManager,
    margin::state::MarginUser,
    orderbook::{
        instructions::sell_tickets_order::*,
        state::{CallbackFlags, OrderParams},
    },
    serialization::RemainingAccounts,
    BondsError,
};

#[derive(Accounts, BondTokenManager)]
pub struct MarginSellTicketsOrder<'info> {
    /// The account tracking borrower debts
    #[account(mut,
        constraint = margin_user.margin_account == inner.authority.key() @ BondsError::UnauthorizedCaller,
        has_one = collateral @ BondsError::WrongCollateralAccount,
    )]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub collateral_mint: AccountInfo<'info>,

    #[bond_manager(orderbook_mut)]
    #[token_program]
    pub inner: SellTicketsOrder<'info>,
}

pub fn handler(ctx: Context<MarginSellTicketsOrder>, params: OrderParams) -> Result<()> {
    let (_, order_summary) = ctx.accounts.inner.orderbook_mut.place_order(
        ctx.accounts.inner.authority.key(),
        Side::Ask,
        params,
        ctx.accounts.margin_user.key(),
        ctx.accounts.margin_user.key(),
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
        CallbackFlags::MARGIN,
    )?;
    ctx.mint(
        &ctx.accounts.collateral_mint,
        &ctx.accounts.collateral,
        order_summary.quote_posted()?,
    )?;

    ctx.accounts.inner.sell_tickets(order_summary)
}
