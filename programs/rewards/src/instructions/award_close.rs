use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Token, TokenAccount};

use crate::events;
use crate::state::*;
use crate::ErrorCode;

#[derive(Accounts)]
pub struct AwardClose<'info> {
    /// The award to be closed
    #[account(mut,
              close = receiver,
              has_one = authority,
              has_one = vault)]
    pub award: Account<'info, Award>,

    /// The vault for the award
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    /// The account to receive the rent
    /// CHECK:
    #[account(mut)]
    pub receiver: UncheckedAccount<'info>,

    /// The authority with permission to close the award
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

impl<'info> AwardClose<'info> {
    fn close_vault_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            CloseAccount {
                account: self.vault.to_account_info(),
                destination: self.receiver.to_account_info(),
                authority: self.award.to_account_info(),
            },
        )
    }
}

pub fn award_close_handler(ctx: Context<AwardClose>) -> Result<()> {
    let award = &ctx.accounts.award;
    let clock = Clock::get()?;

    if award.end_at > (clock.unix_timestamp as u64) {
        msg!("award is not yet fully vested");
        return Err(ErrorCode::AwardNotFullyVested.into());
    }

    token::close_account(
        ctx.accounts
            .close_vault_context()
            .with_signer(&[&award.signer_seeds()]),
    )?;

    emit!(events::AwardClosed { award: award.key() });

    Ok(())
}
