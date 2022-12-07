use std::cmp::min;

use anchor_lang::{prelude::*, AccountsClose};
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};
use jet_program_common::traits::TrySubAssign;

use crate::{
    events::{TermLoanFulfilled, TermLoanRepay},
    margin::state::{MarginUser, TermLoan},
    ErrorCode,
};

#[derive(Accounts)]
pub struct Repay<'info> {
    /// The account tracking information related to this particular user
    #[account(mut)]
    pub borrower_account: Account<'info, MarginUser>,

    #[account(
        mut,
        has_one = borrower_account @ ErrorCode::UserNotInMarket,
        constraint = term_loan.sequence_number
            == borrower_account.debt.next_term_loan_to_repay().unwrap()
            @ ErrorCode::TermLoanHasWrongSequenceNumber
    )]
    pub term_loan: Account<'info, TermLoan>,

    /// No payment will be made towards next_term_loan: it is needed purely for bookkeeping.
    /// if the user has additional term_loan, this must be the one with the following sequence number.
    /// otherwise, put whatever address you want in here
    pub next_term_loan: AccountInfo<'info>,

    /// The token account to deposit tokens from
    #[account(mut)]
    pub source: Account<'info, TokenAccount>,

    /// The signing authority for the source_account
    pub payer: Signer<'info>,

    /// The token vault holding the underlying token of the ticket
    #[account(mut)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

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

    let term_loan = &mut ctx.accounts.term_loan;
    let user = &mut ctx.accounts.borrower_account;

    term_loan.balance.try_sub_assign(amount)?;

    if term_loan.balance > 0 {
        user.debt
            .partially_repay_term_loan(term_loan.sequence_number, amount)?;
    } else {
        emit!(TermLoanFulfilled {
            term_loan: term_loan.key(),
            orderbook_user: user.key(),
            borrower: term_loan.borrower_account,
            timestamp: Clock::get()?.unix_timestamp,
        });

        term_loan.close(ctx.accounts.payer.to_account_info())?;

        let user_key = user.key();
        let next_term_loan =
            Account::<TermLoan>::try_from(&ctx.accounts.next_term_loan).and_then(|ob| {
                require_eq!(ob.borrower_account, user_key, ErrorCode::UserNotInMarket);
                Ok(ob)
            });
        user.debt
            .fully_repay_term_loan(term_loan.sequence_number, amount, next_term_loan)?;
    }

    emit!(TermLoanRepay {
        orderbook_user: ctx.accounts.borrower_account.key(),
        term_loan: term_loan.key(),
        repayment_amount: amount,
        final_balance: term_loan.balance,
    });

    Ok(())
}
