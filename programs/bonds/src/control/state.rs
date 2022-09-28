use anchor_lang::prelude::*;
#[cfg(feature = "cli")]
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

/// The `BondManager` contains all the information necessary to run the bond market
///
/// Utilized by program instructions to verify given transaction accounts are correct. Contains data
/// about the bond market including the tenor and ticket<->token conversion rate
#[cfg_attr(feature = "cli", derive(Deserialize))]
#[account(zero_copy)]
pub struct BondManager {
    /// Versioning and tag information
    pub version_tag: u64,
    /// The address allowed to make changes to this program state
    pub airspace: Pubkey,
    /// The market state of the agnostic orderbook
    pub orderbook_market_state: Pubkey,
    /// The orderbook event queue
    pub event_queue: Pubkey,
    /// The orderbook asks byteslab
    pub asks: Pubkey,
    /// The orderbook bids byteslab
    pub bids: Pubkey,
    /// The token mint for the underlying asset of the bond tickets
    pub underlying_token_mint: Pubkey,
    /// Token account storing the underlying asset accounted for by this ticket program
    pub underlying_token_vault: Pubkey,
    /// The token mint for the bond tickets
    pub bond_ticket_mint: Pubkey,
    /// Mint owned by bonds to issue claims against a user.
    /// These claim notes are monitored by margin to ensure claims are repaid.
    pub claims_mint: Pubkey,
    /// Mint owned by bonds to issue collateral value to a user
    /// The collateral notes are monitored by the margin program to track value
    pub collateral_mint: Pubkey,
    /// oracle that defines the value of the underlying asset
    pub underlying_oracle: Pubkey,
    /// oracle that defines the value of the bond tickets
    pub ticket_oracle: Pubkey,
    /// The user-defined part of the seed that generated this bond manager's PDA
    pub seed: [u8; 32],
    /// The bump seed value for generating the authority address.
    pub(crate) bump: [u8; 1],
    /// Is the market taking orders
    pub orderbook_paused: bool,
    /// Can tickets be redeemed
    pub tickets_paused: bool,
    /// reserved for future use
    pub(crate) _reserved: [u8; 28],
    /// Units added to the initial stake timestamp to determine claim maturity
    pub duration: i64,
    /// Used to generate unique order tags
    pub nonce: u64,
}

impl BondManager {
    /// for signing CPIs with the bond manager account
    pub fn authority_seeds(&self) -> [&[u8]; 4] {
        [
            crate::seeds::BOND_MANAGER,
            self.underlying_token_mint.as_ref(),
            &self.seed,
            &self.bump,
        ]
    }
}

#[cfg(feature = "cli")]
impl Serialize for BondManager {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("BondManager", 14)?;
        s.serialize_field("version", &self.version_tag)?;
        s.serialize_field("airspace", &self.airspace.to_string())?;
        s.serialize_field(
            "orderbookMarketState",
            &self.orderbook_market_state.to_string(),
        )?;
        s.serialize_field("eventQueue", &self.event_queue.to_string())?;
        s.serialize_field("asks", &self.asks.to_string())?;
        s.serialize_field("bids", &self.bids.to_string())?;
        s.serialize_field(
            "underlyingTokenMint",
            &self.underlying_token_mint.to_string(),
        )?;
        s.serialize_field(
            "underlyingTokenVault",
            &self.underlying_token_vault.to_string(),
        )?;
        s.serialize_field("bondTicketMint", &self.bond_ticket_mint.to_string())?;
        s.serialize_field("claimsMint", &self.claims_mint.to_string())?;
        s.serialize_field("collateralMint", &self.collateral_mint.to_string())?;
        s.serialize_field("underlyingOracle", &self.underlying_oracle.to_string())?;
        s.serialize_field("ticketOracle", &self.ticket_oracle.to_string())?;
        s.serialize_field("seed", &Pubkey::new_from_array(self.seed).to_string())?;
        s.serialize_field("orderbookPaused", &self.orderbook_paused)?;
        s.serialize_field("ticketsPaused", &self.tickets_paused)?;
        s.serialize_field("duration", &self.duration)?;
        s.end()
    }
}

/// This authorizes a crank to act on any orderbook within the airspace
#[account]
pub struct CrankAuthorization {
    pub crank: Pubkey,
    pub airspace: Pubkey,
}
