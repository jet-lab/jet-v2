//! Control module for the `jet-bonds` program.
//!
//! Handles initialization of program state including the [`BondManager`](struct@crate::control::state::BondManager) and program authority signer

/// Program instructions for the control module
pub mod instructions;
/// State utilities and structs for the control module
pub mod state;

/// Anchor events
pub mod events;
