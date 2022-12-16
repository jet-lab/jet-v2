use anchor_lang::prelude::*;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    margin::state::MarginUser, market_token_manager::MarketTokenManager,
    tickets::instructions::redeem_ticket::*, FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct MarginRedeemTicket<'info> {
    #[account(mut,
		constraint = margin_user.margin_account == inner.authority.key() @ FixedTermErrorCode::WrongMarginUserAuthority,
        has_one = ticket_collateral,
	)]
    pub margin_user: Account<'info, MarginUser>,

    /// Token account used by the margin program to track the collateral value of assets custodied by fixed-term market
    #[account(mut)]
    pub ticket_collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the collateral value of assets custodied by fixed-term market
    #[account(mut, address = inner.market.load()?.ticket_collateral_mint)]
    pub ticket_collateral_mint: AccountInfo<'info>,

    #[market]
    #[token_program]
    pub inner: RedeemTicket<'info>,
}

pub fn handler(ctx: Context<MarginRedeemTicket>) -> Result<()> {
    let redeemed = ctx
        .accounts
        .inner
        .redeem(ctx.accounts.inner.authority.key())?;
    ctx.accounts
        .margin_user
        .assets
        .redeem_staked_tickets(redeemed);
    ctx.burn_notes(
        &ctx.accounts.ticket_collateral_mint,
        &ctx.accounts.ticket_collateral,
        redeemed,
    )?;

    ctx.accounts.margin_user.emit_asset_balances();

    Ok(())
}
