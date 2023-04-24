use anchor_lang::AccountDeserialize;
use serde::{
    ser::{Serialize, SerializeStruct},
    Serializer,
};
use wasm_bindgen::prelude::*;

use jet_fixed_term::{
    control::state::Market,
    margin::state::{Assets, Debt, MarginUser},
};

use crate::{bindings::serialization::JsAnchorDeserialize, JsResult};

#[wasm_bindgen(typescript_custom_section)]
const MARKET_INFO: &'static str = r#"
/**
 * The anchor struct containing Market information
 */
export interface MarketInfo {
    versionTag: bigint
    airspace: string
    orderbookMarketState: string
    eventQueue: string
    asks: string
    bids: string
    underlyingTokenMint: string
    underlyingTokenVault: string
    ticketMint: string
    claimsMint: string
    ticketCollateralMint: string
    underlyingCollateralMint: string
    underlyingOracle: string
    ticketOracle: string
    feeVault: string
    feeDestination: string
    seed: string
    orderbookPaused: boolean
    ticketsPaused: boolean
    borrowTenor: bigint
    lendTenor: bigint
    originationFee: bigint
}
"#;

#[wasm_bindgen(js_name = "deserializeMarketFromBuffer")]
pub fn deserialize_market(buf: &[u8]) -> JsResult {
    Market::deserialize_from_buffer(buf)
}

#[wasm_bindgen(typescript_custom_section)]
const MARGIN_USER_INFO: &'static str = r#"
export interface MarginUserInfo {
    versionTag: number,
    marginAccount: string,
    market: string,
    claims: string,
    ticketCollateral: string,
    underlyingCollateral: string,
    debt: Debt,
    assets: Assets,
    borrowRollConfig?: BorrowAutoRollConfig,
    lendRollConfig?: LendAutoRollConfig,
}

export interface Debt {
    nextNewTermLoanSeqno: bigint,
    nextUnpaidTermLoanSeqno: bigint,
    nextTermLoanMaturity: bigint,
    pending: bigint,
    committed: bigint,
}

export interface Assets {
    entitledTokens: bigint,
    entitledTickets: bigint,
    nextDepositSeqno: bigint,
    nextUnredeemedDepositSeqno: bigint,
    ticketsStaked: bigint,
    ticketsPosted: bigint,
    tokensPosted: bigint,
}
"#;

pub(crate) struct MarginUserDeserializer(MarginUser);

impl Serialize for MarginUserDeserializer {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("MarginUser", 10)?;
        s.serialize_field("versionTag", &self.0.version)?;
        s.serialize_field("marginAccount", &self.0.margin_account.to_string())?;
        s.serialize_field("market", &self.0.market.to_string())?;
        s.serialize_field("claims", &self.0.claims.to_string())?;
        s.serialize_field("ticketCollateral", &self.0.ticket_collateral.to_string())?;
        s.serialize_field(
            "underlyingCollateral",
            &self.0.underlying_collateral.to_string(),
        )?;
        s.serialize_field("debt", &DebtSerializer(self.0.debt()))?;
        s.serialize_field("assets", &AssetsSerializer(self.0.assets()))?;
        s.serialize_field("borrowRollConfig", &self.0.borrow_roll_config)?;
        s.serialize_field("lendRollConfig", &self.0.lend_roll_config)?;
        s.end()
    }
}

impl AccountDeserialize for MarginUserDeserializer {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        Ok(Self(MarginUser::try_deserialize_unchecked(buf)?))
    }
}

struct AssetsSerializer<'a>(&'a Assets);

impl<'a> Serialize for AssetsSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Assets", 7)?;
        s.serialize_field("entitledTokens", &self.0.entitled_tokens())?;
        s.serialize_field("entitledTickets", &self.0.entitled_tickets())?;
        s.serialize_field("nextDepositSeqno", &self.0.next_new_deposit_seqno())?;
        s.serialize_field(
            "nextUnredeemedDepositSeqno",
            &self.0.next_unredeemed_deposit_seqno(),
        )?;
        s.serialize_field("ticketsStaked", &self.0.tickets_staked())?;
        s.serialize_field("ticketsPosted", &self.0.tickets_posted())?;
        s.serialize_field("tokensPosted", &self.0.tokens_posted())?;
        s.end()
    }
}

struct DebtSerializer<'a>(&'a Debt);

impl<'a> Serialize for DebtSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Debt", 5)?;
        s.serialize_field("nextNewTermLoanSeqno", &self.0.next_new_loan_seqno())?;
        s.serialize_field("nextUnpaidTermLoanSeqno", &self.0.next_term_loan_to_repay())?;
        s.serialize_field("nextTermLoanMaturity", &self.0.next_term_loan_maturity())?;
        s.serialize_field("pending", &self.0.pending())?;
        s.serialize_field("committed", &self.0.committed())?;
        s.end()
    }
}

#[wasm_bindgen(js_name = "deserializeMarginUserFromBuffer")]
pub fn deserialize_margin_user(buf: &[u8]) -> JsResult {
    MarginUserDeserializer::deserialize_from_buffer(buf)
}
