use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::events::{Note, UnbondCancelled};
use crate::spl_addin::{MaxVoterWeightRecord, VoterWeightRecord};
use crate::state::*;

#[derive(Accounts)]
pub struct CancelUnbond<'info> {
    /// The owner of the stake account
    pub owner: Signer<'info>,

    /// The rent receiver
    /// CHECK:
    pub receiver: AccountInfo<'info>,

    /// The account owning the stake to be rebonded
    #[account(mut,
              has_one = owner,
              has_one = stake_pool,
              has_one = voter_weight_record)]
    pub stake_account: Account<'info, StakeAccount>,

    /// The voter weight to be updated
    #[account(mut)]
    pub voter_weight_record: Box<Account<'info, VoterWeightRecord>>,

    /// The max voter weight
    #[account(mut)]
    pub max_voter_weight_record: Box<Account<'info, MaxVoterWeightRecord>>,

    /// The stake pool to be rebonded to
    #[account(mut,
              has_one = stake_pool_vault,
              has_one = max_voter_weight_record)]
    pub stake_pool: Account<'info, StakePool>,

    /// The stake pool token vault
    pub stake_pool_vault: Account<'info, TokenAccount>,

    /// The account to record this unbonding request
    #[account(mut,
              close = receiver,
              has_one = stake_account)]
    pub unbonding_account: Account<'info, UnbondingAccount>,
}

pub fn cancel_unbond_handler(ctx: Context<CancelUnbond>) -> Result<()> {
    let stake_pool = &mut ctx.accounts.stake_pool;
    let stake_account = &mut ctx.accounts.stake_account;
    let unbonding_account = &mut ctx.accounts.unbonding_account;
    let voter_weight = &mut ctx.accounts.voter_weight_record;
    let max_weight = &mut ctx.accounts.max_voter_weight_record;

    stake_pool.update_vault(ctx.accounts.stake_pool_vault.amount);
    let cancelled_amount = stake_pool.rebond(stake_account, unbonding_account);

    stake_account.update_voter_weight_record(voter_weight);
    stake_pool.update_max_vote_weight_record(max_weight);

    emit!(UnbondCancelled {
        stake_pool: stake_pool.key(),
        stake_account: stake_account.key(),
        unbonding_account: unbonding_account.key(),
        owner: ctx.accounts.owner.key(),

        cancelled_amount,

        pool_note: stake_pool.note(),
        account_note: stake_account.note(),

        voter_weight: voter_weight.voter_weight,
        max_voter_weight: max_weight.max_voter_weight,
    });

    Ok(())
}
