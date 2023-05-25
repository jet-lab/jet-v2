use anchor_lang::prelude::*;
use anchor_spl::token::{accessor::mint, Mint, Token, TokenAccount};
use jet_margin::MarginAccount;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    margin::state::MarginUser, orderbook::state::*, serialization::RemainingAccounts,
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct MarginLendOrder<'info> {
    /// The account tracking borrower debts
    #[account(
        mut,
        constraint = margin_user.market == orderbook_mut.market.key() @ FixedTermErrorCode::UserNotInMarket,
        has_one = ticket_collateral @ FixedTermErrorCode::WrongTicketCollateralAccount,
        has_one = margin_account @ FixedTermErrorCode::WrongMarginUserAuthority,
    )]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub ticket_collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut, address = orderbook_mut.ticket_collateral_mint() @ FixedTermErrorCode::WrongTicketCollateralMint)]
    pub ticket_collateral_mint: AccountInfo<'info>,

    /// The margin account responsible for this order
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

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
    // Optional event adapter account
    // pub event_adapter: AccountInfo<'info>,
}

pub fn handler(ctx: Context<MarginLendOrder>, params: OrderParams) -> Result<()> {
    let a = ctx.accounts;
    let accounts = &mut MarginLendAccounts {
        margin_user: &mut a.margin_user,
        ticket_collateral: &a.ticket_collateral,
        ticket_collateral_mint: &a.ticket_collateral_mint,
        inner: &mut LendOrderAccounts {
            authority: &a.margin_account.to_account_info(),
            orderbook_mut: &mut a.orderbook_mut,
            ticket_settlement: &a.ticket_settlement,
            lender_tokens: a.lender_tokens.as_ref(),
            underlying_token_vault: &a.underlying_token_vault,
            ticket_mint: &a.ticket_mint,
            payer: &a.payer,
            system_program: &a.system_program,
            token_program: &a.token_program,
        },
    };
    accounts.margin_lend_order(
        &params,
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
        true,
    )
}
