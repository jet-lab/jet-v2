use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Token, TokenAccount};

use crate::events;
use crate::state::*;
use crate::ErrorCode;

#[derive(Accounts)]
pub struct DistributionClose<'info> {
    /// The distribution to be closed
    #[account(mut,
              close = receiver,
              has_one = authority,
              has_one = vault)]
    pub distribution: Account<'info, Distribution>,

    /// The vault for the distribution
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    /// The account to receive the rent
    /// CHECK:
    #[account(mut)]
    pub receiver: UncheckedAccount<'info>,

    /// The authority with permission to close the distribution
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

impl<'info> DistributionClose<'info> {
    fn close_vault_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            CloseAccount {
                account: self.vault.to_account_info(),
                destination: self.receiver.to_account_info(),
                authority: self.distribution.to_account_info(),
            },
        )
    }
}

pub fn distribution_close_handler(ctx: Context<DistributionClose>) -> Result<()> {
    let distribution = &ctx.accounts.distribution;
    let clock = Clock::get()?;

    if distribution.end_at > (clock.unix_timestamp as u64) {
        msg!("distribution has not ended yet");
        return Err(ErrorCode::DistributionNotEnded.into());
    }

    token::close_account(
        ctx.accounts
            .close_vault_context()
            .with_signer(&[&distribution.signer_seeds()]),
    )?;

    emit!(events::DistributionClosed {
        distribution: distribution.key(),
    });

    Ok(())
}
