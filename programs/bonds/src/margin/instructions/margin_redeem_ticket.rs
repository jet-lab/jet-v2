use anchor_lang::prelude::*;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    margin::state::MarginUser, market_token_manager::MarketTokenManager,
    tickets::instructions::redeem_ticket::*, ErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct MarginRedeemTicket<'info> {
    #[account(mut,
		constraint = margin_user.margin_account == inner.authority.key() @ ErrorCode::WrongMarginUserAuthority,
        has_one = collateral,
	)]
    pub margin_user: Account<'info, MarginUser>,

    /// Token account used by the margin program to track the collateral value of assets custodied by Jet markets
    #[account(mut)]
    pub collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the collateral value of assets custodied by Jet markets
    #[account(mut, address = inner.market_manager.load()?.collateral_mint)]
    pub collateral_mint: AccountInfo<'info>,

    #[market_manager]
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
        &ctx.accounts.collateral_mint,
        &ctx.accounts.collateral,
        redeemed,
    )?;

    Ok(())
}
