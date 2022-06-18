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

use jet_margin_pool::MarginPoolConfig;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_program::ID as SYSTEM_PROGRAM_ID;

use anchor_lang::{InstructionData, ToAccountMetas};

use jet_control::{MarginPoolParams, TokenMetadataParams};

use super::metadata::get_metadata_address;
use crate::accounts::MarginPoolAccounts;

pub fn create_authority(payer: Pubkey) -> Instruction {
    let accounts = jet_control::accounts::CreateAuthority {
        payer,
        authority: get_authority_address(),
        system_program: SYSTEM_PROGRAM_ID,
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        program_id: jet_control::ID,
        data: jet_control::instruction::CreateAuthority {}.data(),
    }
}

pub fn register_token(pool: &MarginPoolAccounts, requester: &Pubkey) -> Instruction {
    let accounts = jet_control::accounts::RegisterToken {
        requester: *requester,
        authority: get_authority_address(),

        margin_pool: pool.address,
        vault: pool.vault,
        deposit_note_mint: pool.deposit_note_mint,
        loan_note_mint: pool.loan_note_mint,
        token_mint: pool.token_mint,
        deposit_note_metadata: get_metadata_address(&pool.deposit_note_mint),
        loan_note_metadata: get_metadata_address(&pool.loan_note_mint),
        token_metadata: get_metadata_address(&pool.token_mint),

        margin_pool_program: jet_margin_pool::ID,
        metadata_program: jet_metadata::ID,
        token_program: anchor_spl::token::ID,
        system_program: SYSTEM_PROGRAM_ID,
        rent: solana_sdk::sysvar::rent::ID,
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        program_id: jet_control::ID,
        data: jet_control::instruction::RegisterToken {}.data(),
    }
}

pub fn register_adapter(adapter: &Pubkey, authority: &Pubkey, payer: &Pubkey) -> Instruction {
    let accounts = jet_control::accounts::RegisterAdapter {
        requester: *authority,
        authority: get_authority_address(),

        adapter: *adapter,
        metadata_account: get_metadata_address(adapter),

        payer: *payer,

        metadata_program: jet_metadata::ID,
        system_program: SYSTEM_PROGRAM_ID,
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        program_id: jet_control::ID,
        data: jet_control::instruction::RegisterAdapter {}.data(),
    }
}

#[derive(Clone, Default)]
pub struct TokenConfiguration {
    pub pyth_product: Option<Pubkey>,
    pub pyth_price: Option<Pubkey>,

    pub pool_config: Option<MarginPoolConfig>,
    pub pool_params: Option<MarginPoolParams>,
    pub metadata: Option<TokenMetadataParams>,
}

pub fn configure_token(
    pool: &MarginPoolAccounts,
    authority: &Pubkey,
    config: &TokenConfiguration,
) -> Instruction {
    let accounts = jet_control::accounts::ConfigureToken {
        requester: *authority,
        authority: get_authority_address(),

        token_mint: pool.token_mint,
        margin_pool: pool.address,
        token_metadata: get_metadata_address(&pool.token_mint),
        deposit_metadata: get_metadata_address(&pool.deposit_note_mint),

        pyth_product: config.pyth_product.unwrap_or_default(),
        pyth_price: config.pyth_price.unwrap_or_default(),

        margin_pool_program: jet_margin_pool::ID,
        metadata_program: jet_metadata::ID,
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        program_id: jet_control::ID,
        data: jet_control::instruction::ConfigureToken {
            metadata: config.metadata.clone(),
            pool_param: config.pool_params.clone(),
            pool_config: config.pool_config.clone(),
        }
        .data(),
    }
}

pub fn get_authority_address() -> Pubkey {
    Pubkey::find_program_address(&[], &jet_control::ID).0
}

pub fn set_liquidator(
    authority: &Pubkey,
    payer: &Pubkey,
    liquidator: &Pubkey,
    is_liquidator: bool,
) -> Instruction {
    let accounts = jet_control::accounts::SetLiquidator {
        requester: *authority,
        authority: get_authority_address(),

        liquidator: *liquidator,
        metadata_account: get_metadata_address(liquidator),

        payer: *payer,

        metadata_program: jet_metadata::ID,
        system_program: SYSTEM_PROGRAM_ID,
    }
    .to_account_metas(None);

    Instruction {
        accounts,
        program_id: jet_control::ID,
        data: jet_control::instruction::SetLiquidator { is_liquidator }.data(),
    }
}
