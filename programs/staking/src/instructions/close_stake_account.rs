use anchor_lang::prelude::*;

use crate::events::StakeAccountClosed;
use crate::spl_addin::VoterWeightRecord;
use crate::state::*;

#[derive(Accounts)]
pub struct CloseStakeAccount<'info> {
    /// The owner of the stake account
    pub owner: Signer<'info>,

    /// The receiver for the rent recovered
    /// CHECK:
    #[account(mut)]
    pub closer: AccountInfo<'info>,

    /// The empty stake account to be closed
    #[account(mut,
              close = closer,
              has_one = owner)]
    pub stake_account: Account<'info, StakeAccount>,

    /// The voter weight
    #[account(mut,
              close = closer,
              has_one = owner)]
    pub voter_weight_record: Account<'info, VoterWeightRecord>,
}

pub fn close_stake_account_handler(ctx: Context<CloseStakeAccount>) -> Result<()> {
    let stake_account = &ctx.accounts.stake_account;

    assert!(stake_account.bonded_shares == 0);
    assert!(stake_account.unbonding_shares == 0);

    emit!(StakeAccountClosed {
        stake_account: stake_account.key(),
        owner: ctx.accounts.owner.key(),
    });

    Ok(())
}
