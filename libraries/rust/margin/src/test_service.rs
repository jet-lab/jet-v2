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

use solana_sdk::pubkey::Pubkey;

use jet_instructions::{
    control::ControlIxBuilder, get_metadata_address, test_service::if_not_initialized,
};

use crate::{
    cat, solana::transaction::TransactionBuilder, tx_builder::global_initialize_instructions,
};

static ADAPTERS: &[Pubkey] = &[jet_margin_pool::ID, jet_margin_swap::ID, jet_fixed_term::ID];

/// Basic environment setup for hosted tests that has only the necessary global
/// state initialized
pub fn minimal_environment(authority: Pubkey) -> Vec<TransactionBuilder> {
    cat![
        global_initialize_instructions(authority),
        create_global_adapter_register_tx(authority),
    ]
}

fn create_global_adapter_register_tx(authority: Pubkey) -> Vec<TransactionBuilder> {
    let ctrl_ix = ControlIxBuilder::new(authority);
    ADAPTERS
        .iter()
        .map(|a| if_not_initialized(get_metadata_address(a), ctrl_ix.register_adapter(a)).into())
        .collect()
}
