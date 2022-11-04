use anchor_lang::{prelude::*, solana_program::clock::UnixTimestamp};
use bytemuck::Zeroable;
use jet_margin::{AdapterResult, MarginAccount};
use jet_program_common::traits::{SafeAdd, TryAddAssign, TrySubAssign};

use crate::{orderbook::state::OrderTag, BondsError};

pub const MARGIN_USER_VERSION: u8 = 0;

/// An acocunt used to track margin users of the market
#[account]
#[derive(Debug)]
pub struct MarginUser {
    /// used to determine if a migration step is needed before user actions are allowed
    pub version: u8,
    /// The margin account used for signing actions
    pub margin_account: Pubkey,
    /// The `BondManager` for the market
    pub bond_manager: Pubkey,
    /// Token account used by the margin program to track the debt
    pub claims: Pubkey,
    /// Token account used by the margin program to track the collateral value of positions
    /// which are internal to bonds, such as SplitTicket, ClaimTicket, and open orders.
    /// this does *not* represent underlying tokens or bond ticket tokens, those are registered independently in margin
    pub collateral: Pubkey,
    /// The `settle` instruction is permissionless, therefore the user must specify upon margin account creation
    /// the address to send owed tokens
    pub underlying_settlement: Pubkey,
    /// The `settle` instruction is permissionless, therefore the user must specify upon margin account creation
    /// the address to send owed tickets
    pub ticket_settlement: Pubkey,
    /// The amount of debt that must be collateralized or repaid
    /// This debt is expressed in terms of the underlying token - not bond tickets
    pub debt: Debt,
    /// Accounting used to track assets in custody of the bond market
    pub assets: Assets,
}

#[derive(Zeroable, Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Debt {
    /// The sequence number for the next obligation to be created
    next_new_obligation_seqno: u64,

    /// The sequence number of the next obligation to be paid
    next_unpaid_obligation_seqno: u64,

    /// The maturation timestamp of the next obligation that is unpaid
    next_obligation_maturity: UnixTimestamp,

    /// Amount that must be collateralized because there is an open order for it.
    /// Does not accrue interest because the loan has not been received yet.
    pending: u64,

    /// Debt that has already been borrowed because the order was matched.
    /// This debt will be due when the loan term ends.
    /// This includes all debt, including past due debt
    committed: u64,
}

pub type ObligationSequenceNumber = u64;

impl Debt {
    pub fn total(&self) -> u64 {
        self.pending.checked_add(self.committed).unwrap()
    }

    pub fn next_obligation_to_repay(&self) -> Option<ObligationSequenceNumber> {
        if self.next_new_obligation_seqno > self.next_unpaid_obligation_seqno {
            Some(self.next_unpaid_obligation_seqno)
        } else {
            None
        }
    }

    fn outstanding_obligations(&self) -> u64 {
        self.next_new_obligation_seqno - self.next_unpaid_obligation_seqno
    }

    pub fn post_borrow_order(&mut self, posted_amount: u64) -> Result<()> {
        self.pending.try_add_assign(posted_amount)
    }

    pub fn new_obligation_without_posting(
        &mut self,
        amount_filled_as_taker: u64,
        maturation_timestamp: UnixTimestamp,
    ) -> Result<ObligationSequenceNumber> {
        self.committed.try_add_assign(amount_filled_as_taker)?;
        if self.next_new_obligation_seqno == self.next_unpaid_obligation_seqno {
            self.next_obligation_maturity = maturation_timestamp;
        }
        let seqno = self.next_new_obligation_seqno;
        self.next_new_obligation_seqno.try_add_assign(1)?;

        Ok(seqno)
    }

    pub fn new_obligation_from_fill(
        &mut self,
        amount: u64,
        maturation_timestamp: UnixTimestamp,
    ) -> Result<ObligationSequenceNumber> {
        self.pending.try_sub_assign(amount)?;
        self.new_obligation_without_posting(amount, maturation_timestamp)
    }

    pub fn process_out(&mut self, amount: u64) -> Result<()> {
        self.pending.try_sub_assign(amount)
    }

    pub fn partially_repay_obligation(
        &mut self,
        sequence_number: ObligationSequenceNumber,
        amount_repaid: u64,
    ) -> Result<()> {
        if sequence_number != self.next_unpaid_obligation_seqno {
            todo!()
        }
        self.committed.try_sub_assign(amount_repaid)?;

        Ok(())
    }

    // The obligation is fully repaid by this repayment, and the obligation account is being closed
    pub fn fully_repay_obligation(
        &mut self,
        sequence_number: ObligationSequenceNumber,
        amount_repaid: u64,
        next_obligation: Result<Account<Obligation>>,
    ) -> Result<()> {
        if sequence_number != self.next_unpaid_obligation_seqno {
            todo!()
        }
        self.committed.try_sub_assign(amount_repaid)?;
        self.next_unpaid_obligation_seqno.try_add_assign(1)?;

        if self.next_unpaid_obligation_seqno < self.next_new_obligation_seqno {
            let next_obligation = next_obligation?;
            require_eq!(
                next_obligation.sequence_number,
                self.next_unpaid_obligation_seqno,
                BondsError::ObligationHasWrongSequenceNumber
            );
            self.next_obligation_maturity = next_obligation.maturation_timestamp;
        }

        Ok(())
    }

    pub fn is_past_due(&self) -> bool {
        self.outstanding_obligations() > 0
            && self.next_obligation_maturity <= Clock::get().unwrap().unix_timestamp
    }

    pub fn pending(&self) -> u64 {
        self.pending
    }
}

#[derive(Zeroable, Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub struct Assets {
    /// tokens to transfer into settlement account
    pub entitled_tokens: u64,
    /// tickets to transfer into settlement account
    pub entitled_tickets: u64,

    /// The number of bond tickets locked up in ClaimTicket or SplitTicket
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
    pub fn stake_tickets(&mut self, tickets: u64) -> Result<()> {
        self.tickets_staked.try_add_assign(tickets)
    }

    pub fn redeem_staked_tickets(&mut self, tickets: u64) {
        if tickets >= self.tickets_staked {
            self.tickets_staked = 0;
        } else {
            self.tickets_staked -= tickets;
        }
    }

    /// represents the amount of collateral in staked tickets and open orders.
    /// does not reflect the entitled tickets/tokens because they are expected
    /// to be disbursed whenever this value is used.
    pub fn collateral(&self) -> Result<u64> {
        self.tickets_staked.safe_add(self.posted_quote)
    }
}

#[account]
#[derive(Debug)]
pub struct Obligation {
    pub sequence_number: ObligationSequenceNumber,

    /// The user borrower account this obligation is assigned to
    pub borrower_account: Pubkey,

    /// The bond manager where the obligation was created
    pub bond_manager: Pubkey,

    /// The `OrderTag` associated with the creation of this `Obligation`
    pub order_tag: OrderTag,

    /// The time that the obligation must be repaid
    pub maturation_timestamp: UnixTimestamp,

    /// The remaining amount due by the end of the loan term
    pub balance: u64,

    /// Any boolean flags for this data type compressed to a single byte
    pub flags: ObligationFlags,
}

impl Obligation {
    pub fn make_seeds<'a>(user: &'a [u8], bytes: &'a [u8]) -> [&'a [u8]; 3] {
        [crate::seeds::OBLIGATION, user, bytes]
    }
}

bitflags! {
    #[derive(Default, AnchorSerialize, AnchorDeserialize)]
    pub struct ObligationFlags: u8 {
        /// This obligation has already been marked as due.
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
