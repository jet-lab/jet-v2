use anchor_lang::prelude::*;
use anchor_spl::token::Transfer;
use anchor_spl::token::{self, CloseAccount, Token, TokenAccount};

use crate::events::AwardRevoked;
use crate::state::*;

#[derive(Accounts)]
pub struct AwardRevoke<'info> {
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

    /// The account to receive any remaining tokens
    /// CHECK:
    #[account(mut)]
    pub token_receiver: UncheckedAccount<'info>,

    /// The authority with permission to close the award
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

impl<'info> AwardRevoke<'info> {
    fn transfer_remaining_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                to: self.token_receiver.to_account_info(),
                from: self.vault.to_account_info(),
                authority: self.award.to_account_info(),
            },
        )
    }

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

pub fn award_revoke_handler(ctx: Context<AwardRevoke>) -> Result<()> {
    let award = &ctx.accounts.award;

    let vault_amount = ctx.accounts.vault.amount;
    let total_released = award.distributed;
    let unreleased_amount = award.target_amount - total_released;

    token::transfer(
        ctx.accounts
            .transfer_remaining_context()
            .with_signer(&[&award.signer_seeds()]),
        vault_amount,
    )?;

    token::close_account(
        ctx.accounts
            .close_vault_context()
            .with_signer(&[&award.signer_seeds()]),
    )?;

    emit!(AwardRevoked {
        award: award.key(),

        unreleased_amount,
        total_released,

        vault_amount,
    });

    Ok(())
}
