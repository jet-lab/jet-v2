use anchor_lang::prelude::*;

use crate::tickets::{events::DepositTransferred, state::TermDeposit};

#[derive(Accounts)]
pub struct TransferDeposit<'info> {
    /// The deposit to transfer
    #[account(mut, has_one = owner)]
    pub deposit: Account<'info, TermDeposit>,

    /// The current owner of the deposit
    pub owner: AccountInfo<'info>,

    /// The authority with control over the deposit
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<TransferDeposit>, new_owner: Pubkey) -> Result<()> {
    ctx.accounts.deposit.owner = new_owner;

    emit!(DepositTransferred {
        deposit: ctx.accounts.deposit.key(),
        previous_owner: ctx.accounts.owner.key(),
        new_owner,
    });

    Ok(())
}
