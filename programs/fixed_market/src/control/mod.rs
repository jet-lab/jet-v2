//! Control module for the `jet-market` program.
//!
//! Handles initialization of program state including the [`MarketManager`](struct@crate::control::state::MarketManager) and program authority signer

/// Program instructions for the control module
pub mod instructions;
/// State utilities and structs for the control module
pub mod state;

/// Anchor events
pub(crate) mod events;
