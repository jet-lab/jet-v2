/// construct transactions out of instructions
pub mod transaction;

/// missing implementations for keypair
pub mod keypair {
    use crate::seal;
    use solana_sdk::signature::Keypair;

    /// Clone is not implemented for Keypair
    pub fn clone(keypair: &Keypair) -> Keypair {
        Keypair::from_bytes(&keypair.to_bytes()).unwrap()
    }

    /// Clone is not implemented for Keypair
    pub fn clone_vec(vec: &[Keypair]) -> Vec<Keypair> {
        vec.iter().map(clone).collect()
    }

    /// Clone is not implemented for Keypair
    pub fn clone_refs(vec: &[&Keypair]) -> Vec<Keypair> {
        vec.iter().map(|k| clone(k)).collect()
    }

    /// additional methods for keypair
    pub trait KeypairExt: Sealed {
        /// Clone is not implemented for Keypair. This lets you write the same
        /// code you could use if Clone were implemented.
        fn clone(&self) -> Self;
    }
    seal!(Keypair);

    impl KeypairExt for Keypair {
        fn clone(&self) -> Self {
            clone(self)
        }
    }
}

/// missing implementations for Pubkey
pub mod pubkey {
    use solana_sdk::pubkey::Pubkey;
    use spl_associated_token_account::get_associated_token_address;

    use crate::seal;

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
}
