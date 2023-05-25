use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use spl_governance::state::token_owner_record::TokenOwnerRecordV2;

use crate::events::{Note, StakeUnbonded};
use crate::spl_addin::{MaxVoterWeightRecord, VoterWeightRecord};
use crate::state::*;
use crate::ErrorCode;

#[derive(Accounts)]
#[instruction(seed: u32)]
pub struct UnbondStake<'info> {
    /// The owner of the stake account
    pub owner: Signer<'info>,

    /// The payer for rent
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The account owning the stake to be unbonded
    #[account(mut,
              has_one = owner,
              has_one = stake_pool,
              has_one = voter_weight_record)]
    pub stake_account: Box<Account<'info, StakeAccount>>,

    /// The stake pool to be unbonded from
    #[account(mut,
              has_one = stake_pool_vault,
              has_one = max_voter_weight_record)]
    pub stake_pool: Box<Account<'info, StakePool>>,

    /// The stake pool token vault
    pub stake_pool_vault: Box<Account<'info, TokenAccount>>,

    /// The account to record this unbonding request
    #[account(
        init,
        payer = payer,
        seeds = [
            stake_account.key().as_ref(),
            seed.to_le_bytes().as_ref()
        ],
        bump,
        space = 8 + std::mem::size_of::<UnbondingAccount>(),
    )]
    pub unbonding_account: Box<Account<'info, UnbondingAccount>>,

    /// The voter weight to be updated
    #[account(mut)]
    pub voter_weight_record: Box<Account<'info, VoterWeightRecord>>,

    /// The max voter weight
    #[account(mut)]
    pub max_voter_weight_record: Box<Account<'info, MaxVoterWeightRecord>>,

    /// The token owner record for the owner of the stake
    /// CHECK: This has to be validated that its correct for the owner,
    ///        and is owned by the governance program
    pub token_owner_record: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> UnbondStake<'info> {
    fn read_token_owner_record(&self) -> Result<TokenOwnerRecordV2> {
        let record = spl_governance_tools::account::get_account_data::<TokenOwnerRecordV2>(
            &crate::spl_governance::ID,
            &self.token_owner_record,
        )?;

        if record.governing_token_owner != *self.owner.key {
            return err!(ErrorCode::InvalidTokenOwnerRecord);
        }

        if record.realm != self.stake_pool.governance_realm {
            return err!(ErrorCode::InvalidTokenOwnerRecord);
        }

        if record.governing_token_mint != self.stake_pool.token_mint {
            return err!(ErrorCode::InvalidTokenOwnerRecord);
        }

        Ok(record)
    }
}

pub fn unbond_stake_handler(
    ctx: Context<UnbondStake>,
    _seed: u32,
    amount: Option<u64>,
) -> Result<()> {
    let gov_owner_record = ctx.accounts.read_token_owner_record()?;
    let stake_pool = &mut ctx.accounts.stake_pool;
    let stake_account = &mut ctx.accounts.stake_account;
    let unbonding_account = &mut ctx.accounts.unbonding_account;
    let voter_weight = &mut ctx.accounts.voter_weight_record;
    let max_weight = &mut ctx.accounts.max_voter_weight_record;
    let clock = Clock::get()?;

    // User can't have any outstanding votes at the time of unbond
    gov_owner_record
        .assert_can_withdraw_governing_tokens()
        .map_err(|_| error!(ErrorCode::OutstandingVotes))?;

    unbonding_account.stake_account = stake_account.key();
    unbonding_account.unbonded_at = clock.unix_timestamp + stake_pool.unbond_period;

    stake_pool.update_vault(ctx.accounts.stake_pool_vault.amount);
    let unbonded_amount = stake_pool.unbond(stake_account, unbonding_account, amount)?;

    stake_account.update_voter_weight_record(voter_weight);
    stake_pool.update_max_vote_weight_record(max_weight);

    emit!(StakeUnbonded {
        stake_pool: stake_pool.key(),
        stake_account: stake_account.key(),
        unbonding_account: unbonding_account.key(),
        owner: ctx.accounts.owner.key(),

        unbonded_amount,
        unbonded_at: unbonding_account.unbonded_at,

        pool_note: stake_pool.note(),
        account_note: stake_account.note(),

        voter_weight: voter_weight.voter_weight,
        max_voter_weight: max_weight.max_voter_weight,
    });

    Ok(())
}
