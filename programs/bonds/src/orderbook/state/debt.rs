use anchor_lang::{prelude::*, solana_program::clock::UnixTimestamp};
use bytemuck::Zeroable;
use jet_proto_math::traits::{TryAddAssign, TrySubAssign};

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

    pub fn add_pending_debt(&mut self, amount: u64) -> Result<()> {
        self.pending.try_add_assign(amount)
    }

    pub fn commit(&mut self, amount: u64) -> Result<()> {
        self.pending.try_sub_assign(amount)?;
        self.committed.try_add_assign(amount)?;

        Ok(())
    }

    pub fn mark_due(&mut self, amount: u64) -> Result<()> {
        self.committed.try_sub_assign(amount)?;
        self.past_due.try_add_assign(amount)?;

        Ok(())
    }

    pub fn cancel_pending(&mut self, amount: u64) -> Result<()> {
        self.pending.try_sub_assign(amount)
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

#[account]
#[derive(Debug)]
pub struct Obligation {
    /// The user (margin account) this obligation is owed by
    pub orderbook_user_account: Pubkey,

    /// The bond manager where the obligation was created
    pub bond_manager: Pubkey,

    /// The `OrderTag` associated with the creation of this `Obligation`
    pub order_tag: [u8; 16],

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
        let mut s = serializer.serialize_struct("Obligation", 4)?;
        s.serialize_field(
            "orderbookUserAccount",
            &self.orderbook_user_account.to_string(),
        )?;
        s.serialize_field("bondManager", &self.bond_manager.to_string())?;
        s.serialize_field("maturationTimestamp", &self.maturation_timestamp)?;
        s.serialize_field("balance", &self.balance)?;
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
