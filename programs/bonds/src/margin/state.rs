use anchor_lang::{prelude::*, solana_program::clock::UnixTimestamp};
use bytemuck::Zeroable;
use jet_margin::{AdapterResult, MarginAccount};
use jet_proto_math::traits::{TryAddAssign, TrySubAssign};

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

    pub underlying_settlement: Pubkey,
    pub ticket_settlement: Pubkey,
    /// The amount of debt that must be collateralized or repaid
    /// This debt is expressed in terms of the underlying token - not bond tickets
    pub debt: Debt,
    /// Accounting used to track assets in custody of the bond market
    pub assets: Assets,
}

#[cfg(feature = "cli")]
impl Serialize for MarginUser {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("MarginUser", 9)?;
        s.serialize_field("user", &self.user.to_string())?;
        s.serialize_field("bondManager", &self.bond_manager.to_string())?;
        s.serialize_field("eventAdapter", &self.event_adapter.to_string())?;
        s.serialize_field("claims", &self.claims.to_string())?;
        s.serialize_field("bondTicketsStored", &self.bond_tickets_stored)?;
        s.serialize_field("underlyingTokenStored", &self.underlying_token_stored)?;
        s.serialize_field("outstandingObligations", &self.outstanding_obligations)?;
        s.serialize_field("debt", &self.debt.total())?;
        s.serialize_field("nonce", &self.nonce)?;
        s.end()
    }
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

    pub fn cancel_borrow_order(&mut self, amount: u64) -> Result<()> {
        self.pending.try_add_assign(amount)
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
    pub fn fully_repay_obligation<'info, F: Fn() -> Result<Account<'info, Obligation>>>(
        &mut self,
        sequence_number: ObligationSequenceNumber,
        amount_repaid: u64,
        next_obligation: F,
    ) -> Result<()> {
        if sequence_number != self.next_unpaid_obligation_seqno {
            todo!()
        }
        self.committed.try_sub_assign(amount_repaid)?;
        self.next_unpaid_obligation_seqno.try_add_assign(1)?;

        if self.next_unpaid_obligation_seqno < self.next_new_obligation_seqno {
            let next_obligation = next_obligation()?;
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
}

#[derive(Zeroable, Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Assets {
    /// tokens to transfer into settlement account with next position refresh
    pub entitled_tokens: u64,
    /// tickets to transfer into settlement account with next position refresh
    pub entitled_tickets: u64,
    /// reserved data that may be used to determine the size of a user's collateral
    /// pessimistically prepared to persist aggregated values for:
    /// base and quote quantities, separately for bid/ask, on open orders and unsettled fills
    /// 2^3 = 8 u64's
    _reserved0: [u8; 64],
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

#[cfg(feature = "cli")]
impl Serialize for Obligation {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Obligation", 6)?;
        s.serialize_field(
            "MarginUserAccount",
            &self.orderbook_user_account.to_string(),
        )?;
        s.serialize_field("bondManager", &self.bond_manager.to_string())?;
        s.serialize_field("orderTag", &self.order_tag.0)?;
        s.serialize_field("maturationTimestamp", &self.maturation_timestamp)?;
        s.serialize_field("balance", &self.balance)?;
        s.serialize_field("flags", &self.flags.bits())?;
        s.end()
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
