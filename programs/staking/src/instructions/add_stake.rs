use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::events::{Note, StakeAdded};
use crate::spl_addin::{MaxVoterWeightRecord, VoterWeightRecord};
use crate::state::*;

#[derive(Accounts)]
pub struct AddStake<'info> {
    /// The stake pool to deposit stake into
    #[account(mut,
              has_one = stake_pool_vault,
              has_one = max_voter_weight_record)]
    pub stake_pool: Box<Account<'info, StakePool>>,

    /// The stake pool token vault
    #[account(mut)]
    pub stake_pool_vault: Box<Account<'info, TokenAccount>>,

    /// The account to own the stake being deposited
    #[account(mut,
              has_one = stake_pool,
              has_one = voter_weight_record)]
    pub stake_account: Box<Account<'info, StakeAccount>>,

    /// The voter weight to be updated
    #[account(mut)]
    pub voter_weight_record: Box<Account<'info, VoterWeightRecord>>,

    /// The max voter weight
    #[account(mut)]
    pub max_voter_weight_record: Box<Account<'info, MaxVoterWeightRecord>>,

    /// The depositor of the stake
    pub payer: Signer<'info>,

    /// The depositor's token account to taken the deposit from
    #[account(mut)]
    pub payer_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

impl<'info> AddStake<'info> {
    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.payer_token_account.to_account_info(),
                to: self.stake_pool_vault.to_account_info(),
                authority: self.payer.to_account_info(),
            },
        )
    }
}

/// handler handler
pub fn add_stake_handler(ctx: Context<AddStake>, amount: Option<u64>) -> Result<()> {
    let stake_pool = &mut ctx.accounts.stake_pool;
    let stake_account = &mut ctx.accounts.stake_account;
    let voter_weight = &mut ctx.accounts.voter_weight_record;
    let max_weight = &mut ctx.accounts.max_voter_weight_record;

    stake_pool.update_vault(ctx.accounts.stake_pool_vault.amount);

    let token_amount = match amount {
        Some(n) => n,
        None => ctx.accounts.payer_token_account.amount,
    };

    let full_amount = stake_pool.deposit(stake_account, token_amount);
    stake_account.update_voter_weight_record(voter_weight);
    stake_pool.update_max_vote_weight_record(max_weight);

    let stake_pool = &ctx.accounts.stake_pool;
    let stake_account = &ctx.accounts.stake_account;
    let voter_weight = &ctx.accounts.voter_weight_record;
    let max_weight = &ctx.accounts.max_voter_weight_record;

    token::transfer(ctx.accounts.transfer_context(), full_amount.token_amount)?;

    emit!(StakeAdded {
        stake_pool: stake_pool.key(),
        stake_account: stake_account.key(),
        owner: stake_account.owner,
        depositor: ctx.accounts.payer.key(),

        staked_amount: full_amount,

        pool_note: stake_pool.note(),
        account_note: stake_account.note(),

        voter_weight: voter_weight.voter_weight,
        max_voter_weight: max_weight.max_voter_weight,
    });

    Ok(())
}
