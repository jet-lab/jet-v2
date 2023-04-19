/// Declare a custom logging target hierarchy, where the modules may use
/// different names from the log targets.
macro_rules! declare_logging {
    ($parent_mod:ident = $parent_target:literal {
        $($mod:ident = $target:literal);+$(;)?
    }) => {
        mod $parent_mod {
            $(pub mod $mod {
                use super::super::declare_logging;
                declare_logging!(target: concat!($parent_target, "::", $target));
            })+
        }
    };
    (target: $target:expr) => {
        declare_logging!(($) trace as trace, $target);
        declare_logging!(($) debug as debug, $target);
        declare_logging!(($) info as info, $target);
        declare_logging!(($) _warn as warn, $target);
        declare_logging!(($) error as error, $target);
    };
    (($dollar:tt) $internal_name:ident as $log_macro:ident, $target:expr) => {
        #[allow(unused_macros)]
        macro_rules! $internal_name {
            ($dollar($arg:tt)+) => {
                ::log::$log_macro!(target: $target, $dollar($arg)*)
            };
        }
        #[allow(unused_imports)]
        pub(crate) use $internal_name as $log_macro;
    };
}
pub(crate)use declare_logging;
