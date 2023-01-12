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

use anchor_spl::associated_token::get_associated_token_address;
use jet_margin::seeds::{ADAPTER_CONFIG_SEED, PERMIT_SEED, TOKEN_CONFIG_SEED};
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_program::ID as SYSTEM_PROGAM_ID;
use solana_sdk::sysvar::{rent::Rent, SysvarId};

use anchor_lang::prelude::{Id, System, ToAccountMetas};
use anchor_lang::{system_program, InstructionData};

use jet_margin::accounts as ix_account;
use jet_margin::instruction as ix_data;
use jet_margin::program::JetMargin;
pub use jet_margin::{TokenAdmin, TokenConfigUpdate, TokenKind, TokenOracle};

pub use jet_margin::ID as MARGIN_PROGRAM;

/// Utility for creating instructions to interact with the margin
/// program for a specific account.
#[derive(Clone)]
pub struct MarginIxBuilder {
    /// Owner of the margin account.
    pub owner: Pubkey,

    /// Seed used to generate margin account PDA.
    pub seed: u16,

    /// The address of the margin account.
    pub address: Pubkey,

    /// The address of the airspace the margin account belongs to.
    pub airspace: Pubkey,

    /// The account paying for any rent.
    /// - Defaults to authority, which defaults to owner.
    payer: Option<Pubkey>,

    /// Key that will sign to authorize changes to the margin account.
    /// - Defaults to owner.
    authority: Option<Pubkey>,
}

impl MarginIxBuilder {
    /// Create a new [MarginIxBuilder] which uses the margin account as the authority.
    /// Ordinary margin users should use this function to create a builder.
    pub fn new(airspace: Pubkey, owner: Pubkey, seed: u16) -> Self {
        let (address, _) = Pubkey::find_program_address(
            &[owner.as_ref(), seed.to_le_bytes().as_ref()],
            &jet_margin::ID,
        );
        Self {
            owner,
            seed,
            payer: None,
            address,
            airspace,
            authority: None,
        }
    }

    pub fn new_for_address(airspace: Pubkey, address: Pubkey, payer: Pubkey) -> Self {
        Self {
            owner: payer,
            seed: 0,
            payer: Some(payer),
            address,
            airspace,
            authority: None,
        }
    }

    /// Use if an administrator is managing the account instead of the owner.
    pub fn with_authority(mut self, authority: Pubkey) -> Self {
        self.authority = Some(authority);
        self
    }

    /// Use if a different wallet should pay or receive rent instead of the authority.
    pub fn with_payer(mut self, payer: Pubkey) -> Self {
        self.payer = Some(payer);
        self
    }

    pub fn authority(&self) -> Pubkey {
        self.authority.unwrap_or(self.owner)
    }

    pub fn payer(&self) -> Pubkey {
        self.payer.unwrap_or_else(|| self.authority())
    }

    /// Get instruction to create the account
    pub fn create_account(&self) -> Instruction {
        let accounts = ix_account::CreateAccount {
            owner: self.owner,
            payer: self.payer(),
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
            receiver: self.payer(),
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
    pub fn register_position(&self, position_token_mint: Pubkey) -> Instruction {
        let token_account = derive_position_token_account(&self.address, &position_token_mint);

        let (metadata, _) =
            Pubkey::find_program_address(&[position_token_mint.as_ref()], &jet_metadata::ID);

        let accounts = ix_account::RegisterPosition {
            authority: self.authority(),
            payer: self.payer(),
            margin_account: self.address,
            position_token_mint,
            metadata,
            token_account,
            token_program: spl_token::ID,
            system_program: System::id(),
            rent: Rent::id(),
        };

        Instruction {
            program_id: JetMargin::id(),
            data: ix_data::RegisterPosition {}.data(),
            accounts: accounts.to_account_metas(None),
        }
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
            receiver: self.payer(),
            margin_account: self.address,
            position_token_mint,
            token_account,
            token_program: spl_token::ID,
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
            permit: derive_margin_permit(&self.airspace, &self.authority()),
            refresher: self.authority(),
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
        let config = MarginConfigIxBuilder::new(self.airspace, self.payer(), None)
            .derive_token_config(position_token_mint);

        let accounts = ix_account::RefreshPositionConfig {
            config,
            margin_account: self.address,
            permit: derive_margin_permit(&self.airspace, &self.authority()),
            refresher: self.authority(),
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
        accounting_invoke(self.address, adapter_ix)
    }

    /// Begin liquidating a margin account
    ///
    /// # Params
    ///
    /// `liquidator` - The address of the liquidator
    pub fn liquidate_begin(&self) -> Instruction {
        let liquidator = self.authority();
        let (liquidator_metadata, _) =
            Pubkey::find_program_address(&[liquidator.as_ref()], &jet_metadata::id());

        let (liquidation, _) = Pubkey::find_program_address(
            &[b"liquidation", self.address.as_ref(), liquidator.as_ref()],
            &jet_margin::id(),
        );

        let accounts = ix_account::LiquidateBegin {
            margin_account: self.address,
            payer: self.payer(),
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
    pub fn liquidator_invoke(&self, adapter_ix: Instruction) -> Instruction {
        let liquidator = self.authority();
        let (liquidation, _) = Pubkey::find_program_address(
            &[b"liquidation", self.address.as_ref(), liquidator.as_ref()],
            &jet_margin::id(),
        );

        invoke!(
            self.address,
            adapter_ix,
            LiquidatorInvoke {
                liquidator,
                liquidation,
            }
        )
    }

    /// End liquidating a margin account
    ///
    /// # Params
    ///
    /// `liquidator` - The address of the liquidator
    /// `original_liquidator` - The liquidator that started the liquidation process
    pub fn liquidate_end(&self, original_liquidator: Option<Pubkey>) -> Instruction {
        let authority = self.authority();
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
        let config_ix = MarginConfigIxBuilder::new(self.airspace, self.payer(), None);
        let token_account = get_associated_token_address(&self.address, &token_mint);
        let accounts = ix_account::CreateDepositPosition {
            margin_account: self.address,
            authority: self.authority(),
            payer: self.payer(),
            mint: token_mint,
            config: config_ix.derive_token_config(&token_mint),
            token_account,
            associated_token_program: spl_associated_token_account::ID,
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

    /// Peform an administrative transfer for a position
    pub fn admin_transfer_position_to(
        &self,
        target: &Pubkey,
        position_token_mint: &Pubkey,
        amount: u64,
    ) -> Instruction {
        let accounts = ix_account::AdminTransferPosition {
            authority: jet_program_common::GOVERNOR_ID,
            source_account: self.address,
            target_account: *target,
            source_token_account: self.get_token_account_address(position_token_mint),
            target_token_account: derive_position_token_account(target, position_token_mint),
            token_program: spl_token::ID,
        }
        .to_account_metas(None);

        Instruction {
            program_id: jet_margin::ID,
            data: ix_data::AdminTransferPosition { amount }.data(),
            accounts,
        }
    }

    /// Helper function to get token account address for a position mint
    #[inline]
    pub fn get_token_account_address(&self, position_token_mint: &Pubkey) -> Pubkey {
        derive_position_token_account(&self.address, position_token_mint)
    }
}

/// Get instruction to invoke through an adapter for permissionless accounting instructions
///
/// # Params
///
/// `adapter_ix` - The instruction to be invoked
pub fn accounting_invoke(margin_account: Pubkey, adapter_ix: Instruction) -> Instruction {
    invoke!(margin_account, adapter_ix, AccountingInvoke)
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
    /// Create a new [MarginConfigIxBuilder] for a given airspace, assuming the
    /// payer is the authority if not provided.
    pub fn new(airspace: Pubkey, payer: Pubkey, airspace_authority: Option<Pubkey>) -> Self {
        Self {
            airspace,
            authority: airspace_authority.unwrap_or(payer),
            payer,
        }
    }

    /// Set the authority address to use separately from the payer
    pub fn with_authority(self, authority: Pubkey) -> Self {
        Self { authority, ..self }
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
        Instruction {
            program_id: jet_margin::ID,
            data: ix_data::ConfigureLiquidator { is_liquidator }.data(),
            accounts: self.configure_permit(liquidator).to_account_metas(None),
        }
    }

    /// Enable or disable permission to refresh position metadata
    pub fn configure_position_config_refresher(
        &self,
        refresher: Pubkey,
        may_refresh: bool,
    ) -> Instruction {
        Instruction {
            program_id: jet_margin::ID,
            data: ix_data::ConfigurePositionConfigRefresher { may_refresh }.data(),
            accounts: self.configure_permit(refresher).to_account_metas(None),
        }
    }

    /// get the accounts to configure a permit
    fn configure_permit(&self, owner: Pubkey) -> ix_account::ConfigurePermit {
        ix_account::ConfigurePermit {
            authority: self.authority,
            airspace: self.airspace,
            payer: self.payer,
            owner,
            permit: self.derive_margin_permit(&owner),
            system_program: system_program::ID,
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
    #[deprecated(note = "use derive_margin_permit")]
    pub fn derive_liquidator_config(&self, liquidator: &Pubkey) -> Pubkey {
        derive_margin_permit(&self.airspace, liquidator)
    }

    /// Derive address for a user's permit account in an airspace
    pub fn derive_margin_permit(&self, liquidator: &Pubkey) -> Pubkey {
        derive_margin_permit(&self.airspace, liquidator)
    }
}

/// The token account that holds position tokens when the position is custodied
/// by the margin account
pub fn derive_position_token_account(
    margin_account: &Pubkey,
    position_token_mint: &Pubkey,
) -> Pubkey {
    Pubkey::find_program_address(
        &[margin_account.as_ref(), position_token_mint.as_ref()],
        &JetMargin::id(),
    )
    .0
}

/// Derive the address for a user's margin account
pub fn derive_margin_account(_airspace: &Pubkey, owner: &Pubkey, seed: u16) -> Pubkey {
    Pubkey::find_program_address(
        &[owner.as_ref(), seed.to_le_bytes().as_ref()],
        &jet_margin::ID,
    )
    .0
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
#[deprecated(note = "use derive_margin_permit")]
pub fn derive_liquidator_config(airspace: &Pubkey, liquidator: &Pubkey) -> Pubkey {
    derive_margin_permit(airspace, liquidator)
}

/// Derive address for a user's permit account in an airspace
pub fn derive_margin_permit(airspace: &Pubkey, owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[PERMIT_SEED, airspace.as_ref(), owner.as_ref()],
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
            $($additional_field:ident$(: $value:expr)?),* $(,)?
        })?
    ) => {{
        let (adapter_metadata, _) =
            Pubkey::find_program_address(&[$adapter_ix.program_id.as_ref()], &jet_metadata::ID);

        let mut accounts = ix_account::$Instruction {
            margin_account: $margin_account,
            adapter_program: $adapter_ix.program_id,
            adapter_metadata,
            $($($additional_field$(: $value)?),*)?
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
