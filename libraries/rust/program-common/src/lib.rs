#![allow(clippy::result_large_err)]

mod fp32;
mod functions;
pub mod log;
mod number;
mod number_128;

pub mod interest_pricing;
pub mod pod;
pub mod programs;
pub mod serialization;
pub mod traits;
pub mod map;
pub mod seeds;

#[doc(inline)]
pub use functions::*;

#[doc(inline)]
pub use number::*;

#[doc(inline)]
pub use number_128::*;

#[doc(inline)]
pub use fp32::*;

use solana_program::{pubkey, pubkey::Pubkey};

#[cfg(not(feature = "devnet"))]
mod governor_addresses {
    use super::*;

    pub const GOVERNOR_ID: Pubkey = GOVERNOR_MAINNET;
    pub const GOVERNOR_PAYER: Pubkey = GOVERNOR_MAINNET_PAYER;
}

#[cfg(feature = "devnet")]
mod governor_addresses {
    use super::*;

    pub const GOVERNOR_ID: Pubkey = GOVERNOR_DEVNET;
    pub const GOVERNOR_PAYER: Pubkey = GOVERNOR_DEVNET_PAYER;
}

pub use governor_addresses::*;

pub const GOVERNOR_MAINNET: Pubkey = pubkey!("7R6FjP2HfXAgKQjURC4tCBrUmRQLCgEUeX2berrfU4ox");
pub const GOVERNOR_MAINNET_PAYER: Pubkey = pubkey!("2J2K1wHK3U8bsow1shUZJvEx1L2og2h5T5JGPqBS1uKA");
pub const GOVERNOR_DEVNET: Pubkey = pubkey!("4DePZb9T6PD1bWC8pn9htYvs2QHY1VtwjW5TEDWjjDWd");
pub const GOVERNOR_DEVNET_PAYER: Pubkey = pubkey!("7etg4hgAdUjGnYAuj9E22MWNGLtHJCc1HmnEJLrCb6UN");

/// The control authority that owns accounts such as fee destinations.
pub const CONTROL_AUTHORITY: Pubkey = pubkey!("4W1XXCnJs16UYNwJaGbwWbjPZyuLWeEJ4FfSHeMpdiXY");

pub const DEFAULT_AIRSPACE: Pubkey = pubkey!("BwQhHumhyyyRBtCsiSrdnFCinJDCaaMBbbyRhqJ5p81d");

/// The lookup table registry program ID is added here as a convenience to avoid
/// importing the crate just to get the ID.
pub const ADDRESS_LOOKUP_REGISTRY_ID: Pubkey =
    pubkey!("LTR8xXcSrEDsCbTWPY4JmJREFdMz4uYh65uajkVjzru");

pub const GOVERNANCE_PROGRAM: Pubkey = pubkey!("JPGov2SBA6f7XSJF5R4Si5jEJekGiyrwP2m7gSEqLUs");
pub const GOVERNANCE_REALM_DAO: &'static str = "Jet DAO";