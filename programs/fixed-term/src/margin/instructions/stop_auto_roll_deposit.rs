use anchor_lang::prelude::*;
use jet_margin::MarginAccount;

use crate::{
    tickets::state::{TermDeposit, TermDepositFlags},
    FixedTermErrorCode,
};

#[derive(Accounts)]
pub struct StopAutoRollDeposit<'info> {
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

pub fn handler(ctx: Context<StopAutoRollDeposit>) -> Result<()> {
    ctx.accounts
        .deposit
        .flags
        .remove(TermDepositFlags::AUTO_ROLL);

    Ok(())
}
