use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{events, state::*};

#[derive(Accounts)]
pub struct DistributionRelease<'info> {
    /// The account storing the distribution info
    #[account(mut,
              has_one = vault,
              has_one = target_account)]
    pub distribution: Account<'info, Distribution>,

    /// The account storing the tokens to be distributed
    /// CHECK:
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    /// The account to transfer the distributed tokens to
    /// CHECK:
    #[account(mut)]
    pub target_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

impl<'info> DistributionRelease<'info> {
    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.vault.to_account_info(),
                to: self.target_account.to_account_info(),
                authority: self.distribution.to_account_info(),
            },
        )
    }
}

pub fn distribution_release_handler(ctx: Context<DistributionRelease>) -> Result<()> {
    let distribution = &mut ctx.accounts.distribution;
    let clock = Clock::get()?;

    let to_distribute = distribution.distribute(clock.unix_timestamp as u64);
    let distribution = &ctx.accounts.distribution;

    token::transfer(
        ctx.accounts
            .transfer_context()
            .with_signer(&[&distribution.signer_seeds()]),
        to_distribute,
    )?;

    emit!(events::DistributionReleased {
        distribution: distribution.key(),
        amount_released: to_distribute,
        total_distributed: distribution.distributed,
        vault_balance: ctx.accounts.vault.amount,
    });

    Ok(())
}
