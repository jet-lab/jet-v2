#[cfg(any(test, feature = "debug-msg"))]
pub mod _internal {
    pub use solana_program::log::sol_log;

    /// Sends messages to the solana logs, only if debug-msg feature is enabled
    #[macro_export]
    macro_rules! debug_msg {
        ($msg:literal) => {
            $crate::log::_internal::sol_log(concat!("debug: ", $msg))
        };
        ($msg:literal, $($arg:tt)*) => {
            $crate::log::_internal::sol_log(&format!(concat!("debug: ", $msg), $($arg)*))
        };
    }
}

/// Sends messages to the solana logs, only if debug-msg feature is enabled
#[cfg(not(any(test, feature = "debug-msg")))]
#[macro_export]
macro_rules! debug_msg {
    ($msg:literal $(, $($arg:tt)*)?) => {};
}
