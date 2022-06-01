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
use anchor_spl::token::Token;
use std::convert::TryInto;

use jet_margin_pool::cpi::accounts::CreatePool;
use jet_margin_pool::program::JetMarginPool;
use jet_metadata::cpi::accounts::{CreateEntry, SetEntry};
use jet_metadata::program::JetMetadata;
use jet_metadata::{PositionTokenMetadata, TokenKind, TokenMetadata};

use crate::events;

use super::Authority;

#[derive(Accounts)]
pub struct RegisterToken<'info> {
    #[account(mut)]
    requester: Signer<'info>,
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

    margin_pool_program: Program<'info, JetMarginPool>,
    metadata_program: Program<'info, JetMetadata>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

impl<'info> RegisterToken<'info> {
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
                payer: self.requester.to_account_info(),
                token_program: self.token_program.to_account_info(),
                system_program: self.system_program.to_account_info(),
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

pub fn register_token_handler(ctx: Context<RegisterToken>) -> Result<()> {
    let authority = [&ctx.accounts.authority.seed[..]];

    // create the pool
    jet_margin_pool::cpi::create_pool(
        ctx.accounts
            .create_pool_context()
            .with_signer(&[&authority]),
    )?;

    // set metadata for the deposit/loan tokens to be used as positions
    let deposit_note_metadata = PositionTokenMetadata {
        underlying_token_mint: ctx.accounts.token_mint.key(),
        position_token_mint: ctx.accounts.deposit_note_mint.key(),
        adapter_program: ctx.accounts.margin_pool_program.key(),
        token_kind: TokenKind::NonCollateral,
        collateral_weight: 0,
        collateral_max_staleness: 0,
    };

    let loan_note_metadata = PositionTokenMetadata {
        underlying_token_mint: ctx.accounts.token_mint.key(),
        position_token_mint: ctx.accounts.loan_note_mint.key(),
        adapter_program: ctx.accounts.margin_pool_program.key(),
        token_kind: TokenKind::Claim,
        collateral_weight: 0,
        collateral_max_staleness: 0,
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

    emit!(events::TokenRegistered { 
        requester: ctx.accounts.requester.key(), 
        authority: ctx.accounts.authority.key(),
        margin_pool: ctx.accounts.margin_pool.key(),
        vault: ctx.accounts.vault.key(),
        deposit_note_mint: ctx.accounts.deposit_note_mint.key(),
        loan_note_mint: ctx.accounts.loan_note_mint.key(),
        token_mint: ctx.accounts.token_mint.key(),
        token_metadata: ctx.accounts.token_metadata.key(),
        deposit_note_metadata: ctx.accounts.deposit_note_metadata.key(), 
        loan_note_metadata: ctx.accounts.loan_note_metadata.key(), 
        margin_pool_program: ctx.accounts.margin_pool_program.key(), 
        metadata_program: ctx.accounts.metadata_program.key() 
    });

    Ok(())
}
