//! This file only has simple functions with explicit, bare minimum
//! dependencies, that act only as a thin wrapper around anchor's auto-generated
//! sdk in the program crate.
//!
//! These functions should:
//! - be named after an instruction in the crate
//! - simply construct and return an Instruction for a specific instruction in
//!   the program.
//! - derive pdas that can be derived from other inputs
//! - Facilitate construction of the Instruction struct
//!
//! These functions should NOT:
//! - be methods of a struct
//! - depend on complex data types that contain data that is not necessary for
//!   the instruction. Only use a struct of pubkeys as a param if EVERY pubkey
//!   is used by that function!
//! - have any side effects such as mutations or sending requests
//! - be async

use anchor_lang::{prelude::Pubkey, InstructionData, ToAccountMetas};
use jet_fixed_term::control::instructions::InitializeMarketParams;
use solana_sdk::instruction::Instruction;

use super::derive::{claims_mint, market, ticket_collateral, ticket_mint, underlying_token_vault};

pub fn initialize_market(
    params: InitializeMarketParams,
    airspace: Pubkey,
    underlying_token_mint: Pubkey,
    authority: Pubkey,
    underlying_oracle: Pubkey,
    ticket_oracle: Pubkey,
    fee_destination: Pubkey,
    payer: Pubkey,
) -> Instruction {
    let market = market(&airspace, &underlying_token_mint, params.seed);
    let data = jet_fixed_term::instruction::InitializeMarket { params }.data();
    let accounts = jet_fixed_term::accounts::InitializeMarket {
        underlying_token_vault: underlying_token_vault(&market),
        ticket_mint: ticket_mint(&market),
        claims: claims_mint(&market),
        collateral: ticket_collateral(&market),
        market,
        underlying_token_mint,
        airspace,
        authority,
        underlying_oracle,
        ticket_oracle,
        fee_destination,
        payer,
        rent: solana_sdk::sysvar::rent::ID,
        token_program: spl_token::ID,
        system_program: solana_sdk::system_program::ID,
    }
    .to_account_metas(None);
    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}
