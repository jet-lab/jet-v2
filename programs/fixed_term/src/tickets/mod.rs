//! # Market Tickets
//!
//! The program abstracts the concept of a fixed rate and fixed term into market tickets. Market tickets are fungible spl tokens
//! that must be staked to claim their underlying value. In order to create market tickets, a user must either
//! place a lend order on the orderbook, or exchange the token underlying the market market (in practice, almost never
//! will users do this, as it locks their tokens for at least the tenor of the market).
//!
//! ## Ticket kinds
//!
//! The program allots for two types of ticket redemption. The [`ClaimTicket`](struct@crate::tickets::state::ClaimTicket) is given when
//! a user stakes directly with the program. As there is no information about the creation of the tickets, a `ClaimTicket` does not
//! have accounting for principal or interest, and only contains a redemptive value.
//!
//! Conversely, a [`SplitTicket`](struct@crate::tickets::state::SplitTicket) contains split principal and interest values. As well as
//! the slot it was minted. To create a `SplitTicket`, you must configure your [`OrderParams`](struct@crate::orderbook::state::OrderParams) `auto_stake` flag to
//! `true`. This will allow to program to immediately stake your tickets as the match event is processed.
//!

/// Program instructions for using market tickets
pub mod instructions;
/// Methods and structs for defining market tickets
pub mod state;

/// Anchor events
pub(crate) mod events;
