use anchor_lang::prelude::*;
use jet_margin::MarginAccount;

use crate::margin::state::{MarginUser, TermLoan, TermLoanFlags};

#[derive(Accounts)]
pub struct StopAutoRollLoan<'info> {
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

pub fn handler(ctx: Context<StopAutoRollLoan>) -> Result<()> {
    ctx.accounts.loan.flags.remove(TermLoanFlags::AUTO_ROLL);

    Ok(())
}
