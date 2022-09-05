use agnostic_orderbook::state::OrderSummary;
use anchor_lang::{prelude::*, solana_program::clock::UnixTimestamp};
use bytemuck::Zeroable;
use jet_margin::{AdapterResult, MarginAccount};
use jet_proto_math::traits::{TryAddAssign, TrySubAssign};

use crate::orderbook::state::OrderTag;

/// An acocunt used to track margin users of the market
#[account]
#[derive(Debug)]
pub struct MarginUser {
    /// The margin account used for signing actions
    pub margin_account: Pubkey,
    /// The `BondManager` for the market
    pub bond_manager: Pubkey,
    /// Token account used by the margin program to track the debt
    pub claims: Pubkey,
    /// Token account used by the margin program to track deposited asset value
    pub deposits: Pubkey,
    /// total number of outstanding obligations with committed debt
    pub outstanding_obligations: u64,
    /// The amount of debt that must be collateralized or repaid
    /// This debt is expressed in terms of the underlying token - not bond tickets
    pub debt: Debt,
    /// Accounting used to track assets in custody of the bond market
    pub assets: Assets,
}

impl MarginUser {
    /// Accounts debt for a borrow order
    pub fn borrow(&mut self, order_summary: &OrderSummary) -> Result<()> {
        if order_summary.total_base_qty > 0 {
            self.outstanding_obligations += 1;
        }
        self.debt.borrow_order(order_summary)
    }
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
    /// Amount that must be collateralized because there is an open order for it.
    /// Does not accrue interest because the loan has not been received yet.
    pending: u64,

    /// Debt that has already been borrowed because the order was matched.
    /// This debt will be due when the loan term ends.
    /// Some of this debt may actually be due already, but a crank has not yet been marked it as due.
    committed: u64,

    /// Amount of debt that has already been discovered and marked as being due
    /// This is not guaranteed to be comprehensive. It may not include some
    /// obligations that have not yet been marked due.
    past_due: u64,
}

impl Debt {
    pub fn total(&self) -> u64 {
        self.pending
            .checked_add(self.committed)
            .unwrap()
            .checked_add(self.past_due)
            .unwrap()
    }

    pub fn borrow_order(&mut self, order_summary: &OrderSummary) -> Result<()> {
        self.pending
            .try_add_assign(order_summary.total_base_qty_posted)?;
        self.committed.try_add_assign(order_summary.total_base_qty)
    }

    pub fn process_fill(&mut self, amount: u64) -> Result<()> {
        self.pending.try_sub_assign(amount)?;
        self.committed.try_add_assign(amount)?;
        Ok(())
    }

    pub fn process_out(&mut self, amount: u64) -> Result<()> {
        self.pending.try_sub_assign(amount)
    }

    pub fn mark_due(&mut self, amount: u64) -> Result<()> {
        self.committed.try_sub_assign(amount)?;
        self.past_due.try_add_assign(amount)?;

        Ok(())
    }

    pub fn repay_committed(&mut self, amount: u64) -> Result<()> {
        self.committed.try_sub_assign(amount)
    }

    pub fn repay_past_due(&mut self, amount: u64) -> Result<()> {
        self.past_due.try_sub_assign(amount)
    }

    pub fn is_past_due(&self) -> bool {
        self.past_due > 0
    }
}

#[derive(Zeroable, Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Assets {
    /// underlying token tied up in open orders
    open_orders: u64,
    /// bond tickets tied up in `SplitTicket` structs
    open_tickets: u64,
}

#[account]
#[derive(Debug)]
pub struct Obligation {
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
