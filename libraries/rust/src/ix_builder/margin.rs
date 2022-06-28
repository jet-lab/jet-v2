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

use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_program::ID as SYSTEM_PROGAM_ID;
use solana_sdk::sysvar::{rent::Rent, SysvarId};

use anchor_lang::prelude::{Id, System, ToAccountMetas};
use anchor_lang::InstructionData;
use anchor_spl::token::Token;

use jet_margin::instruction as ix_data;
use jet_margin::program::JetMargin;
use jet_margin::{accounts as ix_account, CompactAccountMeta};

/// Utility for creating instructions to interact with the margin
/// program for a specific account.
pub struct MarginIxBuilder {
    /// The account owner,
    pub owner: Pubkey,

    /// The account seed
    pub seed: u16,

    /// The account paying for any rent
    pub payer: Pubkey,

    /// The address of the margin account for the owner
    pub address: Pubkey,

    /// The authority to use in place of the owner
    authority: Option<Pubkey>,
}

impl MarginIxBuilder {
    pub fn new(owner: Pubkey, seed: u16) -> Self {
        Self::new_with_payer(owner, seed, owner)
    }

    pub fn new_with_payer(owner: Pubkey, seed: u16, payer: Pubkey) -> Self {
        let (address, _) = Pubkey::find_program_address(
            &[owner.as_ref(), 0u16.to_le_bytes().as_ref()],
            &jet_margin::ID,
        );
        Self {
            owner,
            seed,
            payer,
            address,
            authority: None,
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
        let (token_account, _) = self.get_token_account_address(&position_token_mint);

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
        Pubkey::find_program_address(
            &[self.address.as_ref(), position_token_mint.as_ref()],
            &JetMargin::id(),
        )
    }

    fn authority(&self) -> Pubkey {
        match self.authority {
            None => self.owner,
            Some(authority) => authority,
        }
    }
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

        let adapter_metas = $adapter_ix.accounts.iter().skip(1);
        let compact_account_metas = adapter_metas
            .clone() // accounts already provided above
            .map(|a| CompactAccountMeta {
                is_signer: if a.is_signer { 1 } else { 0 },
                is_writable: if a.is_writable { 1 } else { 0 },
            })
            .collect();

        accounts.extend(adapter_metas.cloned());

        Instruction {
            program_id: JetMargin::id(),
            data: ix_data::$Instruction {
                data: $adapter_ix.data,
                account_metas: compact_account_metas,
            }
            .data(),
            accounts,
        }
    }};
}
use invoke;
