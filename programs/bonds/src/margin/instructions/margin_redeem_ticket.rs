use anchor_lang::prelude::*;
use proc_macros::BondTokenManager;

use crate::{
    margin::state::MarginUser, tickets::instructions::redeem_ticket::*, utils::burn_notes,
    BondsError,
};

#[derive(Accounts, BondTokenManager)]
pub struct MarginRedeemTicket<'info> {
    #[account(mut,
		constraint = margin_user.margin_account == inner.authority.key() @ BondsError::WrongMarginUserAuthority,
        has_one = collateral,
	)]
    pub margin_user: Account<'info, MarginUser>,

    /// Token account used by the margin program to track the collateral value of assets custodied by bonds
    #[account(mut)]
    pub collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the collateral value of assets custodied by bonds
    #[account(mut, address = inner.bond_manager.load()?.collateral_mint)]
    pub collateral_mint: AccountInfo<'info>,

    #[bond_manager]
    #[token_program]
    pub inner: RedeemTicket<'info>,
}

pub fn handler(ctx: Context<MarginRedeemTicket>) -> Result<()> {
    let redeemed = ctx
        .accounts
        .inner
        .redeem(ctx.accounts.inner.authority.key())?;
    ctx.accounts.margin_user.assets.redeem_staked_tickets(redeemed);
    burn_notes!(ctx, collateral_mint, collateral, redeemed)?;

    Ok(())
}
