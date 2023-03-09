use std::ops::Range;

use anchor_lang::{prelude::*, solana_program::clock::UnixTimestamp};
use bytemuck::Zeroable;
use jet_margin::{AdapterResult, MarginAccount};
use jet_program_common::traits::{SafeAdd, TryAddAssign, TrySubAssign};

use crate::{
    events::{AssetsUpdated, DebtUpdated},
    orderbook::state::OrderTag,
    FixedTermErrorCode,
};

pub const MARGIN_USER_VERSION: u8 = 0;

/// An acocunt used to track margin users of the market
#[account]
#[repr(C, align(8))]
#[derive(Debug)]
pub struct MarginUser {
    /// used to determine if a migration step is needed before user actions are allowed
    pub version: u8,
    /// The margin account used for signing actions
    pub margin_account: Pubkey,
    /// The `Market` for the market
    pub market: Pubkey,
    /// Token account used by the margin program to track the debt
    pub claims: Pubkey,
    /// Token account used by the margin program to track the collateral value of positions
    /// which are internal to fixed-term market, such as SplitTicket, ClaimTicket, and open orders.
    /// this does *not* represent underlying tokens or ticket tokens, those are registered independently in margin
    pub ticket_collateral: Pubkey,
    /// The amount of debt that must be collateralized or repaid
    /// This debt is expressed in terms of the underlying token - not tickets
    pub debt: Debt,
    /// Accounting used to track assets in custody of the fixed term market
    pub assets: Assets,
    /// Settings for borrow order "auto rolling"
    pub borrow_roll_config: AutoRollConfig,
    /// Settings for lend order "auto rolling"
    pub lend_roll_config: AutoRollConfig,
}

impl MarginUser {
    /// Emits an Anchor event with the latest balances for [Assets] and [Debt].
    /// Callers should take care to invoke this function after mutating both assets
    /// and debts. When only mutating either, they can emit the individual balances.
    pub fn emit_all_balances(&self) {
        let margin_user = self.derive_address();
        emit!(AssetsUpdated::new(margin_user, &self.assets));
        emit!(DebtUpdated::new(margin_user, &self.debt));
    }

    /// Emits an Anchor event with the latest balances for [Assets].
    pub fn emit_asset_balances(&self) {
        let margin_user = self.derive_address();
        emit!(AssetsUpdated::new(margin_user, &self.assets));
    }

    /// Emits an Anchor event with the latest balances for [Debt].
    pub fn emit_debt_balances(&self) {
        let margin_user = self.derive_address();
        emit!(DebtUpdated::new(margin_user, &self.debt));
    }

    #[inline]
    fn derive_address(&self) -> Pubkey {
        Pubkey::find_program_address(
            &[
                crate::seeds::MARGIN_USER,
                self.market.as_ref(),
                self.margin_account.as_ref(),
            ],
            &crate::id(),
        )
        .0
    }
}

#[derive(Zeroable, Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Debt {
    /// The sequence number for the next term loan to be created
    next_new_term_loan_seqno: u64,

    /// The sequence number of the next term loan to be paid
    next_unpaid_term_loan_seqno: u64,

    /// The maturation timestamp of the next term loan that is unpaid
    next_term_loan_maturity: UnixTimestamp,

    /// Amount that must be collateralized because there is an open order for it.
    /// Does not accrue interest because the loan has not been received yet.
    pending: u64,

    /// Debt that has already been borrowed because the order was matched.
    /// This debt will be due when the loan term ends.
    /// This includes all debt, including past due debt
    committed: u64,
}

pub type SequenceNumber = u64;

impl Debt {
    pub fn total(&self) -> u64 {
        self.pending.checked_add(self.committed).unwrap()
    }

    pub fn active_loans(&self) -> Range<SequenceNumber> {
        self.next_unpaid_term_loan_seqno..self.next_new_term_loan_seqno
    }

    pub fn next_new_loan_seqno(&self) -> SequenceNumber {
        self.next_new_term_loan_seqno
    }

    pub fn next_term_loan_to_repay(&self) -> Option<SequenceNumber> {
        if self.next_new_term_loan_seqno > self.next_unpaid_term_loan_seqno {
            Some(self.next_unpaid_term_loan_seqno)
        } else {
            None
        }
    }

    pub fn outstanding_term_loans(&self) -> u64 {
        self.next_new_term_loan_seqno - self.next_unpaid_term_loan_seqno
    }

    pub fn post_borrow_order(&mut self, posted_amount: u64) -> Result<()> {
        self.pending.try_add_assign(posted_amount)
    }

    pub fn new_term_loan_without_posting(
        &mut self,
        amount_filled_as_taker: u64,
        maturation_timestamp: UnixTimestamp,
    ) -> Result<SequenceNumber> {
        self.committed.try_add_assign(amount_filled_as_taker)?;
        if self.next_new_term_loan_seqno == self.next_unpaid_term_loan_seqno {
            self.next_term_loan_maturity = maturation_timestamp;
        }
        let seqno = self.next_new_term_loan_seqno;
        self.next_new_term_loan_seqno.try_add_assign(1)?;

        Ok(seqno)
    }

    pub fn new_term_loan_from_fill(
        &mut self,
        amount: u64,
        maturation_timestamp: UnixTimestamp,
    ) -> Result<SequenceNumber> {
        self.pending.try_sub_assign(amount)?;
        self.new_term_loan_without_posting(amount, maturation_timestamp)
    }

    pub fn process_out(&mut self, amount: u64) -> Result<()> {
        self.pending.try_sub_assign(amount)
    }

    pub fn partially_repay_term_loan(
        &mut self,
        sequence_number: SequenceNumber,
        amount_repaid: u64,
    ) -> Result<()> {
        if sequence_number != self.next_unpaid_term_loan_seqno {
            todo!()
        }
        self.committed.try_sub_assign(amount_repaid)?;

        Ok(())
    }

    // The term loan is fully repaid by this repayment, and the term loan account is being closed
    pub fn fully_repay_term_loan(
        &mut self,
        sequence_number: SequenceNumber,
        amount_repaid: u64,
        next_term_loan: Result<Account<TermLoan>>,
    ) -> Result<()> {
        if sequence_number != self.next_unpaid_term_loan_seqno {
            todo!()
        }
        self.committed.try_sub_assign(amount_repaid)?;
        self.next_unpaid_term_loan_seqno.try_add_assign(1)?;

        if self.next_unpaid_term_loan_seqno < self.next_new_term_loan_seqno {
            let next_term_loan = next_term_loan?;
            require_eq!(
                next_term_loan.sequence_number,
                self.next_unpaid_term_loan_seqno,
                FixedTermErrorCode::TermLoanHasWrongSequenceNumber
            );
            self.next_term_loan_maturity = next_term_loan.maturation_timestamp;
        }

        Ok(())
    }

    pub fn is_past_due(&self) -> bool {
        self.outstanding_term_loans() > 0
            && self.next_term_loan_maturity <= Clock::get().unwrap().unix_timestamp
    }

    pub fn pending(&self) -> u64 {
        self.pending
    }

    pub fn committed(&self) -> u64 {
        self.committed
    }
}

#[derive(Zeroable, Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub struct Assets {
    /// tokens to transfer into settlement account
    pub entitled_tokens: u64,
    /// tickets to transfer into settlement account
    pub entitled_tickets: u64,

    /// The sequence number for the next deposit
    next_deposit_seqno: u64,

    /// The sequence number for the oldest deposit that has yet to be redeemed
    next_unredeemed_deposit_seqno: u64,

    /// The number of tickets locked up in ClaimTicket or SplitTicket
    tickets_staked: u64,

    /// The amount of quote included in all orders posted by the user for both
    /// bids and asks. Since the orderbook tracks base, not quote, this is only
    /// an approximation. This value must always be less than or equal to the
    /// actual posted quote.
    posted_quote: u64,

    /// reserved data that may be used to determine the size of a user's collateral
    /// pessimistically prepared to persist aggregated values for:
    /// base and quote quantities, separately for bid/ask, on open orders and unsettled fills
    /// 2^3 = 8 u64's
    _reserved0: [u8; 64],
}

impl Assets {
    /// either a bid or ask was placed
    /// quote: the amount of quote that was posted
    /// IMPORTANT: always input the quote (underlying), not the base
    /// always shorts by one lamport to be defensive
    /// todo maybe this is too defensive
    pub fn post_order(&mut self, quote: u64) -> Result<()> {
        if quote > 1 {
            return self.posted_quote.try_add_assign(quote - 1);
        }
        Ok(())
    }

    /// An order was filled or cancelled
    /// quote: the amount of quote that was removed from the order
    /// IMPORTANT: always input the quote (underlying), not the base
    /// always subtracts an extra lamport to be defensive
    /// todo maybe this is too defensive
    pub fn reduce_order(&mut self, quote: u64) {
        if quote + 1 >= self.posted_quote {
            self.posted_quote = 0;
        } else {
            self.posted_quote -= quote + 1;
        }
    }

    /// make sure the order has already been accounted for before calling this method
    pub fn new_deposit(&mut self, tickets: u64) -> Result<SequenceNumber> {
        let seqno = self.next_deposit_seqno;

        if tickets > 0 {
            self.next_deposit_seqno += 1;
            self.tickets_staked.try_add_assign(tickets)?;
        }

        Ok(seqno)
    }

    pub fn redeem_deposit(&mut self, seqno: SequenceNumber, tickets: u64) -> Result<()> {
        if seqno != self.next_unredeemed_deposit_seqno {
            msg!(
                "Given sequence number: [{}] Expected sequence number: [{}]",
                seqno,
                self.next_unredeemed_deposit_seqno
            );
            return Err(FixedTermErrorCode::TermDepositHasWrongSequenceNumber.into());
        }

        self.next_unredeemed_deposit_seqno += 1;
        self.tickets_staked = self.tickets_staked.saturating_sub(tickets);

        Ok(())
    }

    /// represents the amount of collateral in staked tickets and open orders.
    /// does not reflect the entitled tickets/tokens because they are expected
    /// to be disbursed whenever this value is used.
    pub fn collateral(&self) -> Result<u64> {
        self.tickets_staked.safe_add(self.posted_quote)
    }

    pub fn next_new_deposit_seqno(&self) -> SequenceNumber {
        self.next_deposit_seqno
    }

    pub fn active_deposits(&self) -> Range<SequenceNumber> {
        self.next_unredeemed_deposit_seqno..self.next_deposit_seqno
    }
}

#[derive(Zeroable, Default, Debug, Clone, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub struct AutoRollConfig {
    /// the limit price at which orders may be placed by an authority
    pub limit_price: u64,
}

#[account]
#[derive(Debug)]
pub struct TermLoan {
    pub sequence_number: SequenceNumber,

    /// The user borrower account this term loan is assigned to
    pub margin_user: Pubkey,

    /// The market where the term loan was created
    pub market: Pubkey,

    /// Which account recieves the rent when this PDA is destructed
    pub payer: Pubkey,

    /// The `OrderTag` associated with the creation of this `TermLoan`
    pub order_tag: OrderTag,

    /// The time that the term loan must be repaid
    pub maturation_timestamp: UnixTimestamp,

    /// The remaining amount due by the end of the loan term
    pub balance: u64,

    /// Any boolean flags for this data type compressed to a single byte
    pub flags: TermLoanFlags,
}

bitflags! {
    #[derive(Default, AnchorSerialize, AnchorDeserialize)]
    pub struct TermLoanFlags: u8 {
        /// This term loan has already been marked as due.
        const MARKED_DUE = 0b00000001;
    }
}

#[cfg(not(feature = "mock-margin"))]
pub fn return_to_margin(user: &AccountInfo, adapter_result: &AdapterResult) -> Result<()> {
    let loader = AccountLoader::<MarginAccount>::try_from(user)?;
    let margin_account = loader.load()?;
    jet_margin::write_adapter_result(&margin_account, adapter_result)
}

#[cfg(feature = "mock-margin")]
pub fn return_to_margin(_user: &AccountInfo, _adapter_result: &AdapterResult) -> Result<()> {
    Ok(())
}
