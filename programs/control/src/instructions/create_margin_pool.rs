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
use anchor_spl::token::{InitializeAccount, Token, TokenAccount};
use std::convert::TryInto;

use jet_margin_pool::cpi::accounts::CreatePool;
use jet_margin_pool::program::JetMarginPool;
use jet_metadata::cpi::accounts::{CreateEntry, SetEntry};
use jet_metadata::program::JetMetadata;
use jet_metadata::{PositionTokenMetadata, TokenKind, TokenMetadata};

#[cfg(not(feature = "testing"))]
use jet_program_common::GOVERNOR_ID;

use crate::events;

use super::Authority;

#[derive(Accounts)]
pub struct CreateMarginPool<'info> {
    #[cfg_attr(not(feature = "testing"), account(address = GOVERNOR_ID))]
    requester: Signer<'info>,

    #[account(mut)]
    payer: AccountInfo<'info>,

    authority: Account<'info, Authority>,

    /// CHECK:
    #[account(mut)]
    margin_pool: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    vault: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    deposit_note_mint: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    loan_note_mint: UncheckedAccount<'info>,

    /// CHECK:
    token_mint: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    token_metadata: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    deposit_note_metadata: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    loan_note_metadata: UncheckedAccount<'info>,

    /// CHECK:
    #[account(init,
              seeds = [
                    crate::seeds::FEE_DESTINATION,
                    margin_pool.key().as_ref()
              ],
              bump,
              space = TokenAccount::LEN,
              payer = payer,
              owner = Token::id()
    )]
    fee_destination: AccountInfo<'info>,

    margin_pool_program: Program<'info, JetMarginPool>,
    metadata_program: Program<'info, JetMetadata>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

impl<'info> CreateMarginPool<'info> {
    fn create_pool_context(&self) -> CpiContext<'_, '_, '_, 'info, CreatePool<'info>> {
        CpiContext::new(
            self.margin_pool_program.to_account_info(),
            CreatePool {
                margin_pool: self.margin_pool.to_account_info(),
                vault: self.vault.to_account_info(),
                deposit_note_mint: self.deposit_note_mint.to_account_info(),
                loan_note_mint: self.loan_note_mint.to_account_info(),
                token_mint: self.token_mint.to_account_info(),
                authority: self.authority.to_account_info(),
                payer: self.payer.to_account_info(),
                token_program: self.token_program.to_account_info(),
                system_program: self.system_program.to_account_info(),
                rent: self.rent.to_account_info(),
            },
        )
    }

    fn create_fee_destination_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeAccount<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            InitializeAccount {
                account: self.fee_destination.to_account_info(),
                mint: self.deposit_note_mint.to_account_info(),
                authority: self.authority.to_account_info(),
                rent: self.rent.to_account_info(),
            },
        )
    }

    fn create_token_metadata_context(&self) -> CpiContext<'_, '_, '_, 'info, CreateEntry<'info>> {
        CpiContext::new(
            self.metadata_program.to_account_info(),
            CreateEntry {
                key_account: self.token_mint.to_account_info(),
                metadata_account: self.token_metadata.to_account_info(),
                authority: self.authority.to_account_info(),
                payer: self.requester.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
        )
    }

    fn set_token_metadata_context(&self) -> CpiContext<'_, '_, '_, 'info, SetEntry<'info>> {
        CpiContext::new(
            self.metadata_program.to_account_info(),
            SetEntry {
                metadata_account: self.token_metadata.to_account_info(),
                authority: self.authority.to_account_info(),
            },
        )
    }

    fn create_deposit_metadata_context(&self) -> CpiContext<'_, '_, '_, 'info, CreateEntry<'info>> {
        CpiContext::new(
            self.metadata_program.to_account_info(),
            CreateEntry {
                key_account: self.deposit_note_mint.to_account_info(),
                metadata_account: self.deposit_note_metadata.to_account_info(),
                authority: self.authority.to_account_info(),
                payer: self.requester.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
        )
    }

    fn set_deposit_metadata_context(&self) -> CpiContext<'_, '_, '_, 'info, SetEntry<'info>> {
        CpiContext::new(
            self.metadata_program.to_account_info(),
            SetEntry {
                metadata_account: self.deposit_note_metadata.to_account_info(),
                authority: self.authority.to_account_info(),
            },
        )
    }

    fn create_loan_metadata_context(&self) -> CpiContext<'_, '_, '_, 'info, CreateEntry<'info>> {
        CpiContext::new(
            self.metadata_program.to_account_info(),
            CreateEntry {
                key_account: self.loan_note_mint.to_account_info(),
                metadata_account: self.loan_note_metadata.to_account_info(),
                authority: self.authority.to_account_info(),
                payer: self.requester.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
        )
    }

    fn set_loan_metadata_context(&self) -> CpiContext<'_, '_, '_, 'info, SetEntry<'info>> {
        CpiContext::new(
            self.metadata_program.to_account_info(),
            SetEntry {
                metadata_account: self.loan_note_metadata.to_account_info(),
                authority: self.authority.to_account_info(),
            },
        )
    }
}

#[inline(never)]
pub fn create_margin_pool_handler(ctx: Context<CreateMarginPool>) -> Result<()> {
    let authority = [&ctx.accounts.authority.seed[..]];

    // create the pool
    jet_margin_pool::cpi::create_pool(
        ctx.accounts
            .create_pool_context()
            .with_signer(&[&authority]),
        ctx.accounts.fee_destination.key(),
    )?;

    // create fee collection account
    anchor_spl::token::initialize_account(ctx.accounts.create_fee_destination_context())?;

    // set metadata for the deposit/loan tokens to be used as positions
    let deposit_note_metadata = PositionTokenMetadata {
        underlying_token_mint: ctx.accounts.token_mint.key(),
        position_token_mint: ctx.accounts.deposit_note_mint.key(),
        adapter_program: ctx.accounts.margin_pool_program.key(),
        token_kind: TokenKind::NonCollateral,
        value_modifier: 0,
        max_staleness: 0,
    };

    let loan_note_metadata = PositionTokenMetadata {
        underlying_token_mint: ctx.accounts.token_mint.key(),
        position_token_mint: ctx.accounts.loan_note_mint.key(),
        adapter_program: ctx.accounts.margin_pool_program.key(),
        token_kind: TokenKind::Claim,
        value_modifier: 0,
        max_staleness: 0,
    };

    let token_metadata = TokenMetadata {
        token_mint: ctx.accounts.token_mint.key(),
        ..Default::default()
    };

    let mut token_md_data = vec![];
    let mut deposit_md_data = vec![];
    let mut loan_md_data = vec![];

    deposit_note_metadata.try_serialize(&mut deposit_md_data)?;
    loan_note_metadata.try_serialize(&mut loan_md_data)?;
    token_metadata.try_serialize(&mut token_md_data)?;

    jet_metadata::cpi::create_entry(
        ctx.accounts
            .create_deposit_metadata_context()
            .with_signer(&[&authority]),
        String::new(),
        deposit_md_data.len().try_into().unwrap(),
    )?;

    jet_metadata::cpi::set_entry(
        ctx.accounts
            .set_deposit_metadata_context()
            .with_signer(&[&authority]),
        0,
        deposit_md_data,
    )?;

    emit!(events::PositionTokenMetadataConfigured {
        requester: ctx.accounts.requester.key(),
        authority: ctx.accounts.authority.key(),
        metadata_account: ctx.accounts.deposit_note_metadata.key(),
        metadata: deposit_note_metadata,
    });

    jet_metadata::cpi::create_entry(
        ctx.accounts
            .create_loan_metadata_context()
            .with_signer(&[&authority]),
        String::new(),
        loan_md_data.len().try_into().unwrap(),
    )?;

    jet_metadata::cpi::set_entry(
        ctx.accounts
            .set_loan_metadata_context()
            .with_signer(&[&authority]),
        0,
        loan_md_data,
    )?;

    emit!(events::PositionTokenMetadataConfigured {
        requester: ctx.accounts.requester.key(),
        authority: ctx.accounts.authority.key(),
        metadata_account: ctx.accounts.loan_note_metadata.key(),
        metadata: loan_note_metadata,
    });

    jet_metadata::cpi::create_entry(
        ctx.accounts
            .create_token_metadata_context()
            .with_signer(&[&authority]),
        String::new(),
        token_md_data.len().try_into().unwrap(),
    )?;

    jet_metadata::cpi::set_entry(
        ctx.accounts
            .set_token_metadata_context()
            .with_signer(&[&authority]),
        0,
        token_md_data,
    )?;

    emit!(events::TokenMetadataConfigured {
        requester: ctx.accounts.requester.key(),
        authority: ctx.accounts.authority.key(),
        metadata_account: ctx.accounts.token_metadata.key(),
        metadata: token_metadata,
    });

    Ok(())
}
