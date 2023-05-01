use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    control::state::Market,
    margin::state::{MarginUser, RepayAccounts, TermLoan},
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct Repay<'info> {
    /// The account tracking information related to this particular user
    #[account(mut, has_one = claims @ FixedTermErrorCode::WrongClaimAccount)]
    pub margin_user: Account<'info, MarginUser>,

    #[account(
        mut,
        has_one = margin_user @ FixedTermErrorCode::UserNotInMarket,
        has_one = payer,
        constraint = term_loan.sequence_number
            == margin_user.debt().next_term_loan_to_repay().unwrap()
            @ FixedTermErrorCode::TermLoanHasWrongSequenceNumber
    )]
    pub term_loan: Account<'info, TermLoan>,

    /// No payment will be made towards next_term_loan: it is needed purely for bookkeeping.
    /// if the user has additional term_loan, this must be the one with the following sequence number.
    /// otherwise, put whatever address you want in here
    pub next_term_loan: AccountInfo<'info>,

    /// The token account to deposit tokens from
    #[account(mut)]
    pub source: AccountInfo<'info>,

    /// The signing authority for the source_account
    pub source_authority: Signer<'info>,

    /// The payer for the `TermLoan` to return rent to
    #[account(mut)]
    pub payer: AccountInfo<'info>,

    /// The token vault holding the underlying token of the ticket
    #[account(mut)]
    pub underlying_token_vault: AccountInfo<'info>,

    /// The token account representing claims for this margin user
    #[account(mut)]
    pub claims: AccountInfo<'info>,

    /// The token account representing claims for this margin user
    #[account(mut)]
    pub claims_mint: AccountInfo<'info>,

    #[account(
        has_one = claims_mint @ FixedTermErrorCode::WrongClaimMint,
        has_one = underlying_token_vault @ FixedTermErrorCode::WrongVault,
    )]
    pub market: AccountLoader<'info, Market>,

    /// SPL token program
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Repay>, amount: u64) -> Result<()> {
    let a = ctx.accounts;
    RepayAccounts {
        margin_user: &mut a.margin_user,
        term_loan: &mut a.term_loan,
        next_term_loan: &a.next_term_loan,
        source: &a.source,
        source_authority: &a.source_authority,
        payer: &a.payer,
        underlying_token_vault: &a.underlying_token_vault,
        claims: &a.claims,
        claims_mint: &a.claims_mint,
        market: &a.market,
        token_program: &a.token_program,
    }
    .repay(amount, false)
}
