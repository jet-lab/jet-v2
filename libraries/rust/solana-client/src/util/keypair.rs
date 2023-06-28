//! missing implementations for keypair

use crate::seal;
use solana_sdk::signature::Keypair;

/// Clone is not implemented for Keypair
pub fn clone<K: ToKeypair>(keypair: K) -> Keypair {
    keypair.to_keypair()
}

/// Clone is not implemented for Keypair
pub fn clone_vec<K: ToKeypair>(vec: impl IntoIterator<Item = K>) -> Vec<Keypair> {
    vec.into_iter().map(ToKeypair::to_keypair).collect()
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

pub trait ToKeypair {
    fn to_keypair(self) -> Keypair;
}

impl ToKeypair for Keypair {
    fn to_keypair(self) -> Keypair {
        self
    }
}

impl ToKeypair for &Keypair {
    fn to_keypair(self) -> Keypair {
        Keypair::from_bytes(&self.to_bytes()).unwrap()
    }
}

pub trait ToKeypairs {
    fn to_keypairs(self) -> Vec<Keypair>;
}

impl<I: IntoIterator<Item = K>, K: ToKeypair> ToKeypairs for I {
    fn to_keypairs(self) -> Vec<Keypair> {
        self.into_iter().map(ToKeypair::to_keypair).collect()
    }
}
