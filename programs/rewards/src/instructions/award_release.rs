use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::{events, state::*};
use jet_staking::cpi::accounts::AddStake;
use jet_staking::program::JetStaking;

#[derive(Accounts)]
pub struct AwardRelease<'info> {
    /// The account storing the award info
    #[account(mut,
              has_one = vault,
              has_one = stake_account)]
    pub award: Account<'info, Award>,

    /// The account storing the tokens to be distributed
    /// CHECK:
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    /// The account to transfer the distributed tokens to
    /// CHECK:
    #[account(mut)]
    pub stake_account: UncheckedAccount<'info>,

    /// The voter weight for the stake account
    /// CHECK:
    #[account(mut)]
    pub voter_weight_record: AccountInfo<'info>,

    /// The max voter weight
    /// CHECK:
    #[account(mut)]
    pub max_voter_weight_record: AccountInfo<'info>,

    /// The stake pool the account is part of
    /// CHECK:
    #[account(mut)]
    pub stake_pool: UncheckedAccount<'info>,

    /// The token vault for the pool
    /// CHECK:
    #[account(mut)]
    pub stake_pool_vault: UncheckedAccount<'info>,

    pub staking_program: Program<'info, JetStaking>,
    pub token_program: Program<'info, Token>,
}

impl<'info> AwardRelease<'info> {
    fn add_stake_context(&self) -> CpiContext<'_, '_, '_, 'info, AddStake<'info>> {
        CpiContext::new(
            self.staking_program.to_account_info(),
            AddStake {
                stake_pool: self.stake_pool.to_account_info(),
                stake_pool_vault: self.stake_pool_vault.to_account_info(),
                stake_account: self.stake_account.to_account_info(),
                voter_weight_record: self.voter_weight_record.to_account_info(),
                max_voter_weight_record: self.max_voter_weight_record.to_account_info(),
                payer: self.award.to_account_info(),
                payer_token_account: self.vault.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}

pub fn award_release_handler(ctx: Context<AwardRelease>) -> Result<()> {
    let award = &mut ctx.accounts.award;
    let clock = Clock::get()?;

    let to_distribute = award.distribute(clock.unix_timestamp as u64);
    let award = &ctx.accounts.award;

    jet_staking::cpi::add_stake(
        ctx.accounts
            .add_stake_context()
            .with_signer(&[&award.signer_seeds()]),
        Some(to_distribute),
    )?;

    emit!(events::AwardReleased {
        award: award.key(),
        amount_released: to_distribute,
        total_released: award.distributed,

        vault_balance: ctx.accounts.vault.amount,
    });

    Ok(())
}
