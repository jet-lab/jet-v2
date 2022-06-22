pub mod programs;

pub use paste::paste;
pub use programs::*;

pub mod macro_imports {
    pub use crate::{match_pubkey, RegistryError};
    pub use anchor_lang::prelude::{declare_id, Id, ProgramError, Pubkey};
    pub use std::convert::TryFrom;
}

#[anchor_lang::error_code]
pub enum RegistryError {
    #[msg("program id is not associated with a registered program (static-program-registry)")]
    UnknownProgramId,
}

/// Declares a program ID for a module.
/// Makes a struct that can be used as a program account in anchor.
#[macro_export]
macro_rules! program {
    ($Name:ident, $id:literal) => {
        $crate::macro_imports::declare_id!($id);

        #[derive(Copy, Clone)]
        pub struct $Name;

        impl $crate::macro_imports::Id for $Name {
            fn id() -> $crate::macro_imports::Pubkey {
                ID
            }
        }
    };
}

/// Rust tries to destruct the Pubkey tuple struct, which is not allowed due to privacy.
/// you need something like this to bypass that compiler logic.
#[macro_export]
macro_rules! match_pubkey {
    (($item:expr) {
        $($possibility:expr => $blk:expr,)*
        _ => $default:expr,
    }) => {{
        let evaluated = $item;
        $(if evaluated == $possibility {
            $blk
        } else)* {
            $default
        }
    }};
}

/// Creates an enum that implements TryFrom<Pubkey> with a variant for each program
/// Creates `use_*_client` macros, see docs in implementation.
/// Use labelled square brackets to sub-group programs based on client implementations.
///
/// This creates one SwapProgram enum and one `use_client` macro:
/// ```
/// related_programs! {
///     SwapProgram {[
///         spl_token_swap_v2::Spl2,
///         orca_swap_v1::OrcaV1,
///         orca_swap_v2::OrcaV2,
///     ]}
/// }
/// ```
///
/// This creates one SwapProgram enum, plus `use_orca_client` and `use_spl_client` macros:
/// ```
/// related_programs! {
///     SwapProgram {
///         spl [spl_token_swap_v2::Spl2]
///         orca [
///             orca_swap_v1::OrcaV1,
///             orca_swap_v2::OrcaV2,
///         ]
///     }
/// }
/// ```
#[macro_export]
macro_rules! related_programs {
    ($Name:ident {
        $($($client_group_name:ident)? [
            $($module:ident::$Variant:ident),+$(,)?
        ])+
    }) => {
        #[derive(PartialEq, Eq, Debug)]
        pub enum $Name {
            $($($Variant),+),+
        }

        const _: () = {
            use super::*;
            use $crate::macro_imports::*;
            $($(use $module::{$Variant};)+)+

            impl TryFrom<Pubkey> for $Name {
                type Error = RegistryError;

                fn try_from(value: Pubkey) -> std::result::Result<Self, Self::Error> {
                    match_pubkey! { (value) {
                        $($($Variant::id() => Ok($Name::$Variant)),+),+,
                        _ => Err(RegistryError::UnknownProgramId),
                    }}
                }
            }
        };

        $($crate::paste! {
            /// If all programs within a [] share identical syntax in their client libraries,
            /// use this macro to conditionally access the crate for the given program_id
            /// ```
            /// let swap_ix = use_client!(program_id {
            ///    client::instruction::swap(...)
            /// }?;
            /// ```
            #[allow(unused)]
            macro_rules! [<use_ $($client_group_name _)? client>] {
                ($program_id:expr, $blk:block) => {{
                    use anchor_lang::prelude::{Id, msg};
                    use $crate::RegistryError;
                    $(use $module::{$Variant};)+
                    $crate::macro_imports::match_pubkey! { ($program_id) {
                        $($Variant::id() => Ok({
                            use $module as client;
                            $blk
                        })),+,
                        _ => {
                            msg!("program id {} not registered", $program_id);
                            Err(RegistryError::UnknownProgramId)
                        },
                    }}
                }};
            }
            pub(crate) use [<use_ $($client_group_name _)? client>];
        })+
    };
}

#[cfg(test)]
mod test {
    use anchor_lang::prelude::Pubkey;

    use crate::programs::*;

    related_programs! {
        SwapProgram {[
            spl_token_swap_v2::Spl2,
            orca_swap_v1::OrcaV1,
            orca_swap_v2::OrcaV2,
        ]}
    }

    related_programs! {
        SwapProgram2 {
            spl[
                spl_token_swap_v2::Spl2,
            ]
            orca[
                orca_swap_v1::OrcaV1,
                orca_swap_v2::OrcaV2,
            ]
        }
    }

    #[test]
    fn conversions_work() {
        assert_eq!(SwapProgram::Spl2, spl_token_swap_v2::ID.try_into().unwrap());
        assert_eq!(SwapProgram::OrcaV1, orca_swap_v1::ID.try_into().unwrap());
        assert_eq!(SwapProgram::OrcaV2, orca_swap_v2::ID.try_into().unwrap());
        assert_eq!(
            SwapProgram2::Spl2,
            spl_token_swap_v2::ID.try_into().unwrap()
        );
        assert_eq!(SwapProgram2::OrcaV1, orca_swap_v1::ID.try_into().unwrap());
        assert_eq!(SwapProgram2::OrcaV2, orca_swap_v2::ID.try_into().unwrap());
    }

    #[test]
    fn bad_conversions_dont_work() {
        SwapProgram::try_from(Pubkey::default()).unwrap_err();
        SwapProgram2::try_from(Pubkey::default()).unwrap_err();
    }

    #[test]
    fn use_client_works() {
        for id in &[spl_token_swap_v2::ID, orca_swap_v1::ID, orca_swap_v2::ID] {
            assert_eq!(*id, use_client!(*id, { client::id() }).unwrap());
        }
        assert_eq!(
            spl_token_swap_v2::ID,
            use_spl_client!(spl_token_swap_v2::ID, { client::id() }).unwrap()
        );
        for id in &[orca_swap_v1::ID, orca_swap_v2::ID] {
            assert_eq!(*id, use_orca_client!(*id, { client::id() }).unwrap());
        }
    }

    #[test]
    fn use_client_errors_when_expected() {
        use_client!(Pubkey::default(), { client::id() }).unwrap_err();
        use_orca_client!(spl_token_swap_v2::ID, { client::id() }).unwrap_err();
        for id in &[orca_swap_v1::ID, orca_swap_v2::ID] {
            use_spl_client!(*id, { client::id() }).unwrap_err();
        }
    }
}
