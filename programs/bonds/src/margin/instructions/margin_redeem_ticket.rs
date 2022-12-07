use anchor_lang::prelude::*;
use jet_program_proc_macros::BondTokenManager;

use crate::{
    bond_token_manager::BondTokenManager, events::AssetsUpdated, margin::state::MarginUser,
    tickets::instructions::redeem_ticket::*, BondsError,
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
    ctx.accounts
        .margin_user
        .assets
        .redeem_staked_tickets(redeemed);
    ctx.burn_notes(
        &ctx.accounts.collateral_mint,
        &ctx.accounts.collateral,
        redeemed,
    )?;

    emit!(AssetsUpdated::from((
        &ctx.accounts.margin_user.assets,
        ctx.accounts.margin_user.key()
    )));

    Ok(())
}
