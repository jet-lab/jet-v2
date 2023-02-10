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

//! Interact with the lookup table program, generate lookups into tables.

#![cfg_attr(not(feature = "localnet"), allow(unused))]

use std::{collections::HashSet, sync::Arc, time::Duration};

use anchor_client::solana_client::rpc_config::RpcSendTransactionConfig;
use anyhow::{bail, Context, Result};
use jet_simulation::solana_rpc_api::{RpcConnection, SolanaRpcClient};
use solana_address_lookup_table_program::state::{AddressLookupTable, LookupTableMeta};
use solana_sdk::{
    account::ReadableAccount,
    address_lookup_table_account::AddressLookupTableAccount,
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    message::v0,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::VersionedTransaction,
};
use solana_transaction_status::UiTransactionEncoding;

/// Lookup tables are used to interact with the lookup table program
///
/// Note: this structure is experimental, and is subject to change
pub struct LookupTable;

impl LookupTable {
    /// Get the contents of a lookup table
    pub async fn get_lookup_table<'a>(
        rpc: &Arc<dyn SolanaRpcClient>,
        address: &Pubkey,
    ) -> Result<Option<AddressLookupTableAccount>> {
        let account = rpc.get_account(address).await?;
        let table = account
            .map(|acc| match AddressLookupTable::deserialize(acc.data()) {
                Ok(table) => Ok(AddressLookupTableAccount {
                    key: *address,
                    addresses: table.addresses.to_vec(),
                }),
                Err(e) => {
                    bail!("Error deserializing lookup table {:?}", e)
                }
            })
            .transpose()?;
        Ok(table)
    }
    /// Create a new lookup table and return its address
    pub async fn create_lookup_table(
        rpc: &Arc<dyn SolanaRpcClient>,
        authority: Option<Pubkey>,
    ) -> Result<Pubkey> {
        let recent_slot = rpc.get_slot(Some(CommitmentConfig::finalized())).await?;
        let authority = authority.unwrap_or_else(|| rpc.payer().pubkey());
        let (create_ix, table_address) =
            solana_address_lookup_table_program::instruction::create_lookup_table(
                authority,
                rpc.payer().pubkey(),
                recent_slot,
            );

        let tx = rpc.create_transaction(&[], &[create_ix]).await?;

        rpc.send_and_confirm_transaction(&tx).await?;

        Ok(table_address)
    }

    /// Extend a lookup table by adding more accounts.
    /// First checks if an account already exists, and excludes it if it does,
    /// as lookup tables can have duplicate accounts and waste space.
    pub async fn extend_lookup_table(
        rpc: &Arc<dyn SolanaRpcClient>,
        table_address: Pubkey,
        authority: Option<Pubkey>,
        accounts: &[Pubkey],
    ) -> Result<()> {
        if accounts.is_empty() {
            bail!("Cannot extend lookup table if there are no accounts to add")
        }
        // Get the lookup table
        let table = Self::get_lookup_table(rpc, &table_address)
            .await?
            .context("Table not found")?;
        let existing_accounts = table.addresses.iter().cloned().collect::<HashSet<_>>();
        let accounts_to_add = accounts
            .iter()
            .filter(|a| !existing_accounts.contains(a))
            .cloned()
            .collect::<Vec<_>>();
        if accounts_to_add.is_empty() {
            bail!("All the accounts being added already exist, preventing adding duplicates")
        }
        // Keep track of the last signature
        let authority = authority.unwrap_or_else(|| rpc.payer().pubkey());
        let payer = rpc.payer().pubkey();
        let mut signature = Signature::default();
        for pubkeys in accounts_to_add.chunks(20) {
            let ix = solana_address_lookup_table_program::instruction::extend_lookup_table(
                table_address,
                authority,
                Some(payer),
                pubkeys.to_vec(),
            );

            let tx = rpc.create_transaction(&[], &[ix]).await?;

            signature = rpc.send_and_confirm_transaction(&tx).await?;
        }

        // Trying to use the lookup table immediately doesn't work
        tokio::time::sleep(Duration::from_secs(10)).await;

        Ok(())
    }

    /// Use a lookup table to create a [VersionedTransaction]
    pub async fn use_lookup_tables(
        rpc: &Arc<dyn SolanaRpcClient>,
        table_addresses: &[Pubkey],
        instructions: &[Instruction],
        keypairs: &[&Keypair],
    ) -> Result<VersionedTransaction> {
        let mut tables = vec![];
        for address in table_addresses {
            if let Ok(Some(table)) = Self::get_lookup_table(rpc, address).await {
                tables.push(table)
            }
        }

        let mut signers = vec![rpc.payer()];
        signers.extend_from_slice(keypairs);

        let blockhash = rpc.get_latest_blockhash().await?;
        let tx = VersionedTransaction::try_new(
            solana_sdk::message::VersionedMessage::V0(v0::Message::try_compile(
                &rpc.payer().pubkey(),
                instructions,
                &tables,
                blockhash,
            )?),
            &signers,
        )?;

        Ok(tx)
    }

    /// Send a versioned transaction and wait for its confirmation
    ///
    /// We are using this until we can update to solana 1.14 which supports this with `SerializedTransaction`
    pub async fn send_versioned_transaction(
        rpc: &Arc<dyn SolanaRpcClient>,
        transaction: &VersionedTransaction,
    ) -> Result<Signature> {
        let serialized = bincode::serialize(transaction)?;
        let encoded = base64::encode(serialized);

        let connection = rpc
            .as_any()
            .downcast_ref::<RpcConnection>()
            .context("rpc is not an RpcConnection")?;
        let config = connection
            .tx_config()
            .cloned()
            .unwrap_or_else(|| RpcSendTransactionConfig {
                skip_preflight: true,
                preflight_commitment: None,
                encoding: Some(UiTransactionEncoding::Base64),
                ..Default::default()
            });
        let signature = connection
            .client()
            .send_transaction_with_config(transaction, config)
            .await?;

        Ok(signature)
    }
}
