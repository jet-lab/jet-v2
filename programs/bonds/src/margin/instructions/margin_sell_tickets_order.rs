use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::{accessor::mint, Mint, Token, TokenAccount};

use crate::{orderbook::state::*, serialization::RemainingAccounts, BondsError};

#[derive(Accounts)]
pub struct MarginSellTicketsOrder<'info> {
    /// Signing authority over the ticket vault transferring for a borrow order
    pub user: Signer<'info>,

    /// Account containing the bond tickets being sold
    #[account(mut, constraint =
        mint(&user_ticket_vault.to_account_info()).unwrap()
        == bond_ticket_mint.key() @ BondsError::WrongTicketMint
    )]
    pub user_ticket_vault: Account<'info, TokenAccount>,

    /// The account to recieve the matched tokens
    #[account(mut, constraint =
        mint(&user_token_vault.to_account_info()).unwrap()
        == orderbook_mut.bond_manager.load().unwrap().underlying_token_mint.key() @ BondsError::WrongUnderlyingTokenMint
    )]
    pub user_token_vault: Account<'info, TokenAccount>,

    pub orderbook_mut: OrderbookMut<'info>,

    /// The market ticket mint
    #[account(mut, address = orderbook_mut.bond_manager.load().unwrap().bond_ticket_mint.key() @ BondsError::WrongTicketMint)]
    pub bond_ticket_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    // Optional event adapter account
    // pub event_adapter: AccountInfo<'info>,
}

pub fn handler(ctx: Context<MarginSellTicketsOrder>, params: OrderParams) -> Result<()> {
    let (_, order_summary) = ctx.accounts.orderbook_mut.place_order(
        ctx.accounts.user.key(),
        Side::Ask,
        params,
        ctx.accounts.user_token_vault.key(),
        ctx.accounts.user_ticket_vault.key(),
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
        CallbackFlags::empty(),
    )?;

    anchor_spl::token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Burn {
                mint: ctx.accounts.bond_ticket_mint.to_account_info(),
                from: ctx.accounts.user_ticket_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        order_summary.base_combined(),
    )?;
    emit!(crate::events::SellTicketsOrder {
        bond_market: ctx.accounts.orderbook_mut.bond_manager.key(),
        borrower: ctx.accounts.user.key(),
        order_summary: order_summary.summary(),
    });

    Ok(())
}
