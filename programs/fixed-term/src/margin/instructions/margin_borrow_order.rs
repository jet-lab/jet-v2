use anchor_lang::prelude::*;
use anchor_spl::{associated_token::get_associated_token_address, token::Token};
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    margin::state::MarginUser, orderbook::state::*, serialization::RemainingAccounts,
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct MarginBorrowOrder<'info> {
    /// The account tracking borrower debts
    #[account(
        mut,
        has_one = margin_account,
        has_one = claims @ FixedTermErrorCode::WrongClaimAccount,
        has_one = underlying_collateral @ FixedTermErrorCode::WrongTicketCollateralAccount,
        constraint = margin_user.market == orderbook_mut.market.key() @ FixedTermErrorCode::UserNotInMarket
    )]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// TermLoan account minted upon match
    /// CHECK: in instruction logic
    #[account(mut)]
    pub term_loan: AccountInfo<'info>,

    /// The margin account for this borrow order
    pub margin_account: Signer<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    /// CHECK: margin_user
    #[account(mut)]
    pub claims: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    /// CHECK: in instruction handler
    #[account(mut, address = orderbook_mut.claims_mint() @ FixedTermErrorCode::WrongClaimMint)]
    pub claims_mint: AccountInfo<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub underlying_collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut, address = orderbook_mut.underlying_collateral_mint() @ FixedTermErrorCode::WrongUnderlyingCollateralMint)]
    pub underlying_collateral_mint: AccountInfo<'info>,

    /// The market token vault
    #[account(mut, address = orderbook_mut.vault() @ FixedTermErrorCode::WrongVault)]
    pub underlying_token_vault: AccountInfo<'info>,

    /// The market fee vault
    #[account(mut, address = orderbook_mut.fee_vault() @ FixedTermErrorCode::WrongVault)]
    pub fee_vault: AccountInfo<'info>,

    /// Where to receive borrowed tokens
    #[account(mut, address = get_associated_token_address(
        &margin_user.margin_account,
        &orderbook_mut.underlying_mint(),
    ))]
    pub underlying_settlement: AccountInfo<'info>,

    #[market]
    pub orderbook_mut: OrderbookMut<'info>,

    /// payer for `TermLoan` initialization
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana system program
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
    // Optional event adapter account
    // pub event_adapter: AccountInfo<'info>,
}

pub fn handler(ctx: Context<MarginBorrowOrder>, params: OrderParams) -> Result<()> {
    let a = ctx.accounts;
    MarginBorrowOrderAccounts {
        margin_user: &mut a.margin_user,
        term_loan: &a.term_loan,
        margin_account: &a.margin_account,
        claims: &a.claims,
        claims_mint: &a.claims_mint,
        underlying_collateral: &a.underlying_collateral,
        underlying_collateral_mint: &a.underlying_collateral_mint,
        underlying_token_vault: &a.underlying_token_vault,
        fee_vault: &a.fee_vault,
        underlying_settlement: &a.underlying_settlement,
        orderbook_mut: &mut a.orderbook_mut,
        payer: &a.payer,
        system_program: &a.system_program.to_account_info(),
        token_program: &a.token_program.to_account_info(),
        event_adapter: ctx
            .remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
    }
    .borrow_order(params)?;
    Ok(())
}
