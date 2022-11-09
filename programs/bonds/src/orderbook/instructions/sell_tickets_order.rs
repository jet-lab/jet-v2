use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::{accessor::mint, Mint, Token, TokenAccount};
use jet_program_proc_macros::BondTokenManager;

use crate::{
    bond_token_manager::BondTokenManager, events::OrderType, orderbook::state::*,
    serialization::RemainingAccounts, BondsError,
};

#[derive(Accounts, BondTokenManager)]
pub struct SellTicketsOrder<'info> {
    /// Signing authority over the ticket vault transferring for a borrow order
    pub authority: Signer<'info>,

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

    #[bond_manager]
    pub orderbook_mut: OrderbookMut<'info>,

    /// The market ticket mint
    #[account(mut, address = orderbook_mut.bond_manager.load().unwrap().bond_ticket_mint.key() @ BondsError::WrongTicketMint)]
    pub bond_ticket_mint: Account<'info, Mint>,

    /// The market ticket mint
    #[account(mut, address = orderbook_mut.bond_manager.load().unwrap().underlying_token_vault.key() @ BondsError::WrongTicketMint)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    // Optional event adapter account
    // pub event_adapter: AccountInfo<'info>,
}

impl<'info> SellTicketsOrder<'info> {
    pub fn sell_tickets(
        &self,
        order_summary: SensibleOrderSummary,
        params: &OrderParams,
        margin_user: Option<Pubkey>,
        order_type: OrderType,
    ) -> Result<()> {
        self.withdraw(
            &self.underlying_token_vault,
            &self.user_token_vault,
            order_summary.quote_filled()?,
        )?;
        anchor_spl::token::burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                anchor_spl::token::Burn {
                    mint: self.bond_ticket_mint.to_account_info(),
                    from: self.user_ticket_vault.to_account_info(),
                    authority: self.authority.to_account_info(),
                },
            ),
            order_summary.base_combined(),
        )?;
        emit!(crate::events::OrderPlaced {
            bond_manager: self.orderbook_mut.bond_manager.key(),
            authority: self.authority.key(),
            order_summary: order_summary.summary(),
            margin_user,
            order_type,
            limit_price: params.limit_price,
            auto_stake: params.auto_stake,
            post_only: params.post_only,
            post_allowed: params.post_allowed,
        });

        Ok(())
    }
}

pub fn handler(ctx: Context<SellTicketsOrder>, params: OrderParams) -> Result<()> {
    let (_, order_summary) = ctx.accounts.orderbook_mut.place_order(
        ctx.accounts.authority.key(),
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

    ctx.accounts
        .sell_tickets(order_summary, &params, None, OrderType::SellTickets)
}
