use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::{
    control::state::BondManager, tickets::events::TokensExchanged, utils::mint_to, BondsError,
};

#[derive(Accounts)]
pub struct ExchangeTokens<'info> {
    /// The BondManager manages asset tokens for a particular bond duration
    #[account(
            has_one = bond_ticket_mint @ BondsError::WrongTicketMint,
            has_one = underlying_token_vault @ BondsError::WrongVault,
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,

    /// The vault stores the tokens of the underlying asset managed by the BondManager
    #[account(mut)]
    pub underlying_token_vault: Box<Account<'info, TokenAccount>>,

    /// The minting account for the bond tickets
    #[account(mut)]
    pub bond_ticket_mint: Account<'info, Mint>,

    /// The token account to recieve the exchanged bond tickets
    #[account(mut)]
    pub user_bond_ticket_vault: Account<'info, TokenAccount>,

    /// The user controlled token account to exchange for bond tickets
    #[account(mut)]
    pub user_underlying_token_vault: Account<'info, TokenAccount>,

    /// The signing authority in charge of the user's underlying token vault
    pub user_authority: Signer<'info>,

    /// SPL token program
    pub token_program: Program<'info, Token>,
}

impl<'info> ExchangeTokens<'info> {
    pub fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_underlying_token_vault.to_account_info(),
                to: self.underlying_token_vault.to_account_info(),
                authority: self.user_authority.to_account_info(),
            },
        )
    }
}

pub fn handler(ctx: Context<ExchangeTokens>, amount: u64) -> Result<()> {
    transfer(ctx.accounts.transfer_context(), amount)?;
    mint_to!(ctx, bond_ticket_mint, user_bond_ticket_vault, amount)?;

    emit!(TokensExchanged {
        bond_manager: ctx.accounts.bond_manager.key(),
        user: ctx.accounts.user_authority.key(),
        amount,
    });

    Ok(())
}
