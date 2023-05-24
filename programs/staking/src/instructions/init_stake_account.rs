use anchor_lang::prelude::*;
use jet_auth::UserAuthentication;

use crate::events::StakeAccountCreated;
use crate::seeds;
use crate::spl_addin::VoterWeightRecord;
use crate::state::*;

#[derive(Accounts)]
pub struct InitStakeAccount<'info> {
    /// The owner for the stake
    pub owner: Signer<'info>,

    /// The authentication account, which identifies that the given owner
    /// is actually allowed to use this program.
    #[account(has_one = owner,
              constraint = auth.allowed)]
    pub auth: Account<'info, UserAuthentication>,

    /// The stake pool to create an account with
    pub stake_pool: Account<'info, StakePool>,

    /// The new stake account
    #[account(
        init,
        payer = payer,
        seeds = [
            stake_pool.key().as_ref(),
            owner.key.as_ref()
        ],
        bump,
        space = 8 + std::mem::size_of::<StakeAccount>(),
    )]
    pub stake_account: Account<'info, StakeAccount>,

    /// The voter weight record to be created for this stake
    #[account(init,
              seeds = [
                  seeds::VOTER_WEIGHT_RECORD,
                  stake_account.key().as_ref()
              ],
              bump,
              payer = payer,
              space = 8 + std::mem::size_of::<VoterWeightRecord>())]
    pub voter_weight_record: Account<'info, VoterWeightRecord>,

    /// The address that will pay for the rent
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn init_stake_account_handler(ctx: Context<InitStakeAccount>) -> Result<()> {
    let pool = &ctx.accounts.stake_pool;
    let account = &mut ctx.accounts.stake_account;
    let voter_weight = &mut ctx.accounts.voter_weight_record;

    account.owner = *ctx.accounts.owner.key;
    account.stake_pool = pool.key();
    account.voter_weight_record = voter_weight.key();

    voter_weight.realm = pool.governance_realm;
    voter_weight.governing_token_mint = pool.token_mint;

    account.update_voter_weight_record(voter_weight);

    emit!(StakeAccountCreated {
        stake_pool: ctx.accounts.stake_pool.key(),
        stake_account: ctx.accounts.stake_account.key(),
        owner: ctx.accounts.owner.key(),
    });

    Ok(())
}
