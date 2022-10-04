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

use anchor_spl::associated_token::{self, get_associated_token_address};
use jet_margin::seeds::{ADAPTER_CONFIG_SEED, LIQUIDATOR_CONFIG_SEED, TOKEN_CONFIG_SEED};
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_program::ID as SYSTEM_PROGAM_ID;
use solana_sdk::sysvar::{rent::Rent, SysvarId};

use anchor_lang::prelude::{Id, System, ToAccountMetas};
use anchor_lang::{system_program, InstructionData};
use anchor_spl::token::Token;

use jet_margin::instruction as ix_data;
use jet_margin::program::JetMargin;
use jet_margin::{accounts as ix_account, TokenConfigUpdate};

/// Utility for creating instructions to interact with the margin
/// program for a specific account.
#[derive(Clone)]
pub struct MarginIxBuilder {
    /// The account owner,
    pub owner: Pubkey,

    /// The account seed
    pub seed: u16,

    /// The account paying for any rent
    pub payer: Pubkey,

    /// The address of the margin account for the owner
    pub address: Pubkey,

    /// The address of the airspace the margin account belongs to
    pub airspace: Pubkey,

    /// The authority to use in place of the owner
    authority: Option<Pubkey>,
}

impl MarginIxBuilder {
    /// Create a new [MarginIxBuilder] which uses the margin account as the authority.
    /// Ordinary margin users should use this function to create a builder.
    pub fn new(owner: Pubkey, seed: u16) -> Self {
        Self::new_with_payer(owner, seed, owner, None)
    }

    /// Create a new [MarginIxBuilder] with a custom payer and authority.
    /// The authority is expected to sign the instructions generated, and
    /// is normally the margin account or its registered liquidator.
    /// If the authority is not set, it defaults to the margin account.
    pub fn new_with_payer(
        owner: Pubkey,
        seed: u16,
        payer: Pubkey,
        authority: Option<Pubkey>,
    ) -> Self {
        Self::new_with_payer_and_airspace(owner, seed, payer, Pubkey::default(), authority)
    }

    /// Create a new [MarginIxBuilder] with a custom payer and authority.
    /// The authority is expected to sign the instructions generated, and
    /// is normally the margin account or its registered liquidator.
    /// If the authority is not set, it defaults to the margin account.
    pub fn new_with_payer_and_airspace(
        owner: Pubkey,
        seed: u16,
        payer: Pubkey,
        airspace: Pubkey,
        authority: Option<Pubkey>,
    ) -> Self {
        let (address, _) = Pubkey::find_program_address(
            &[owner.as_ref(), seed.to_le_bytes().as_ref()],
            &jet_margin::ID,
        );
        Self {
            owner,
            seed,
            payer,
            address,
            authority,
            airspace,
        }
    }

    /// Get instruction to create the account
    pub fn create_account(&self) -> Instruction {
        let accounts = ix_account::CreateAccount {
            owner: self.owner,
            payer: self.payer,
            margin_account: self.address,
            system_program: SYSTEM_PROGAM_ID,
        };

        Instruction {
            program_id: JetMargin::id(),
            data: ix_data::CreateAccount { seed: self.seed }.data(),
            accounts: accounts.to_account_metas(None),
        }
    }

    /// Get instruction to close account
    pub fn close_account(&self) -> Instruction {
        let accounts = ix_account::CloseAccount {
            owner: self.owner,
            receiver: self.payer,
            margin_account: self.address,
        };

        Instruction {
            program_id: JetMargin::id(),
            data: ix_data::CloseAccount.data(),
            accounts: accounts.to_account_metas(None),
        }
    }

    /// Get instruction to update the accounting for assets in
    /// the custody of the margin account.
    ///
    /// # Params
    ///
    /// `account` - The account address that has had a balance change
    pub fn update_position_balance(&self, account: Pubkey) -> Instruction {
        let accounts = ix_account::UpdatePositionBalance {
            margin_account: self.address,
            token_account: account,
        };

        Instruction {
            program_id: JetMargin::id(),
            data: ix_data::UpdatePositionBalance.data(),
            accounts: accounts.to_account_metas(None),
        }
    }

    /// Get instruction to register new position
    ///
    /// # Params
    ///
    /// `position_token_mint` - The mint for the relevant token for the position
    /// `token_oracle` - The oracle account with price information on the token
    ///
    /// # Returns
    ///
    /// Returns the instruction, and the address of the token account to be
    /// created for the position.
    pub fn register_position(&self, position_token_mint: Pubkey) -> (Pubkey, Instruction) {
        let (token_account, _) = owned_position_token_account(&self.address, &position_token_mint);

        let (metadata, _) =
            Pubkey::find_program_address(&[position_token_mint.as_ref()], &jet_metadata::ID);

        let accounts = ix_account::RegisterPosition {
            authority: self.authority(),
            payer: self.payer,
            margin_account: self.address,
            position_token_mint,
            metadata,
            token_account,
            token_program: Token::id(),
            system_program: System::id(),
            rent: Rent::id(),
        };

        let ix = Instruction {
            program_id: JetMargin::id(),
            data: ix_data::RegisterPosition {}.data(),
            accounts: accounts.to_account_metas(None),
        };

        (token_account, ix)
    }

    /// Get instruction to close a position
    ///
    /// # Params
    ///
    /// `position_token_mint` - The address of the token mint for the position, this is the
    ///   pool token mint, not the SPL mint.
    /// `token_account` - The address of the token account for the position being closed.
    pub fn close_position(
        &self,
        position_token_mint: Pubkey,
        token_account: Pubkey,
    ) -> Instruction {
        let accounts = ix_account::ClosePosition {
            authority: self.authority(),
            receiver: self.payer,
            margin_account: self.address,
            position_token_mint,
            token_account,
            token_program: Token::id(),
        };

        Instruction {
            program_id: JetMargin::id(),
            data: ix_data::ClosePosition.data(),
            accounts: accounts.to_account_metas(None),
        }
    }

    /// Get instruction to refresh the metadata for a position
    ///
    /// # Params
    ///
    /// `position_token_mint` - The mint for the position to be refreshed
    pub fn refresh_position_metadata(&self, position_token_mint: &Pubkey) -> Instruction {
        let (metadata, _) =
            Pubkey::find_program_address(&[position_token_mint.as_ref()], &jet_metadata::ID);

        let accounts = ix_account::RefreshPositionMetadata {
            metadata,
            margin_account: self.address,
        };

        Instruction {
            program_id: JetMargin::id(),
            data: ix_data::RefreshPositionMetadata.data(),
            accounts: accounts.to_account_metas(None),
        }
    }

    /// Get instruction to refresh the config for a position
    ///
    /// # Params
    ///
    /// `position_token_mint` - The mint for the position to be refreshed
    pub fn refresh_position_config(&self, position_token_mint: &Pubkey) -> Instruction {
        let config = MarginConfigIxBuilder::new(self.airspace, self.payer)
            .derive_token_config(position_token_mint);

        let accounts = ix_account::RefreshPositionConfig {
            config,
            margin_account: self.address,
        };

        Instruction {
            program_id: JetMargin::id(),
            data: ix_data::RefreshPositionConfig.data(),
            accounts: accounts.to_account_metas(None),
        }
    }

    /// Get instruction to refresh the price value for a deposit account
    ///
    /// # Params
    ///
    /// `token_config` - The token config for the position to be refreshed
    /// `price_oracle` - The price oracle for the token, stored in the token config
    pub fn refresh_deposit_position(
        &self,
        token_config: &Pubkey,
        price_oracle: &Pubkey,
    ) -> Instruction {
        let accounts = ix_account::RefreshDepositPosition {
            config: *token_config,
            price_oracle: *price_oracle,
            margin_account: self.address,
        };

        Instruction {
            program_id: JetMargin::id(),
            data: ix_data::RefreshDepositPosition.data(),
            accounts: accounts.to_account_metas(None),
        }
    }

    /// Get instruction to invoke through an adapter
    ///
    /// # Params
    ///
    /// `adapter_ix` - The instruction to be invoked
    pub fn adapter_invoke(&self, adapter_ix: Instruction) -> Instruction {
        invoke!(
            self.address,
            adapter_ix,
            AdapterInvoke { owner: self.owner }
        )
    }

    /// Get instruction to invoke through an adapter for permissionless accounting instructions
    ///
    /// # Params
    ///
    /// `adapter_ix` - The instruction to be invoked
    pub fn accounting_invoke(&self, adapter_ix: Instruction) -> Instruction {
        invoke!(self.address, adapter_ix, AccountingInvoke)
    }

    /// Begin liquidating a margin account
    ///
    /// # Params
    ///
    /// `liquidator` - The address of the liquidator
    pub fn liquidate_begin(&self, liquidator: Pubkey) -> Instruction {
        let (liquidator_metadata, _) =
            Pubkey::find_program_address(&[liquidator.as_ref()], &jet_metadata::id());

        let (liquidation, _) = Pubkey::find_program_address(
            &[b"liquidation", self.address.as_ref(), liquidator.as_ref()],
            &jet_margin::id(),
        );

        let accounts = ix_account::LiquidateBegin {
            margin_account: self.address,
            payer: self.payer,
            liquidator,
            liquidator_metadata,
            liquidation,
            system_program: SYSTEM_PROGAM_ID,
        };

        Instruction {
            program_id: JetMargin::id(),
            accounts: accounts.to_account_metas(None),
            data: ix_data::LiquidateBegin {}.data(),
        }
    }

    /// Invoke action as liquidator
    #[allow(clippy::redundant_field_names)]
    pub fn liquidator_invoke(&self, adapter_ix: Instruction, liquidator: &Pubkey) -> Instruction {
        let (liquidation, _) = Pubkey::find_program_address(
            &[b"liquidation", self.address.as_ref(), liquidator.as_ref()],
            &jet_margin::id(),
        );

        invoke!(
            self.address,
            adapter_ix,
            LiquidatorInvoke {
                liquidator: *liquidator,
                liquidation: liquidation,
            }
        )
    }

    /// End liquidating a margin account
    ///
    /// # Params
    ///
    /// `liquidator` - The address of the liquidator
    /// `original_liquidator` - The liquidator that started the liquidation process
    pub fn liquidate_end(
        &self,
        authority: Pubkey,
        original_liquidator: Option<Pubkey>,
    ) -> Instruction {
        let original = original_liquidator.unwrap_or(authority);
        let (liquidation, _) = Pubkey::find_program_address(
            &[b"liquidation", self.address.as_ref(), original.as_ref()],
            &JetMargin::id(),
        );

        let accounts = ix_account::LiquidateEnd {
            margin_account: self.address,
            authority,
            liquidation,
        };

        Instruction {
            program_id: JetMargin::id(),
            accounts: accounts.to_account_metas(None),
            data: ix_data::LiquidateEnd.data(),
        }
    }

    /// Create a new token account registered as a position
    ///
    /// Can be used to deposit tokens into the custody of the margin account, without
    /// the use of any adapters to manage it.
    ///
    /// # Params
    ///
    /// `token_mint` - The mint for the token to be deposited
    pub fn create_deposit_position(&self, token_mint: Pubkey) -> Instruction {
        let config_ix = MarginConfigIxBuilder::new(self.airspace, self.payer);
        let token_account = get_associated_token_address(&self.address, &token_mint);
        let accounts = ix_account::CreateDepositPosition {
            margin_account: self.address,
            authority: self.authority(),
            payer: self.payer,
            mint: token_mint,
            config: config_ix.derive_token_config(&token_mint),
            token_account,
            associated_token_program: associated_token::ID,
            token_program: spl_token::ID,
            system_program: system_program::ID,
            rent: Rent::id(),
        };

        Instruction {
            program_id: jet_margin::ID,
            accounts: accounts.to_account_metas(None),
            data: ix_data::CreateDepositPosition.data(),
        }
    }

    /// Transfer tokens into or out of a deposit account associated with the margin account
    pub fn transfer_deposit(
        &self,
        source_owner: Pubkey,
        source: Pubkey,
        destination: Pubkey,
        amount: u64,
    ) -> Instruction {
        let accounts = ix_account::TransferDeposit {
            owner: self.owner,
            margin_account: self.address,
            source_owner,
            source,
            destination,
            token_program: spl_token::ID,
        };

        Instruction {
            program_id: jet_margin::ID,
            data: ix_data::TransferDeposit { amount }.data(),
            accounts: accounts.to_account_metas(None),
        }
    }

    /// Verify that an account is healthy
    ///
    pub fn verify_healthy(&self) -> Instruction {
        let accounts = ix_account::VerifyHealthy {
            margin_account: self.address,
        };

        Instruction {
            program_id: JetMargin::id(),
            accounts: accounts.to_account_metas(None),
            data: ix_data::VerifyHealthy.data(),
        }
    }

    /// Helper function to get token account address for a position mint
    #[inline]
    pub fn get_token_account_address(&self, position_token_mint: &Pubkey) -> (Pubkey, u8) {
        owned_position_token_account(&self.address, position_token_mint)
    }

    fn authority(&self) -> Pubkey {
        match self.authority {
            None => self.owner,
            Some(authority) => authority,
        }
    }
}

/// Utility for creating instructions that modify configuration for the margin program within
/// an airspace
#[derive(Eq, PartialEq, Clone)]
pub struct MarginConfigIxBuilder {
    airspace: Pubkey,
    authority: Pubkey,
    payer: Pubkey,
}

impl MarginConfigIxBuilder {
    /// Create a new [MarginConfigIxBuilder] for a given airspace, with the payer as the authority
    pub fn new(airspace: Pubkey, payer: Pubkey) -> Self {
        Self {
            airspace,
            payer,
            authority: payer,
        }
    }

    /// Set the configuration for a token that can be used as a position within a margin account
    pub fn configure_token(
        &self,
        token_mint: Pubkey,
        new_config: Option<TokenConfigUpdate>,
    ) -> Instruction {
        let accounts = ix_account::ConfigureToken {
            authority: self.authority,
            airspace: self.airspace,
            payer: self.payer,
            mint: token_mint,
            token_config: self.derive_token_config(&token_mint),
            system_program: system_program::ID,
        };

        Instruction {
            program_id: jet_margin::ID,
            data: ix_data::ConfigureToken { update: new_config }.data(),
            accounts: accounts.to_account_metas(None),
        }
    }

    /// Set the configuration for an adapter program
    pub fn configure_adapter(&self, program_id: Pubkey, is_adapter: bool) -> Instruction {
        let accounts = ix_account::ConfigureAdapter {
            authority: self.authority,
            airspace: self.airspace,
            payer: self.payer,
            adapter_program: program_id,
            adapter_config: self.derive_adapter_config(&program_id),
            system_program: system_program::ID,
        };

        Instruction {
            program_id: jet_margin::ID,
            data: ix_data::ConfigureAdapter { is_adapter }.data(),
            accounts: accounts.to_account_metas(None),
        }
    }

    /// Set the configuration for a liquidator
    pub fn configure_liquidator(&self, liquidator: Pubkey, is_liquidator: bool) -> Instruction {
        let accounts = ix_account::ConfigureLiquidator {
            authority: self.authority,
            airspace: self.airspace,
            payer: self.payer,
            liquidator,
            liquidator_config: self.derive_liquidator_config(&liquidator),
            system_program: system_program::ID,
        };

        Instruction {
            program_id: jet_margin::ID,
            data: ix_data::ConfigureLiquidator { is_liquidator }.data(),
            accounts: accounts.to_account_metas(None),
        }
    }

    /// Derive address for the config account for a given token
    pub fn derive_token_config(&self, token_mint: &Pubkey) -> Pubkey {
        derive_token_config(&self.airspace, token_mint)
    }

    /// Derive address for the config account for a given adapter
    pub fn derive_adapter_config(&self, adapter_program_id: &Pubkey) -> Pubkey {
        derive_adapter_config(&self.airspace, adapter_program_id)
    }

    /// Derive address for the config account for a given liquidator
    pub fn derive_liquidator_config(&self, liquidator: &Pubkey) -> Pubkey {
        derive_liquidator_config(&self.airspace, liquidator)
    }
}

/// The token account that holds position tokens when the position is custodied
/// by the margin account
pub fn owned_position_token_account(
    margin_account: &Pubkey,
    position_token_mint: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[margin_account.as_ref(), position_token_mint.as_ref()],
        &JetMargin::id(),
    )
}

/// Derive address for the config account for a given token
pub fn derive_token_config(airspace: &Pubkey, token_mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[TOKEN_CONFIG_SEED, airspace.as_ref(), token_mint.as_ref()],
        &jet_margin::ID,
    )
    .0
}

/// Derive address for the config account for a given adapter
pub fn derive_adapter_config(airspace: &Pubkey, adapter_program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            ADAPTER_CONFIG_SEED,
            airspace.as_ref(),
            adapter_program_id.as_ref(),
        ],
        &jet_margin::ID,
    )
    .0
}

/// Derive address for the config account for a given liquidator
pub fn derive_liquidator_config(airspace: &Pubkey, liquidator: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            LIQUIDATOR_CONFIG_SEED,
            airspace.as_ref(),
            liquidator.as_ref(),
        ],
        &jet_margin::ID,
    )
    .0
}

/// Generic invocation logic that can be applied to any margin account invoke
/// instruction, such as adapter_invoke, liquidate_invoke, and accounting_invoke
macro_rules! invoke {
    (
        $margin_account:expr,
        $adapter_ix:ident,
        $Instruction:ident $({
            $($additional_field:ident: $value:expr),* $(,)?
        })?
    ) => {{
        let (adapter_metadata, _) =
            Pubkey::find_program_address(&[$adapter_ix.program_id.as_ref()], &jet_metadata::ID);

        let mut accounts = ix_account::$Instruction {
            margin_account: $margin_account,
            adapter_program: $adapter_ix.program_id,
            adapter_metadata,
            $(
                $($additional_field: $value),*
            )?
        }
        .to_account_metas(None);

        for acc in $adapter_ix.accounts {
            if acc.pubkey == $margin_account {
                accounts.push(anchor_lang::prelude::AccountMeta {
                    is_signer: false,
                    ..acc
                })
            } else {
                accounts.push(acc)
            }
        }

        Instruction {
            program_id: JetMargin::id(),
            data: ix_data::$Instruction {
                data: $adapter_ix.data,
            }
            .data(),
            accounts,
        }
    }};
}
use invoke;
