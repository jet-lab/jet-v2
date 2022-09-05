use anchor_lang::prelude::*;

use crate::{
    margin::{
        events::ObligationMarkedDue,
        state::{MarginUser, Obligation, ObligationFlags},
    },
    BondsError,
};

/// Mark an `Obligation` as due
/// utility for the `jet-margin` liquidator
#[derive(Accounts)]
pub struct MarkDue<'info> {
    /// The account tracking information related to this particular user
    pub borrower_account: Account<'info, MarginUser>,

    /// The `Obligation` account tracking debt
    #[account(has_one = borrower_account @ BondsError::UserDoesNotOwnAccount)]
    pub obligation: Account<'info, Obligation>,
}

pub fn handler(ctx: Context<MarkDue>) -> Result<()> {
    let obligation = &mut ctx.accounts.obligation;
    let timestamp = Clock::get()?.unix_timestamp;
    if obligation.maturation_timestamp >= timestamp {
        return Err(error!(BondsError::ImmatureBond));
    }
    if !obligation.flags.contains(ObligationFlags::MARKED_DUE) {
        ctx.accounts
            .borrower_account
            .debt
            .mark_due(obligation.balance)?;
        obligation.flags |= ObligationFlags::MARKED_DUE;
    }

    emit!(ObligationMarkedDue {
        obligation: obligation.key(),
        bond_manager: obligation.bond_manager,
        borrower_account: obligation.borrower_account,
        balance: obligation.balance,
        obligation_timestamp: obligation.maturation_timestamp,
        marked_due_timestamp: timestamp,
    });

    Ok(())
}
