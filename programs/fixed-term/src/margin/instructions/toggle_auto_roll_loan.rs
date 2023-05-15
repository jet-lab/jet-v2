use anchor_lang::prelude::*;
use jet_margin::MarginAccount;

use crate::{
    events::TermLoanFlagsToggled,
    margin::state::{MarginUser, TermLoan, TermLoanFlags},
};

#[derive(Accounts)]
pub struct ToggleAutoRollLoan<'info> {
    /// The signing authority for this user account
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The fixed-term market state for the user
    #[account(
        mut,
        has_one = margin_account,
    )]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// The fixed-term market this user belongs to
    #[account(
        mut,
        has_one = margin_user,
    )]
    pub loan: Account<'info, TermLoan>,
}

pub fn handler(ctx: Context<ToggleAutoRollLoan>) -> Result<()> {
    ctx.accounts.loan.flags.toggle(TermLoanFlags::AUTO_ROLL);

    emit!(TermLoanFlagsToggled {
        margin_account: ctx.accounts.margin_account.key(),
        margin_user: ctx.accounts.margin_user.key(),
        term_loan: ctx.accounts.loan.key(),
        flags: ctx.accounts.loan.flags,
    });

    Ok(())
}
