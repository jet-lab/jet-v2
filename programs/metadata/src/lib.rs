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

use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use solana_program::pubkey;

declare_id!("JPMetawzxw7WyH3qHUVScYHWFBGhjwqDnM2R9qVbRLp");

pub static CONTROL_PROGRAM_ID: Pubkey = pubkey!("JPCtrLreUqsEbdhtxZ8zpd8wBydKz4nuEjX5u9Eg5H8");

mod authority {
    use super::*;

    declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");
}

#[derive(Accounts)]
#[instruction(seed: String, space: usize)]
pub struct CreateEntry<'info> {
    /// The account used as the key to lookup
    /// CHECK:
    pub key_account: AccountInfo<'info>,

    /// The account containing the metadata for the key
    /// CHECK:
    #[account(init,
              seeds = [key_account.key.as_ref(), seed.as_bytes()],
              bump,
              space = space,
              payer = payer
    )]
    pub metadata_account: AccountInfo<'info>,

    /// The authority that must sign to make this change
    #[cfg_attr(not(feature = "devnet"), account(signer))]
    pub authority: Account<'info, ControlAuthority>,

    /// The address paying the rent for the account
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetEntry<'info> {
    /// The account containing the metadata to change
    /// CHECK:
    #[account(mut)]
    pub metadata_account: AccountInfo<'info>,

    /// The authority that must sign to make this change
    #[cfg_attr(not(feature = "devnet"), account(signer))]
    pub authority: Account<'info, ControlAuthority>,
}

#[derive(Accounts)]
pub struct RemoveEntry<'info> {
    /// The account containing the metadata to change
    /// CHECK:
    #[account(mut)]
    pub metadata_account: AccountInfo<'info>,

    /// The authority that must sign to make this change
    #[cfg_attr(not(feature = "devnet"), account(signer))]
    pub authority: Account<'info, ControlAuthority>,

    /// The address receiving the rent
    #[account(mut)]
    pub receiver: AccountInfo<'info>,
}

#[program]
mod jet_metadata {
    use super::*;

    #[allow(unused_variables)]
    pub fn create_entry(ctx: Context<CreateEntry>, seed: String, space: u64) -> Result<()> {
        // no op
        Ok(())
    }

    pub fn set_entry(ctx: Context<SetEntry>, offset: u64, data: Vec<u8>) -> Result<()> {
        let mut metadata = ctx.accounts.metadata_account.data.borrow_mut();

        let offset: usize = offset as usize;
        (&mut metadata[offset..offset + data.len()]).copy_from_slice(&data);
        Ok(())
    }

    pub fn remove_entry(ctx: Context<RemoveEntry>) -> Result<()> {
        let mut source = ctx.accounts.metadata_account.try_borrow_mut_lamports()?;
        let mut dest = ctx.accounts.receiver.try_borrow_mut_lamports()?;

        **dest = dest.checked_add(**source).unwrap();
        **source = 0;

        let mut data = ctx.accounts.metadata_account.try_borrow_mut_data()?;
        for i in 0..8 {
            data[i] = 0;
        }

        Ok(())
    }
}

/// Description of the token's usage
#[derive(AnchorSerialize, AnchorDeserialize, Eq, PartialEq, Clone, Copy, Debug)]
pub enum TokenKind {
    /// The token has no value within the margin system
    NonCollateral,

    /// The token can be used as collateral
    Collateral,

    /// The token represents a debt that needs to be repaid
    Claim,
}

impl Default for TokenKind {
    fn default() -> TokenKind {
        Self::NonCollateral
    }
}

/// A metadata account referencing information about a position token
#[account]
#[derive(Default)]
pub struct PositionTokenMetadata {
    /// The mint for the position token
    pub position_token_mint: Pubkey,

    /// The underlying token represented by this position
    pub underlying_token_mint: Pubkey,

    /// The adapter program in control of this position
    pub adapter_program: Pubkey,

    /// Description of this token
    pub token_kind: TokenKind,

    /// The weight of the asset's value relative to other tokens when used as collateral.
    pub collateral_weight: u16,

    /// The maximum staleness (seconds) that's acceptable for this token when used as collateral.
    pub collateral_max_staleness: u64,
}

/// An account that references information about a token's price oracle
#[account]
#[derive(Default)]
pub struct TokenMetadata {
    /// The address of the mint for the token being referenced
    pub token_mint: Pubkey,

    /// The address of the price oracle which contains the price data for
    /// the associated token.
    pub pyth_price: Pubkey,

    /// The address of the pyth product metadata associated with the price oracle
    pub pyth_product: Pubkey,
}

/// An account that references a program that's allowed to be invoked by
/// proxy via a margin account.
#[account]
#[derive(Default)]
pub struct MarginAdapterMetadata {
    /// The address of the allowed program
    pub adapter_program: Pubkey,
}

/// An account referencing a liquidator, allowed to use the liquidation
/// instructions on margin accounts.
#[account]
#[derive(Default)]
pub struct LiquidatorMetadata {
    pub liquidator: Pubkey,
}

/// An account representing the Control program's authority
///
/// This can be used when specifying the account parameters for an
/// instruction using Anchor, to validate the provided account is the
/// correct authority created by the control program.
#[derive(Debug, Clone)]
pub struct ControlAuthority {}

impl anchor_lang::Discriminator for ControlAuthority {
    fn discriminator() -> [u8; 8] {
        [36, 108, 254, 18, 167, 144, 27, 36]
    }
}

impl anchor_lang::Owner for ControlAuthority {
    fn owner() -> Pubkey {
        CONTROL_PROGRAM_ID
    }
}

impl anchor_lang::AccountSerialize for ControlAuthority {}

impl anchor_lang::AccountDeserialize for ControlAuthority {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        if buf[..8] != Self::discriminator() {
            return err!(anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch);
        }

        Ok(Self {})
    }
}
