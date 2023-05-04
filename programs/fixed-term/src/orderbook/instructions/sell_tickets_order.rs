use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::{accessor::mint, Mint, Token, TokenAccount};
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    events::OrderType, market_token_manager::MarketTokenManager, orderbook::state::*,
    serialization::RemainingAccounts, FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct SellTicketsOrder<'info> {
    /// Signing authority over the ticket vault transferring for a borrow order
    pub authority: Signer<'info>,

    /// Account containing the tickets being sold
    #[account(mut, constraint =
        mint(&user_ticket_vault.to_account_info()).unwrap()
        == ticket_mint.key() @ FixedTermErrorCode::WrongTicketMint
    )]
    pub user_ticket_vault: Account<'info, TokenAccount>,

    /// The account to receive the matched tokens
    #[account(mut, constraint =
        mint(&user_token_vault.to_account_info()).unwrap()
        == orderbook_mut.market.load().unwrap().underlying_token_mint.key() @ FixedTermErrorCode::WrongUnderlyingTokenMint
    )]
    pub user_token_vault: Account<'info, TokenAccount>,

    #[market]
    pub orderbook_mut: OrderbookMut<'info>,

    /// The ticket mint
    #[account(mut, address = orderbook_mut.market.load().unwrap().ticket_mint.key() @ FixedTermErrorCode::WrongTicketMint)]
    pub ticket_mint: Account<'info, Mint>,

    /// The token vault holding the underlying token of the ticket
    #[account(mut, address = orderbook_mut.market.load().unwrap().underlying_token_vault.key() @ FixedTermErrorCode::WrongTicketMint)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    // Optional event adapter account
    // pub event_adapter: AccountInfo<'info>,
}

impl<'info> SellTicketsOrder<'info> {
    pub fn sell_tickets(
        &self,
        order_tag: u128,
        order_summary: SensibleOrderSummary,
        params: &OrderParams,
        margin_user: Option<Pubkey>,
        order_type: OrderType,
    ) -> Result<()> {
        self.withdraw(
            &self.underlying_token_vault,
            &self.user_token_vault,
            order_summary.quote_filled(RoundingAction::FillBorrow.direction())?,
        )?;
        anchor_spl::token::burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                anchor_spl::token::Burn {
                    mint: self.ticket_mint.to_account_info(),
                    from: self.user_ticket_vault.to_account_info(),
                    authority: self.authority.to_account_info(),
                },
            ),
            order_summary.base_combined(),
        )?;
        emit!(crate::events::OrderPlaced {
            market: self.orderbook_mut.market.key(),
            authority: self.authority.key(),
            order_tag,
            order_summary: order_summary.summary(),
            margin_user,
            order_type,
            limit_price: params.limit_price,
            auto_stake: params.auto_stake,
            post_only: params.post_only,
            post_allowed: params.post_allowed,
            auto_roll: params.auto_roll
        });

        Ok(())
    }
}

pub fn handler(ctx: Context<SellTicketsOrder>, params: OrderParams) -> Result<()> {
    let (info, order_summary) = ctx.accounts.orderbook_mut.place_signer_order(
        Side::Ask,
        params,
        ctx.accounts.authority.key(),
        ctx.accounts.user_token_vault.key(),
        ctx.accounts.user_ticket_vault.key(),
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
        CallbackFlags::empty(),
    )?;

    ctx.accounts.sell_tickets(
        info.order_tag.as_u128(),
        order_summary,
        &params,
        None,
        OrderType::SellTickets,
    )
}
