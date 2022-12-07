use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    control::state::Market, market_token_manager::MarketTokenManager,
    tickets::events::TokensExchanged, ErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct ExchangeTokens<'info> {
    /// The Market manages asset tokens for a particular tenor
    #[account(
            has_one = market_ticket_mint @ ErrorCode::WrongTicketMint,
            has_one = underlying_token_vault @ ErrorCode::WrongVault,
    )]
    pub market: AccountLoader<'info, Market>,

    /// The vault stores the tokens of the underlying asset managed by the Market
    #[account(mut)]
    pub underlying_token_vault: Box<Account<'info, TokenAccount>>,

    /// The minting account for the tickets
    #[account(mut)]
    pub market_ticket_mint: Account<'info, Mint>,

    /// The token account to receive the exchanged tickets
    #[account(mut)]
    pub user_market_ticket_vault: Account<'info, TokenAccount>,

    /// The user controlled token account to exchange for tickets
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
    ctx.mint(
        &ctx.accounts.market_ticket_mint,
        &ctx.accounts.user_market_ticket_vault,
        amount,
    )?;

    emit!(TokensExchanged {
        market: ctx.accounts.market.key(),
        user: ctx.accounts.user_authority.key(),
        amount,
    });

    Ok(())
}
