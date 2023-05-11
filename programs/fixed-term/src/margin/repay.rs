use anchor_lang::prelude::*;
use anchor_spl::token::{burn, transfer, Burn, Transfer};
use jet_program_common::{debug_msg, traits::TrySubAssign};

use crate::{
    control::state::Market,
    events::{TermLoanFulfilled, TermLoanRepay},
    FixedTermErrorCode,
};

use super::state::{MarginUser, TermLoan};

pub struct RepayAccounts<'a, 'info> {
    pub margin_user: &'a mut Account<'info, MarginUser>,
    pub term_loan: &'a mut Account<'info, TermLoan>,
    pub next_term_loan: &'a AccountInfo<'info>,
    pub source: &'a AccountInfo<'info>,
    pub source_authority: &'a AccountInfo<'info>,
    pub payer: &'a AccountInfo<'info>,
    pub underlying_token_vault: &'a AccountInfo<'info>,
    pub claims: &'a AccountInfo<'info>,
    pub claims_mint: &'a AccountInfo<'info>,
    pub market: &'a AccountLoader<'info, Market>,
    pub token_program: &'a AccountInfo<'info>,
}

impl<'a, 'info> RepayAccounts<'a, 'info> {
    /// The `skip_token_transfer` flag determines whether funds must be deposited in the vault
    /// If set to `true` then a transfer from the caller to the market vault is unnecessary
    /// Use caution to prevent leaking funds
    /// The flag is also used as a proxy to determine whether the repayment is a result of an
    /// auto-roll.
    pub fn repay(&mut self, amount: u64, skip_token_transfer: bool) -> Result<()> {
        let amount = std::cmp::min(amount, self.term_loan.balance);
        debug_msg!(
            "Repaying {} out of {} towards term loan {}",
            amount,
            self.term_loan.balance,
            self.term_loan.key()
        );

        // return payment to market vault
        if !skip_token_transfer {
            transfer(self.transfer_context(), amount)?;
        }

        // reduce claim on the margin account
        self.burn_claim_notes(amount)?;

        // repay on the loan
        self.term_loan.balance.try_sub_assign(amount)?;

        if self.term_loan.balance > 0 {
            self.margin_user
                .partially_repay_loan(self.term_loan, amount)?;
            emit!(TermLoanRepay {
                orderbook_user: self.margin_user.key(),
                term_loan: self.term_loan.key(),
                repayment_amount: amount,
                final_balance: self.term_loan.balance,
                is_auto_roll: skip_token_transfer,
            });
        } else {
            let next_term_loan =
                Account::<TermLoan>::try_from(self.next_term_loan).and_then(|ob| {
                    require_eq!(
                        ob.margin_user,
                        self.margin_user.key(),
                        FixedTermErrorCode::UserNotInMarket
                    );
                    Ok(ob)
                });
            self.margin_user
                .fully_repay_term_loan(self.term_loan, amount, next_term_loan)?;

            self.term_loan.close(self.payer.to_account_info())?;

            emit!(TermLoanFulfilled {
                term_loan: self.term_loan.key(),
                orderbook_user: self.margin_user.key(),
                borrower: self.term_loan.margin_user,
                repayment_amount: amount,
                timestamp: Clock::get()?.unix_timestamp,
                is_auto_roll: skip_token_transfer,
            });
        }

        Ok(())
    }

    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.source.to_account_info(),
                to: self.underlying_token_vault.to_account_info(),
                authority: self.source_authority.to_account_info(),
            },
        )
    }

    fn burn_claim_notes(&self, amount: u64) -> Result<()> {
        burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                Burn {
                    mint: self.claims_mint.to_account_info(),
                    from: self.claims.to_account_info(),
                    authority: self.market.to_account_info(),
                },
            )
            .with_signer(&[&self.market.load()?.authority_seeds()]),
            amount,
        )
    }
}
