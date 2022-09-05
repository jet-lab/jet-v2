use anchor_lang::prelude::*;
use anchor_spl::token::{burn, transfer, Burn, Mint, Token, TokenAccount, Transfer};
use jet_proto_math::traits::TryAddAssign;

use crate::{
    control::state::BondManager,
    events::OrderbookDeposit,
    orderbook::state::{user::OrderbookUser, AssetKind},
    BondsError,
};

#[derive(Accounts)]
pub struct Deposit<'info> {
    /// The account tracking information related to this particular user
    #[account(
        mut,
        has_one = bond_manager @ BondsError::UserNotInMarket,
    )]
    pub orderbook_user_account: Account<'info, OrderbookUser>,

    /// The `BondManager` account tracks global information related to this particular bond market
    #[account(
        has_one = bond_ticket_mint @ BondsError::WrongTicketMint,
        has_one = underlying_token_vault @ BondsError::WrongVault,
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,

    /// The token vault to deposit tokens from
    #[account(mut)]
    pub user_token_vault: Account<'info, TokenAccount>,

    /// The signing authority for the user_token_vault
    pub user_token_vault_authority: Signer<'info>,

    /// The token vault holding the underlying token of the bond
    #[account(mut)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

    /// The minting account for the bond tickets
    #[account(mut)]
    pub bond_ticket_mint: Box<Account<'info, Mint>>,

    /// SPL token program
    pub token_program: Program<'info, Token>,
}

impl<'info> Deposit<'info> {
    pub fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_token_vault.to_account_info(),
                to: self.underlying_token_vault.to_account_info(),
                authority: self.user_token_vault_authority.to_account_info(),
            },
        )
    }
    pub fn burn_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                mint: self.bond_ticket_mint.to_account_info(),
                from: self.user_token_vault.to_account_info(),
                authority: self.user_token_vault_authority.to_account_info(),
            },
        )
    }
}

pub fn handler(ctx: Context<Deposit>, amount: u64, kind: AssetKind) -> Result<()> {
    match kind {
        AssetKind::BondTicket => {
            burn(ctx.accounts.burn_context(), amount)?;
            ctx.accounts
                .orderbook_user_account
                .bond_tickets_stored
                .try_add_assign(amount)?;
        }
        AssetKind::UnderlyingToken => {
            transfer(ctx.accounts.transfer_context(), amount)?;
            ctx.accounts
                .orderbook_user_account
                .underlying_token_stored
                .try_add_assign(amount)?;
        }
    };

    emit!(OrderbookDeposit {
        bond_manager: ctx.accounts.bond_manager.key(),
        orderbook_user: ctx.accounts.orderbook_user_account.key(),
        amount,
        kind
    });
    Ok(())
}
