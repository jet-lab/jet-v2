use solana_program::{pubkey, pubkey::Pubkey};

#[cfg(not(feature = "devnet"))]
mod network_addrs {
    use super::*;

    pub const ORCA_V2: Pubkey = ORCA_V2_MAINNET;
}

#[cfg(feature = "devnet")]
mod network_addrs {
    use super::*;

    pub const ORCA_V2: Pubkey = ORCA_V2_DEVNET;
}

pub use network_addrs::*;

pub const ORCA_V2_MAINNET: Pubkey = pubkey!("9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP");
pub const ORCA_V2_DEVNET: Pubkey = pubkey!("3xQ8SWv2GaFXXpHZNqkXsdxq5DZciHBz6ZFoPPfbFd7U");
pub const ORCA_WHIRLPOOL: Pubkey = pubkey!("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");
pub const SABER: Pubkey = pubkey!("SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ");
pub const OPENBOOK: Pubkey = pubkey!("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX");
pub const OPENBOOK_DEVNET: Pubkey = pubkey!("EoTcMgcDRTJVZDMZWBoU6rhYHZfkNTVEAfz3uUJRcYGj");
