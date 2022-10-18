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

use std::sync::Arc;

use anchor_client::solana_client::rpc_config::RpcSendTransactionConfig;
use anyhow::{bail, Context, Result};
use jet_simulation::solana_rpc_api::{RpcConnection, SolanaRpcClient};
use solana_address_lookup_table_program::state::AddressLookupTable;
use solana_sdk::{
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

/// TODO
pub async fn create_lookup_table(
    rpc: &Arc<dyn SolanaRpcClient>,
    // TODO: think more about how we want to handle authority, as they need to sign
    // authority: Option<Pubkey>,
) -> Result<Pubkey> {
    let recent_slot = rpc.get_slot(Some(CommitmentConfig::finalized())).await?;
    // let authority = authority.unwrap_or_else(|| rpc.payer().pubkey());
    let authority = rpc.payer().pubkey();
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

/// TODO add authority
pub async fn extend_lookup_table(
    rpc: &Arc<dyn SolanaRpcClient>,
    table_address: Pubkey,
    accounts: &[Pubkey],
) -> Result<()> {
    if accounts.is_empty() {
        bail!("Cannot extend lookup table if there are no accounts to add")
    }
    // Keep track of the last signature
    let authority = rpc.payer().pubkey();
    let payer = rpc.payer().pubkey();
    let mut signature = Signature::default();
    for pubkeys in accounts.chunks(20) {
        let ix = solana_address_lookup_table_program::instruction::extend_lookup_table(
            table_address,
            authority,
            Some(payer),
            pubkeys.to_vec(),
        );

        let tx = rpc.create_transaction(&[], &[ix]).await?;

        signature = rpc.send_and_confirm_transaction(&tx).await?;
    }

    // Check that the last signature is confirmed
    rpc.confirm_transactions(&[signature]).await?;

    Ok(())
}

/// Use a lookup table
///
/// TODO assumes the payer is not different, change if we find that it's so
pub async fn use_lookup_table(
    rpc: &Arc<dyn SolanaRpcClient>,
    table_address: Pubkey,
    instructions: &[Instruction],
    keypairs: &[&Keypair],
) -> Result<VersionedTransaction> {
    let table = rpc
        .get_account(&table_address)
        .await?
        .with_context(|| format!("Address {table_address} could not be found"))?;
    let table = AddressLookupTable::deserialize(&table.data)?;
    let lookup_table_account = AddressLookupTableAccount {
        key: table_address,
        addresses: table.addresses.to_vec(),
    };

    let mut signers = vec![rpc.payer()];
    signers.extend_from_slice(keypairs);

    let blockhash = rpc.get_latest_blockhash().await?;
    let tx = VersionedTransaction::try_new(
        solana_sdk::message::VersionedMessage::V0(v0::Message::try_compile(
            &rpc.payer().pubkey(),
            instructions,
            &[lookup_table_account],
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
) -> Result<()> {
    let serialized = bincode::serialize(transaction)?;
    let encoded = base64::encode(serialized);

    let connection = rpc
        .as_any()
        .downcast_ref::<RpcConnection>()
        .context("rpc is not an RpcConnection")?;
    // TODO: inherit the config of the client
    let config = RpcSendTransactionConfig {
        skip_preflight: true,
        preflight_commitment: None,
        encoding: Some(UiTransactionEncoding::Base64),
        ..Default::default()
    };
    let signature = connection
        .get_client()
        .send::<String>(
            solana_client::rpc_request::RpcRequest::SendTransaction,
            serde_json::json!([encoded, config]),
        )
        .await?;
    rpc.confirm_transactions(&[signature.parse::<Signature>()?])
        .await?;

    Ok(())
}
