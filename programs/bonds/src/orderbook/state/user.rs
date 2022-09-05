use anchor_lang::prelude::*;

use super::debt::Debt;

/// The orderbook user account tracks data about a user's state within the Bonds Orderbook
#[account]
#[derive(Debug)]
pub struct OrderbookUser {
    /// The pubkey of the user. Used to verfiy withdraws, etc.
    pub user: Pubkey,
    /// The pubkey pointing to the BondMarket account tracking this user
    pub bond_manager: Pubkey,
    /// The address of the registered event adapter for this user
    pub event_adapter: Pubkey,
    /// The quanitity of base token the user may allocate to orders or withdraws
    /// For the bonds program, this represents the bond tickets
    pub bond_tickets_stored: u64,
    /// The quantity of quote token the user may allocate to orders or withdraws
    /// For the bonds program, this represents the asset redeeemed for staking bond tickets
    pub underlying_token_stored: u64,

    /// total number of outstanding obligations with committed debt
    pub outstanding_obligations: u64,

    /// The amount of debt that must be collateralized or repaid
    /// This debt is expressed in terms of the underlying token - not bond tickets
    pub debt: Debt,

    /// Token account used by the margin program to track the debt
    pub claims: Pubkey,

    /// This nonce is used to generate unique order tags
    /// Instantiated as `0` and incremented with each order
    pub nonce: u64,
}

#[cfg(feature = "cli")]
impl Serialize for OrderbookUser {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("OrderbookUser", 8)?;
        s.serialize_field("user", &self.user.to_string())?;
        s.serialize_field("bondManager", &self.bond_manager.to_string())?;
        s.serialize_field("bondTicketsStored", &self.bond_tickets_stored)?;
        s.serialize_field("underlyingTokenStored", &self.underlying_token_stored)?;
        s.serialize_field("outstandingObligations", &self.outstanding_obligations)?;
        s.serialize_field("pendingDebt", &self.pending_debt)?;
        s.serialize_field("committedDebt", &self.committed_debt)?;
        s.serialize_field("nonce", &self.nonce)?;
        s.end()
    }
}
