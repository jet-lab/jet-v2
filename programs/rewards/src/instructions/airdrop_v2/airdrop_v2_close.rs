use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Token, TokenAccount, Transfer};

use crate::ErrorCode;
use crate::{events, state::*};

#[derive(Accounts)]
pub struct AirdropV2Close<'info> {
    /// The airdrop to claim from
    #[account(mut,
              has_one = authority,
              has_one = vault,
              close = receiver)]
    pub airdrop: AccountLoader<'info, AirdropMetadata>,

    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    /// The authority to make changes to the airdrop, which must sign
    pub authority: Signer<'info>,

    /// The account to received the rent recovered
    #[account(mut)]
    pub receiver: UncheckedAccount<'info>,

    /// The account to receive any remaining tokens in the vault
    #[account(mut)]
    pub token_receiver: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> AirdropV2Close<'info> {
    fn transfer_remaining_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                to: self.token_receiver.to_account_info(),
                from: self.vault.to_account_info(),
                authority: self.airdrop.to_account_info(),
            },
        )
    }

    fn close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            CloseAccount {
                account: self.vault.to_account_info(),
                destination: self.receiver.to_account_info(),
                authority: self.airdrop.to_account_info(),
            },
        )
    }
}

pub fn airdrop_v2_close_handler(ctx: Context<AirdropV2Close>) -> Result<()> {
    let airdrop = ctx.accounts.airdrop.load()?;
    let clock = Clock::get()?;
    let vault_amount = ctx.accounts.vault.amount;

    if airdrop.expire_at > clock.unix_timestamp {
        msg!("airdrop not expired");
        return Err(ErrorCode::AirdropExpired.into());
    }

    // transfer remaining tokens somewhere else
    token::transfer(
        ctx.accounts
            .transfer_remaining_context()
            .with_signer(&[&airdrop.signer_seeds()]),
        vault_amount,
    )?;

    // close out the vault to recover rent
    token::close_account(
        ctx.accounts
            .close_context()
            .with_signer(&[&airdrop.signer_seeds()]),
    )?;

    emit!(events::AirdropClosed {
        airdrop: ctx.accounts.airdrop.key(),
        vault_amount,
    });

    Ok(())
}
