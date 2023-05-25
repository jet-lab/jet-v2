use crate::events::BondedWithdrawn;
use crate::events::Note;
use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::Transfer;

use crate::state::*;

#[derive(Accounts)]
pub struct WithdrawBonded<'info> {
    /// The authority for the stake pool
    pub authority: Signer<'info>,

    /// The stake pool to withdraw from
    #[account(mut,
              has_one = authority,
              has_one = stake_pool_vault)]
    pub stake_pool: Account<'info, StakePool>,

    /// The receiver for the withdrawn tokens
    /// CHECK:
    #[account(mut)]
    pub token_receiver: UncheckedAccount<'info>,

    /// The stake pool token vault
    #[account(mut)]
    pub stake_pool_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> WithdrawBonded<'info> {
    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.stake_pool_vault.to_account_info(),
                to: self.token_receiver.to_account_info(),
                authority: self.stake_pool.to_account_info(),
            },
        )
    }
}

pub fn withdraw_bonded_handler(ctx: Context<WithdrawBonded>, amount: u64) -> Result<()> {
    let stake_pool = &mut ctx.accounts.stake_pool;

    stake_pool.update_vault(ctx.accounts.stake_pool_vault.amount);
    stake_pool.withdraw_bonded(amount);

    let stake_pool = &ctx.accounts.stake_pool;
    token::transfer(
        ctx.accounts
            .transfer_context()
            .with_signer(&[&stake_pool.signer_seeds()]),
        amount,
    )?;

    emit!(BondedWithdrawn {
        stake_pool: stake_pool.key(),

        withdrawn_amount: amount,

        pool_note: stake_pool.note(),
    });

    Ok(())
}
