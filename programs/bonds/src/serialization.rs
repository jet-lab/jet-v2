use std::{
    any::type_name,
    convert::{TryFrom, TryInto},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use anchor_lang::{prelude::*, Discriminator, ZeroCopy};
use arrayref::array_ref;

use crate::{
    orderbook::state::{EventAdapterMetadata, EventQueue},
    BondsError,
};

/// Wrapper for account structs that serializes/deserializes and automatically persists changes
pub struct AnchorAccount<'info, T: AnchorStruct, A: AccessMode = ReadOnly> {
    inner: Account<'info, T>,
    _unused: PhantomData<A>,
}

/// TryFrom - deserialization
impl<'info, T: AnchorStruct, A: AccessMode> TryFrom<&AccountInfo<'info>>
    for AnchorAccount<'info, T, A>
{
    type Error = anchor_lang::error::Error;

    fn try_from(info: &AccountInfo<'info>) -> std::result::Result<Self, Self::Error> {
        if A::is(Mut) && !info.is_writable {
            msg!("not writable {}", info.key);
            return err!(anchor_lang::error::ErrorCode::AccountNotMutable);
        }
        let inner: Account<'info, T> = log_on_error!(
            Account::<T>::try_from(info),
            "failed to deserialize {} to {}",
            info.key,
            type_name::<T>(),
        )?; // checks owner and discriminator

        Ok(AnchorAccount {
            inner,
            _unused: PhantomData,
        })
    }
}

/// TryFrom (owned) - composes above implementation
impl<'info, T: AnchorStruct, A: AccessMode> TryFrom<AccountInfo<'info>>
    for AnchorAccount<'info, T, A>
{
    type Error = anchor_lang::error::Error;

    fn try_from(info: AccountInfo<'info>) -> std::result::Result<Self, Self::Error> {
        (&info).try_into()
    }
}

/// Drop - persists changes automatically
impl<'info, T: AnchorStruct, A: AccessMode> Drop for AnchorAccount<'info, T, A> {
    fn drop(&mut self) {
        if A::is(Mut) {
            self.inner
                .serialize(
                    &mut (&mut self.inner.to_account_info().data.borrow_mut()[8..] as &mut [u8]),
                )
                .unwrap()
        }
    }
}

/// Deref - any account can be read
impl<'info, T: AnchorStruct, A: AccessMode> Deref for AnchorAccount<'info, T, A> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

/// DerefMut - only Mutable accounts can be mutated
impl<'info, T: AnchorStruct> DerefMut for AnchorAccount<'info, T, Mut> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Key - get the pubkey
impl<'info, T: AnchorStruct, A: AccessMode> Key for AnchorAccount<'info, T, A> {
    fn key(&self) -> Pubkey {
        self.inner.key()
    }
}

/// Allows us to load the `AdapterEventQueue` with Anchor discriminator checks
pub struct AdapterLoader<'info, T: ZeroCopy + Owner> {
    phantom: PhantomData<&'info T>,
}

impl<'info, T: ZeroCopy + Owner> AdapterLoader<'info, T> {
    #[inline(never)]
    fn run_checks(acc_info: &AccountInfo<'info>) -> Result<()> {
        if acc_info.owner != &T::owner() {
            return Err(Error::from(ErrorCode::AccountOwnedByWrongProgram)
                .with_pubkeys((*acc_info.owner, T::owner())));
        }
        let data: &[u8] = &acc_info.try_borrow_data()?;
        if data.len() < T::discriminator().len() {
            return Err(ErrorCode::AccountDiscriminatorNotFound.into());
        }
        // Discriminator must match.
        let disc_bytes = array_ref![data, 0, 8];
        if disc_bytes != &T::discriminator() {
            return Err(ErrorCode::AccountDiscriminatorMismatch.into());
        }

        // AccountInfo api allows you to borrow mut even if the account isn't
        // writable, so add this check for a better dev experience.
        if !acc_info.is_writable {
            return Err(ErrorCode::AccountNotMutable.into());
        }

        Ok(())
    }

    /// Runs discriminator checks and returns a mutable AdapterEventQueue
    pub fn load_adapter(acc_info: &AccountInfo<'info>) -> Result<EventQueue<'info>> {
        log_on_error!(
            Self::run_checks(acc_info),
            "provided adapter account failed the checks {:?}",
            acc_info.key
        )?;

        EventQueue::deserialize_user_adapter(acc_info.clone())
    }
}

// Specifies in the type whether the account should be writable
algebraic! {
    AccessMode {
        ReadOnly,
        Mut,
    }
}

/// Directly deserialize accounts from the remaining_accounts iterator
pub trait RemainingAccounts<'a, 'info: 'a>: Iterator<Item = &'a AccountInfo<'info>> {
    fn next_account(&mut self) -> Result<&'a AccountInfo<'info>> {
        Ok(self.next().ok_or(BondsError::NoMoreAccounts)?)
    }

    fn next_anchor<T: AnchorStruct, A: AccessMode>(
        &mut self,
    ) -> Result<AnchorAccount<'info, T, A>> {
        self.next_account()?.to_owned().try_into()
    }

    fn next_adapter(&mut self) -> Result<EventQueue<'info>> {
        AdapterLoader::<EventAdapterMetadata>::load_adapter(self.next_account()?)
    }

    fn maybe_next_adapter(&mut self) -> Result<Option<EventQueue<'info>>> {
        self.next()
            .map(AdapterLoader::<EventAdapterMetadata>::load_adapter)
            .transpose()
    }

    fn init_next<T: AnchorStruct>(
        &mut self,
        payer: AccountInfo<'info>,
        system_program: AccountInfo<'info>,
        seeds: &[&[u8]],
    ) -> Result<AnchorAccount<'info, T, Mut>> {
        init(
            self.next_account()?.to_owned(),
            payer,
            system_program,
            seeds,
        )
    }
}
impl<'a, 'info: 'a, T: Iterator<Item = &'a AccountInfo<'info>>> RemainingAccounts<'a, 'info> for T {}

// pub struct AnchorAccountIniterator<'c, 'info: 'c, T: AnchorStruct, I: Iterator<Item = &'c AccountInfo<'info>>> {
//     inner: I,
//     _account_type: PhantomData<T>,
// }

// impl<'c, 'info: 'c, T: AnchorStruct, I: Iterator<Item = &'c AccountInfo<'info>>> Iterator for AnchorAccountIniterator<'c, 'info, T, I> {
//     type Item = AnchorAccount<'info, T, Mut>;

//     fn next(&mut self) -> Option<Self::Item> {
//         init(
//             self.inner.next().to_owned(),
//             payer,
//             system_program,
//             seeds,
//         )
//     }
// }

/// initialize a PDA and return the mutable struct
pub fn init<'info, T: AnchorStruct>(
    new_account: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    seeds: &[&[u8]],
) -> Result<AnchorAccount<'info, T, Mut>> {
    let space = 8 + std::mem::size_of::<T>();
    let (new_pubkey, nonce) = Pubkey::find_program_address(seeds, &T::owner());
    if new_pubkey != *new_account.key {
        msg!(
            "Provided account was {:?} but seeds generate {:?}",
            new_account.key,
            new_pubkey
        );
        return Err(ProgramError::InvalidSeeds.into());
    }

    let mut signer = seeds.to_vec();
    let bump = &[nonce];
    signer.push(bump);
    let ix = anchor_lang::solana_program::system_instruction::create_account(
        payer.key,
        &new_pubkey,
        Rent::get()?.minimum_balance(space),
        space as u64,
        &T::owner(),
    );
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[payer, new_account.clone(), system_program],
        &[&signer[..]],
    )
    .map_err(|_| error!(crate::errors::BondsError::InvokeCreateAccount))?;

    {
        let mut data = new_account.data.borrow_mut();
        for byte in 0..8 {
            data[byte] = T::discriminator()[byte];
        }
    }

    new_account.try_into()
}

/// Consolidates all required interfaces of anchor accounts into a single trait for readability
pub trait AnchorStruct:
    AnchorSerialize + AccountSerialize + AccountDeserialize + Owner + Clone + Discriminator
{
}
impl<T> AnchorStruct for T where
    T: AnchorSerialize + AccountSerialize + AccountDeserialize + Owner + Clone + Discriminator
{
}

pub trait AnchorZeroCopy: ZeroCopy + Owner {}
impl<T> AnchorZeroCopy for T where T: ZeroCopy + Owner {}

/// enum-like behavior except the "enum" is a trait and all the variants are structs
/// approximates using an enum as a const generic (currently unstable in rust)
macro_rules! algebraic {
    ($Name:ident { $($Variant:ident),*$(,)? }) => {
        pub trait $Name: sealed::$Name + Sized {
            const VAL: enumerated::$Name;
            fn is<T: $Name>(_: T) -> bool {
                Self::VAL == T::VAL
            }
        }
        $(pub struct $Variant;
        impl sealed::$Name for $Variant {}
        impl $Name for $Variant {
            const VAL: enumerated::$Name = enumerated::$Name::$Variant;
        })*
        mod sealed {
            pub trait $Name {}
        }
        mod enumerated {
            #[derive(PartialEq, Eq)]
            pub enum $Name {$($Variant,)*}
        }
    };
}
use algebraic;

macro_rules! log_on_error {
    ($result:expr, $($args:tt)*) => {{
        if $result.is_err() {
            msg!($($args)*);
        }
        $result
    }};
}
pub(crate) use log_on_error;
