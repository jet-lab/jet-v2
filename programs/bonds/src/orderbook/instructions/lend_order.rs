use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::{accessor::mint, TokenAccount};

use crate::{orderbook::state::*, serialization::RemainingAccounts, BondsError};

#[derive(Accounts)]
pub struct LendOrder<'info> {
    /// Signing authority over the token vault transferring for a lend order
    pub user: Signer<'info>,

    /// If auto stake is not enabled, the ticket account that will recieve the bond tickets
    #[account(mut, constraint =
        mint(&lender_tickets.to_account_info()).unwrap()
        == orderbook_mut.bond_manager.load().unwrap().bond_ticket_mint.key() @ BondsError::WrongTicketMint
    )]
    pub lender_tickets: Account<'info, TokenAccount>,

    pub orderbook_mut: OrderbookMut<'info>,

    #[account(
        constraint = lend.lender_mint() == orderbook_mut.underlying_mint() @ BondsError::WrongUnderlyingTokenMint,
        constraint = lend.vault() == orderbook_mut.vault() @ BondsError::WrongVault,
    )]
    pub lend: Lend<'info>,
    // Optional event adapter account
    // pub event_adapter: AccountInfo<'info>,
}

pub fn handler(ctx: Context<LendOrder>, params: OrderParams, seed: Vec<u8>) -> Result<()> {
    let (callback_info, order_summary) = ctx.accounts.orderbook_mut.place_order(
        ctx.accounts.user.key(),
        Side::Bid,
        params,
        if params.auto_stake {
            ctx.accounts.user.key()
        } else {
            ctx.accounts.lender_tickets.key()
        },
        ctx.accounts.lend.lender_tokens.key(),
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
        CallbackFlags::empty(),
    )?;
    ctx.accounts.lend.lend(
        ctx.accounts.user.key(),
        ctx.accounts.user.to_account_info(),
        &seed,
        callback_info,
        &order_summary,
        &ctx.accounts.orderbook_mut.bond_manager,
    )?;
    emit!(crate::events::LendOrder {
        bond_market: ctx.accounts.orderbook_mut.bond_manager.key(),
        lender: ctx.accounts.user.key(),
        order_summary: order_summary.summary(),
    });

    Ok(())
}
