//! missing implementations for pubkey

use crate::seal;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;

/// provides or_ata method for Option<Pubkey>
pub trait OrAta: Sealed {
    /// Use when a token account address is an optional parameter to some
    /// function, and you want to resolve None to the ATA for a particular
    /// wallet.
    fn or_ata(&self, wallet: &Pubkey, mint: &Pubkey) -> Pubkey;
}
seal!(Option<Pubkey>);

impl OrAta for Option<Pubkey> {
    fn or_ata(&self, wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
        self.unwrap_or_else(|| get_associated_token_address(wallet, mint))
    }
}
