use std::sync::Arc;

use anchor_lang::prelude::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};

pub mod data;
pub mod keypair;
pub mod pubkey;

/// Produce a trait that you can use in the current module to seal other traits.
/// Sealing a trait means it can only be implemented for the provided types.
///
/// Simple: get a single `Sealed` trait to have a single sealing pattern.
/// ```ignore
/// seal!(u8)
/// trait u8ext: Sealed {}
/// ```
///
/// Advanced: provide a name for the sealing trait if you might need multiple.
/// ```ignore
/// seal!(u8Sealed: u8)
/// trait u8ext: u8Sealed {}
///
/// seal!(u32Sealed: u32)
/// trait u32ext: u32Sealed {}
/// ```
///
/// You can also seal a trait to multiple types.
/// ```ignore
/// seal!(uintSealed: u8, u16, u32, u64, usize)
/// trait uintExt: uintSealed {}
/// ```
///
#[macro_export]
macro_rules! seal {
    ($($Type:ty),+$(,)?) => {
        seal!(Sealed: $($Type),*);
    };
    ($Sealed:ident: $($Type:ty),+$(,)?) => {
        paste::paste! {
            mod [<mod_for_ $Sealed:snake>] {
                use super::*;
                pub trait $Sealed {}
                $(impl $Sealed for $Type {})+
            }
            use [<mod_for_ $Sealed:snake>]::$Sealed;
        }
    };
}

/// A signer or pubkey for a solana account. Use when you generically want
/// anything that has an address, but you don't care if it can sign.
pub trait Key {
    /// The public key of the account.
    fn address(&self) -> Pubkey;
}

impl Key for Pubkey {
    fn address(&self) -> Pubkey {
        *self
    }
}

impl Key for Keypair {
    fn address(&self) -> Pubkey {
        self.pubkey()
    }
}

impl Key for Arc<Keypair> {
    fn address(&self) -> Pubkey {
        self.pubkey()
    }
}

/// Clone the item and move it into the async closure.
#[macro_export]
macro_rules! clone_to_async {
    (
        ($($to_move:ident $(= $orig_name:expr)?),*)
        |$(mut $arg:ident),*|
        $blk:expr
    ) => {{
        $(
            $(let $to_move = $orig_name.clone();)?
            let $to_move = $to_move.clone();
        )*
        move |$(mut $arg),*| {
            $(let $to_move = $to_move.clone();)*
            async move { $blk }
        }
    }};
    (
        ($($to_move:ident $(= $orig_name:expr)?),*)
        |$($arg:ident),*|
        $blk:expr
    ) => {{
        $(
            $(let $to_move = $orig_name.clone();)?
            let $to_move = $to_move.clone();
        )*
        move |$($arg),*| {
            $(let $to_move = $to_move.clone();)*
            async move { $blk }
        }
    }};
}
