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

use anchor_lang::{InstructionData, ToAccountMetas};
use jet_margin_pool::MarginPoolConfig;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, system_program};

use super::get_metadata_address;
use super::margin_pool::MarginPoolIxBuilder;

pub use jet_control::TokenMetadataParams;

pub use jet_control::ID as CONTROL_PROGRAM;

/// A builder for [`jet_control::instruction`] instructions.
pub struct ControlIxBuilder {
    /// The user address that will pay for the transactions
    payer: Pubkey,

    /// The address with authority to request changes
    requester: Pubkey,
}

impl ControlIxBuilder {
    /// Create a new instruction builder
    pub fn new(payer: Pubkey) -> Self {
        Self {
            payer,
            requester: payer,
        }
    }

    /// Create a new builder with a different authority to request changes
    pub fn new_for_authority(authority: Pubkey, payer: Pubkey) -> Self {
        Self {
            payer,
            requester: authority,
        }
    }

    /// [Instruction] to create a new authority
    pub fn create_authority(&self) -> Instruction {
        let accounts = jet_control::accounts::CreateAuthority {
            payer: self.payer,
            authority: get_control_authority_address(),
            system_program: system_program::ID,
        }
        .to_account_metas(None);

        Instruction {
            accounts,
            program_id: jet_control::ID,
            data: jet_control::instruction::CreateAuthority {}.data(),
        }
    }

    /// Instruction to register a margin adapter with the control program.
    ///
    /// An adapter must be registered with the control program before users can
    /// interact with it.
    pub fn register_adapter(&self, adapter: &Pubkey) -> Instruction {
        let accounts = jet_control::accounts::RegisterAdapter {
            requester: self.requester,
            authority: get_control_authority_address(),

            adapter: *adapter,
            metadata_account: get_metadata_address(adapter),

            payer: self.payer,

            metadata_program: jet_metadata::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None);

        Instruction {
            accounts,
            program_id: jet_control::ID,
            data: jet_control::instruction::RegisterAdapter {}.data(),
        }
    }

    /// Instruction to register a margin pool.
    ///
    /// The margin pool is created with default settings, and must be configured
    /// with `configure_margin_pool`
    pub fn create_margin_pool(&self, token: &Pubkey) -> Instruction {
        let pool_builder = MarginPoolIxBuilder::new(*token);
        let accounts = jet_control::accounts::CreateMarginPool {
            requester: self.requester,
            payer: self.payer,
            authority: get_control_authority_address(),

            margin_pool: pool_builder.address,
            vault: pool_builder.vault,
            deposit_note_mint: pool_builder.deposit_note_mint,
            loan_note_mint: pool_builder.loan_note_mint,
            token_mint: *token,
            deposit_note_metadata: get_metadata_address(&pool_builder.deposit_note_mint),
            loan_note_metadata: get_metadata_address(&pool_builder.loan_note_mint),
            token_metadata: get_metadata_address(&pool_builder.token_mint),
            fee_destination: get_margin_pool_fee_destination_address(&pool_builder.address),

            margin_pool_program: jet_margin_pool::ID,
            metadata_program: jet_metadata::ID,
            token_program: spl_token::ID,
            system_program: system_program::ID,
            rent: solana_sdk::sysvar::rent::ID,
        }
        .to_account_metas(None);

        Instruction {
            accounts,
            program_id: jet_control::ID,
            data: jet_control::instruction::CreateMarginPool {}.data(),
        }
    }

    /// Instruction to configure a margin pool.
    ///
    /// Configuration can update various parameters, enable or disable borrowing,
    /// etc. See [MarginPoolConfiguration] for all parameters.
    pub fn configure_margin_pool(
        &self,
        token: &Pubkey,
        config: &MarginPoolConfiguration,
    ) -> Instruction {
        let pool_builder = MarginPoolIxBuilder::new(*token);
        let accounts = jet_control::accounts::ConfigureMarginPool {
            requester: self.requester,
            authority: get_control_authority_address(),

            token_mint: *token,
            margin_pool: pool_builder.address,
            token_metadata: get_metadata_address(token),
            deposit_metadata: get_metadata_address(&pool_builder.deposit_note_mint),
            loan_metadata: get_metadata_address(&pool_builder.loan_note_mint),

            pyth_product: config.pyth_product.unwrap_or_default(),
            pyth_price: config.pyth_price.unwrap_or_default(),

            margin_pool_program: jet_margin_pool::ID,
            metadata_program: jet_metadata::ID,
        }
        .to_account_metas(None);

        Instruction {
            accounts,
            program_id: jet_control::ID,
            data: jet_control::instruction::ConfigureMarginPool {
                metadata: config.metadata.clone(),
                pool_config: config.parameters,
            }
            .data(),
        }
    }

    /// Instruction to enable or disable a liquidator.
    ///
    /// Only authorised accounts are allowed to liquidate margin accounts.
    pub fn set_liquidator(&self, liquidator: &Pubkey, is_liquidator: bool) -> Instruction {
        let accounts = jet_control::accounts::SetLiquidator {
            requester: self.requester,
            authority: get_control_authority_address(),

            liquidator: *liquidator,
            metadata_account: get_metadata_address(liquidator),

            payer: self.payer,

            metadata_program: jet_metadata::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None);

        Instruction {
            accounts,
            program_id: jet_control::ID,
            data: jet_control::instruction::SetLiquidator { is_liquidator }.data(),
        }
    }
}

/// Parameters used to configer a margin pool
#[derive(Clone, Default)]
pub struct MarginPoolConfiguration {
    /// The optional address of the Pyth product
    pub pyth_product: Option<Pubkey>,
    /// The optional address of the Pyth price
    pub pyth_price: Option<Pubkey>,

    /// Optional configuration of the pool
    pub parameters: Option<MarginPoolConfig>,
    /// Optional metadata of the pool, includes collateral weight and risk multiplier
    pub metadata: Option<TokenMetadataParams>,
}

/// Get the address of the control authority
pub fn get_control_authority_address() -> Pubkey {
    Pubkey::find_program_address(&[], &jet_control::ID).0
}

fn get_margin_pool_fee_destination_address(pool: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[jet_control::seeds::FEE_DESTINATION, pool.as_ref()],
        &jet_control::ID,
    )
    .0
}
