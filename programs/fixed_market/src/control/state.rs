use anchor_lang::prelude::*;
#[cfg(any(feature = "cli", test))]
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

use crate::margin::origination_fee;

/// The `Market` contains all the information necessary to run the fixed term market
///
/// Utilized by program instructions to verify given transaction accounts are correct. Contains data
/// about the fixed market including the tenor and ticket<->token conversion rate
#[cfg_attr(any(feature = "cli", test), derive(Deserialize))]
#[account(zero_copy)]
pub struct Market {
    /// Versioning and tag information
    pub version_tag: u64,
    /// The airspace the market is a part of
    pub airspace: Pubkey,
    /// The market state of the agnostic orderbook
    pub orderbook_market_state: Pubkey,
    /// The orderbook event queue
    pub event_queue: Pubkey,
    /// The orderbook asks byteslab
    pub asks: Pubkey,
    /// The orderbook bids byteslab
    pub bids: Pubkey,
    /// The token mint for the underlying asset of the market tickets
    pub underlying_token_mint: Pubkey,
    /// Token account storing the underlying asset accounted for by this ticket program
    pub underlying_token_vault: Pubkey,
    /// The token mint for the market tickets
    pub market_ticket_mint: Pubkey,
    /// Mint owned by Jet markets to issue claims against a user.
    /// These claim notes are monitored by margin to ensure claims are repaid.
    pub claims_mint: Pubkey,
    /// Mint owned by Jet markets to issue collateral value to a user
    /// The collateral notes are monitored by the margin program to track value
    pub collateral_mint: Pubkey,
    /// oracle that defines the value of the underlying asset
    pub underlying_oracle: Pubkey,
    /// oracle that defines the value of the market tickets
    pub ticket_oracle: Pubkey,
    /// where fees can be withdrawn to
    pub fee_destination: Pubkey,
    /// The user-defined part of the seed that generated this market's PDA
    pub seed: [u8; 32],
    /// The bump seed value for generating the authority address.
    pub(crate) bump: [u8; 1],
    /// Is the market taking orders
    pub orderbook_paused: bool,
    /// Can tickets be redeemed
    pub tickets_paused: bool,
    /// reserved for future use
    pub(crate) _reserved: [u8; 28],
    /// Length of time before a borrow is marked as due, in seconds
    pub borrow_tenor: i64,
    /// Length of time before a claim is marked as mature, in seconds
    pub lend_tenor: i64,
    /// assessed on borrows. scaled by origination_fee::FEE_UNIT
    pub origination_fee: u64,
    /// amount of fees currently available to be withdrawn by market owner
    pub collected_fees: u64,
    /// Used to generate unique order tags
    pub nonce: u64,
}

impl Market {
    /// for signing CPIs with the market account
    pub fn authority_seeds(&self) -> [&[u8]; 5] {
        [
            crate::seeds::MARKET,
            self.airspace.as_ref(),
            self.underlying_token_mint.as_ref(),
            &self.seed,
            &self.bump,
        ]
    }

    /// how much a borrower should receive from their fill after an origination fee is assessed
    pub fn loan_to_disburse(&self, filled_quote: u64) -> u64 {
        origination_fee::loan_to_disburse(filled_quote, self.origination_fee)
    }

    /// the size a borrow order should have including the requested amount plus the origination fee
    pub fn borrow_order_qty(&self, requested: u64) -> u64 {
        origination_fee::borrow_order_qty(requested, self.origination_fee)
    }
}

#[cfg(any(feature = "cli", test))]
impl Serialize for Market {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Market", 14)?;
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
        s.serialize_field("marketTicketMint", &self.market_ticket_mint.to_string())?;
        s.serialize_field("claimsMint", &self.claims_mint.to_string())?;
        s.serialize_field("collateralMint", &self.collateral_mint.to_string())?;
        s.serialize_field("underlyingOracle", &self.underlying_oracle.to_string())?;
        s.serialize_field("ticketOracle", &self.ticket_oracle.to_string())?;
        s.serialize_field("seed", &Pubkey::new_from_array(self.seed).to_string())?;
        s.serialize_field("orderbookPaused", &self.orderbook_paused)?;
        s.serialize_field("ticketsPaused", &self.tickets_paused)?;
        s.serialize_field("borrowTenor", &self.borrow_tenor)?;
        s.serialize_field("lendTenor", &self.lend_tenor)?;
        s.end()
    }
}

/// This authorizes a crank to act on any orderbook within the airspace
#[account]
pub struct CrankAuthorization {
    pub crank: Pubkey,
    pub airspace: Pubkey,
}

#[test]
fn serialize_market() {
    let json = serde_json::to_string_pretty(&<Market as bytemuck::Zeroable>::zeroed()).unwrap();
    let expected = "{
      \"version\": 0,
      \"airspace\": \"11111111111111111111111111111111\",
      \"orderbookMarketState\": \"11111111111111111111111111111111\",
      \"eventQueue\": \"11111111111111111111111111111111\",
      \"asks\": \"11111111111111111111111111111111\",
      \"bids\": \"11111111111111111111111111111111\",
      \"underlyingTokenMint\": \"11111111111111111111111111111111\",
      \"underlyingTokenVault\": \"11111111111111111111111111111111\",
      \"marketTicketMint\": \"11111111111111111111111111111111\",
      \"claimsMint\": \"11111111111111111111111111111111\",
      \"collateralMint\": \"11111111111111111111111111111111\",
      \"underlyingOracle\": \"11111111111111111111111111111111\",
      \"ticketOracle\": \"11111111111111111111111111111111\",
      \"seed\": \"11111111111111111111111111111111\",
      \"orderbookPaused\": false,
      \"ticketsPaused\": false,
      \"borrowTenor\": 0,
      \"lendTenor\": 0
    }";
    assert_eq!(
        itertools::Itertools::join(&mut expected.split_whitespace(), " "),
        itertools::Itertools::join(&mut json.split_whitespace(), " ")
    )
}
