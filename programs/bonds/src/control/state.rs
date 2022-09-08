use anchor_lang::prelude::*;
use jet_proto_math::traits::{SafeDiv, SafeMul};
#[cfg(feature = "cli")]
use serde::{ser::SerializeStruct, Serialize, Serializer};

/// The `BondManager` contains all the information necessary to run the bond market
///
/// Utilized by program instructions to verify given transaction accounts are correct. Contains data
/// about the bond market including the tenor and ticket<->token conversion rate
#[account(zero_copy)]
pub struct BondManager {
    /// Versioning and tag information
    pub version_tag: u64,
    /// The address allowed to make changes to this program state
    pub program_authority: Pubkey,
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
    /// The deposit notes are monitored by the margin program to track value
    pub deposits_mint: Pubkey,
    /// oracle that defines the value of the underlying asset
    pub underlying_oracle: Pubkey,
    /// oracle that defines the value of the bond tickets
    pub ticket_oracle: Pubkey,
    /// The user-defined part of the seed that generated this bond manager's PDA
    pub seed: [u8; 8],
    /// The bump seed value for generating the authority address.
    pub(crate) bump: [u8; 1],
    /// The number of decimals added or subtracted to the tickets staked when minting a `ClaimTicket`
    pub conversion_factor: i8,
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

    /// Convert bond tickets to their equivalent token value
    pub fn convert_tickets(&self, ticket_amount: u64) -> Result<u64> {
        match self.conversion_factor {
            decimals if decimals < 0 => {
                ticket_amount.safe_div(10u64.pow(decimals.unsigned_abs() as u32))
            }
            decimals if decimals > 0 => {
                ticket_amount.safe_mul(10u64.pow(decimals.unsigned_abs() as u32))
            }
            _ => Ok(ticket_amount),
        }
    }

    /// Convert underlying tokens to their bond ticket value
    pub fn convert_tokens(&self, token_amount: u64) -> Result<u64> {
        match self.conversion_factor {
            decimals if decimals < 0 => {
                token_amount.safe_mul(10u64.pow(decimals.unsigned_abs() as u32))
            }
            decimals if decimals > 0 => {
                token_amount.safe_div(10u64.pow(decimals.unsigned_abs() as u32))
            }
            _ => Ok(token_amount),
        }
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
        s.serialize_field("programAuthority", &self.program_authority.to_string())?;
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
        s.serialize_field("oracle", &self.oracle.to_string())?;
        s.serialize_field("seed", &self.seed)?;
        s.serialize_field("conversionFactor", &self.conversion_factor)?;
        s.serialize_field("duration", &self.duration)?;
        s.end()
    }
}
