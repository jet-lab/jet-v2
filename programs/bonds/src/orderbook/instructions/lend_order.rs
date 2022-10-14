use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::{accessor::mint, transfer, Token, TokenAccount};
use jet_proto_math::traits::SafeAdd;

use crate::{
    orderbook::state::*,
    serialization::{self, RemainingAccounts},
    tickets::state::SplitTicket,
    utils::transfer_context,
    BondsError,
};

#[derive(Accounts)]
pub struct LendOrder<'info> {
    /// Signing authority over the token vault transferring for a lend order
    pub user: Signer<'info>,

    /// If auto stake is not enabled, the ticket account that will recieve the bond tickets
    #[account(mut, constraint =
        mint(&user_ticket_vault.to_account_info()).unwrap()
        == orderbook_mut.bond_manager.load().unwrap().bond_ticket_mint.key() @ BondsError::WrongTicketMint
    )]
    pub user_ticket_vault: Account<'info, TokenAccount>,

    #[account(mut, constraint =
        mint(&user_token_vault.to_account_info()).unwrap()
        == orderbook_mut.bond_manager.load().unwrap().underlying_token_mint.key() @ BondsError::WrongUnderlyingTokenMint
    )]
    pub user_token_vault: Account<'info, TokenAccount>,

    /// SplitTicket that will be created if the order is filled as a taker and `auto_stake` is enabled
    /// CHECK: initialized in instruction
    #[account(mut)]
    pub split_ticket: AccountInfo<'info>,

    pub orderbook_mut: OrderbookMut<'info>,

    /// The market token vault
    #[account(mut, address = orderbook_mut.bond_manager.load().unwrap().underlying_token_vault.key() @ BondsError::WrongVault)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

    /// payer for `Obligation` initialization
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
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
            ctx.accounts.user_ticket_vault.key()
        },
        ctx.accounts.user_token_vault.key(),
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
        CallbackFlags::empty(),
    )?;

    if params.auto_stake {
        let mut split_ticket = serialization::init::<SplitTicket>(
            ctx.accounts.split_ticket.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            &SplitTicket::make_seeds(ctx.accounts.user.key().as_ref(), seed.as_slice()),
        )?;
        let timestamp = Clock::get()?.unix_timestamp;
        let manager = ctx.accounts.orderbook_mut.bond_manager.load()?;
        *split_ticket = SplitTicket {
            owner: ctx.accounts.user.key(),
            bond_manager: ctx.accounts.orderbook_mut.bond_manager.key(),
            order_tag: callback_info.order_tag,
            struck_timestamp: timestamp,
            maturation_timestamp: timestamp
                .safe_add(manager.duration)?
                .safe_add(manager.deposit_duration)?,
            principal: order_summary.total_quote_qty,
            interest: order_summary.total_base_qty - order_summary.total_quote_qty,
        }
    }
    // todo defensive rounding for posted_quote
    transfer(
        transfer_context!(ctx, underlying_token_vault, user_token_vault, user),
        order_summary.total_quote_qty,
    )?;
    emit!(crate::events::LendOrder {
        bond_market: ctx.accounts.orderbook_mut.bond_manager.key(),
        lender: ctx.accounts.user.key(),
        order_summary,
    });

    Ok(())
}
