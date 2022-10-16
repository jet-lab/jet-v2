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

use anyhow::{Result, bail, Context};
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_address_lookup_table_program::state::AddressLookupTable;
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signer::Signer, signature::Signature, transaction::VersionedTransaction, instruction::Instruction, message::v0
};

/// TODO
pub async fn create_lookup_table(
    rpc: &Arc<dyn SolanaRpcClient>,
    // TODO: think more about how we want to handle authority, as they need to sign
    // authority: Option<Pubkey>,
) -> Result<Pubkey> {
    let recent_slot = rpc.get_slot(Some(CommitmentConfig::finalized())).await?;
    // let authority = authority.unwrap_or_else(|| rpc.payer().pubkey());
    let authority = rpc.payer().pubkey();
    let (create_ix, table_address) = solana_address_lookup_table_program::instruction::create_lookup_table(authority, rpc.payer().pubkey(), recent_slot);

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
            table_address, authority, Some(payer), pubkeys.to_vec()
        );

        let tx = rpc.create_transaction(&[], &[ix]).await?;

        signature = rpc.send_and_confirm_transaction(&tx).await?;
    }

    // Check that the last signature is confirmed
    rpc.confirm_transactions(&[signature]).await?;

    Ok(())
}
