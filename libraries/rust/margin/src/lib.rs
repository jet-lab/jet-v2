// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright (C) 2022 JET PROTOCOL HOLDINGS, LLC.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Jet Margin SDK
//!
//! This crate is the official Rust SDK for the Jet Margin family of programs.
//! It includes instruction and transaction builders that allow users of our
//! programs to conveniently interact with them.
//!
//! The SDK currently supports the following programs and adapters:
//! * Control program - mostly used internally for configuration
//! * Margin - create, manage and interact with [jet_margin::MarginAccount]s
//! * Margin Pool - an adapter for borrowing and lending in our pools
//! * Margin Swap - execute swaps via `spl_token_sawp` compatible programs, e.g. Orca.
//!
//! A good starting point for using the SDK is to create a margin account.
//!
//! ```ignore
//! use std::sync::Arc;
//!
//! use jet_simulation::solana_rpc_api::{RpcConnection, SolanaRpcClient};
//! use solana_client::rpc_client::nonblocking::RpcClient;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!   // Create an RPC connection
//!   let client = RpcClient::new("https://my-endpoint.com");
//!   let rpc = RpcConnection::new(payer, client);
//!   // Create a transaction builder
//!   let tx_builder = jet_margin_sdk::tx_builder::MarginTxBuilder::new(&rpc, ...);
//!   // Create a transaction to register a margin account
//!   let tx = tx_builder.create_account().await?;
//!   // Submit transaction
//!   rpc.send_and_confirm_transaction(&tx).await?;
//! }
//! ```

#![deny(missing_docs)]

/// Instruction builders for programs and adapters supported by the SDK
pub mod ix_builder;
/// generic code to integrate adapters with margin
pub mod margin_integrator;
/// things that should be provided by the solana sdk, but are not
pub mod solana;
/// Utilities for swap adapters
pub mod swap;
/// Utilities for tokens and token prices
pub mod tokens;
/// Transaction builder
pub mod tx_builder;
/// General purpose logic used by this lib and clients, unrelated to jet or solana
pub mod util;

/// jet-bonds sdk
pub mod bonds;

/// Utilities for test environments
pub mod test_service;

pub use jet_airspace;
pub use jet_bonds;
pub use jet_control;
pub use jet_margin;
pub use jet_margin_pool;
pub use jet_margin_swap;
pub use jet_metadata;
pub use jet_test_service;
