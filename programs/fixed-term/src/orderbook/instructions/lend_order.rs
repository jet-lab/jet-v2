use anchor_lang::prelude::*;
use anchor_spl::token::{accessor::mint, Mint, Token, TokenAccount};
use jet_airspace::state::AirspacePermit;
use jet_program_proc_macros::MarketTokenManager;

use crate::{orderbook::state::*, serialization::RemainingAccounts, FixedTermErrorCode};

#[derive(Accounts, MarketTokenManager)]
pub struct LendOrder<'info> {
    /// Metadata permit allowing this user to interact with this market
    #[account(
        constraint = permit.owner == authority.key() @ FixedTermErrorCode::WrongAirspaceAuthorization,
        constraint = permit.airspace == orderbook_mut.airspace() @ FixedTermErrorCode::WrongAirspaceAuthorization,
    )]
    pub permit: Account<'info, AirspacePermit>,

    /// Authority accounted for as the owner of resulting orderbook bids and `TermDeposit` accounts
    pub authority: Signer<'info>,

    #[market]
    pub orderbook_mut: OrderbookMut<'info>,

    /// where to settle tickets on match:
    /// - TermDeposit that will be created if the order is filled as a taker and `auto_stake` is enabled
    /// - ticket token account to receive tickets
    /// be careful to check this properly. one way is by using lender_tickets_token_account
    #[account(mut)]
    pub(crate) ticket_settlement: AccountInfo<'info>,

    /// where to loan tokens from
    #[account(mut, constraint = mint(&lender_tokens.to_account_info())? == orderbook_mut.underlying_mint() @ FixedTermErrorCode::WrongUnderlyingTokenMint)]
    pub lender_tokens: Account<'info, TokenAccount>,

    /// The market token vault
    #[account(mut, address = orderbook_mut.vault() @ FixedTermErrorCode::WrongVault)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

    /// The market token vault
    #[account(mut, address = orderbook_mut.ticket_mint() @ FixedTermErrorCode::WrongTicketMint)]
    pub ticket_mint: Account<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<LendOrder>, params: OrderParams, seed: Vec<u8>) -> Result<()> {
    let accs = ctx.accounts;
    let accounts = &mut LendOrderAccounts {
        authority: &accs.authority,
        orderbook_mut: &mut accs.orderbook_mut,
        ticket_settlement: &accs.ticket_settlement,
        lender_tokens: accs.lender_tokens.as_ref(),
        underlying_token_vault: &accs.underlying_token_vault,
        ticket_mint: &accs.ticket_mint,
        payer: &accs.payer,
        system_program: &accs.system_program,
        token_program: &accs.token_program,
    };
    accounts.lend_order(
        params,
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
        seed,
    )
}
