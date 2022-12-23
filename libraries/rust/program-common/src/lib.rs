mod fp32;
mod functions;
mod number;
mod number_128;

pub mod traits;

#[doc(inline)]
pub use functions::*;

#[doc(inline)]
pub use number::*;

#[doc(inline)]
pub use number_128::*;

#[doc(inline)]
pub use fp32::*;

use solana_program::{pubkey, pubkey::Pubkey};

pub const ADMINISTRATOR: Pubkey = pubkey!("7R6FjP2HfXAgKQjURC4tCBrUmRQLCgEUeX2berrfU4ox");
