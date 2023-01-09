use std::cmp::min;

use anchor_lang::{prelude::*, AccountsClose};
use anchor_spl::token::{transfer, Token, Transfer};
use jet_program_common::traits::TrySubAssign;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    control::state::Market,
    events::{TermLoanFulfilled, TermLoanRepay},
    margin::state::{MarginUser, TermLoan},
    market_token_manager::MarketTokenManager,
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
        constraint = term_loan.sequence_number
            == margin_user.debt.next_term_loan_to_repay().unwrap()
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
    pub payer: Signer<'info>,

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

impl<'info> Repay<'info> {
    pub fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.source.to_account_info(),
                to: self.underlying_token_vault.to_account_info(),
                authority: self.payer.to_account_info(),
            },
        )
    }
}

pub fn handler(ctx: Context<Repay>, amount: u64) -> Result<()> {
    let amount = min(amount, ctx.accounts.term_loan.balance);
    transfer(ctx.accounts.transfer_context(), amount)?;
    ctx.burn_notes(&ctx.accounts.claims_mint, &ctx.accounts.claims, amount)?;

    let term_loan = &mut ctx.accounts.term_loan;
    let user = &mut ctx.accounts.margin_user;

    term_loan.balance.try_sub_assign(amount)?;

    if term_loan.balance > 0 {
        user.debt
            .partially_repay_term_loan(term_loan.sequence_number, amount)?;
        emit!(TermLoanRepay {
            orderbook_user: ctx.accounts.margin_user.key(),
            term_loan: term_loan.key(),
            repayment_amount: amount,
            final_balance: term_loan.balance,
        });
    } else {
        term_loan.close(ctx.accounts.payer.to_account_info())?;

        let user_key = user.key();
        let next_term_loan =
            Account::<TermLoan>::try_from(&ctx.accounts.next_term_loan).and_then(|ob| {
                require_eq!(
                    ob.margin_user,
                    user_key,
                    FixedTermErrorCode::UserNotInMarket
                );
                Ok(ob)
            });
        user.debt
            .fully_repay_term_loan(term_loan.sequence_number, amount, next_term_loan)?;

        emit!(TermLoanFulfilled {
            term_loan: term_loan.key(),
            orderbook_user: user.key(),
            borrower: term_loan.margin_user,
            repayment_amount: amount,
            timestamp: Clock::get()?.unix_timestamp,
        });
    }

    Ok(())
}
