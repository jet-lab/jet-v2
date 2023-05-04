use anchor_lang::prelude::*;
use jet_margin::MarginAccount;

use crate::{
    events::TermDepositFlagsToggled,
    tickets::state::{TermDeposit, TermDepositFlags},
    FixedTermErrorCode,
};

#[derive(Accounts)]
pub struct ToggleAutoRollDeposit<'info> {
    /// The signing authority for this user account
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The fixed-term market this user belongs to
    #[account(
        mut,
        constraint = margin_account.key() == deposit.owner @ FixedTermErrorCode::WrongDepositOwner,
    )]
    pub deposit: Account<'info, TermDeposit>,
}

pub fn handler(ctx: Context<ToggleAutoRollDeposit>) -> Result<()> {
    ctx.accounts
        .deposit
        .flags
        .toggle(TermDepositFlags::AUTO_ROLL);

    emit!(TermDepositFlagsToggled {
        margin_account: ctx.accounts.margin_account.key(),
        term_deposit: ctx.accounts.deposit.key(),
        flags: ctx.accounts.deposit.flags,
    });

    Ok(())
}
