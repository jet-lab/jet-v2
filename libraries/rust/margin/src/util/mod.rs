/// simplify parallel execution of generic tasks
pub mod asynchronous;
/// generic processing of arbitrary data
pub mod data;
/// non-blocking communication between threads through a queue that prevents
/// message duplication.
pub mod no_dupe_queue;

/// Produce a trait that you can use in the current module to seal other traits.
/// Sealing a trait means it can only be implemented for the provided types.
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
