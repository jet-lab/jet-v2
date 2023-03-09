//! Instruction Constructors for the Fixed Term program
//!
//! This module only has simple functions with explicit, bare minimum
//! dependencies, that act only as a thin wrapper around anchor or solana
//! instruction constructors. This abstracts away any guaranteed information
//! about addresses, like the logic to derive PDAs, without making assumptions
//! about what might be "typical".
//!
//! The goal of this code is to enable any fixed term instruction to be invoked
//! as easily as possible without needing to construct an entire
//! FixedTermIxBuilder, since some of the fields of that struct may be difficult
//! to populate depending on the situation. But FixedTermIxBuilder also remains
//! as a standard approach to invoke these instructions when there is no
//! difficulty in accessing every address for a particular market.
//!
//! These functions should:
//! - be named after an instruction in the crate.
//! - simply construct and return an Instruction for a specific instruction in
//!   the program.
//! - derive pdas that can be derived from other inputs.
//! - Facilitate construction of the Instruction struct.
//! - Put data parameters before pubkeys
//!
//! These functions should NOT:
//! - be methods of a struct.
//! - depend on unnecessary data. Do not require complex structs with fields
//!   that are not used by the function. Only use a struct of pubkeys as a param
//!   if EVERY pubkey is used by that function.
//! - have any side effects such as mutations or sending requests.
//! - be async.
//! - make any assumptions, for example setting the payer == crank.
//! - depend on any pubkeys that can always be derived from other inputs.

mod admin;
mod crank;
mod margin_user;
mod user;

pub use admin::*;
pub use crank::*;
pub use margin_user::*;
pub use user::*;
