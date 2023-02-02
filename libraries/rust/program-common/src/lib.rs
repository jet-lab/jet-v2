mod fp32;
mod functions;
mod number;
mod number_128;

pub mod serialization;
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
