use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::pubkey;

pub const ADMINISTRATOR: Pubkey = pubkey!("7R6FjP2HfXAgKQjURC4tCBrUmRQLCgEUeX2berrfU4ox");

mod admin_transfer_loan;

pub use admin_transfer_loan::*;
