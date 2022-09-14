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
use jet_margin::cpi::accounts::{PositionTokenAccounts, RegisterToken};
use jet_margin::PositionKind;

use jet_margin::program::JetMargin;
use jet_margin_pool::cpi::accounts::CreatePool;
use jet_margin_pool::program::JetMarginPool;

use crate::events;

use super::Authority;

#[derive(Accounts)]
pub struct CreateMarginPool<'info> {
    #[cfg_attr(not(feature = "testing"), account(address = crate::ROOT_AUTHORITY))]
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

    /// CHECK:
    #[account(init,
              seeds = [
                    crate::seeds::FEE_DESTINATION,
                    margin_pool.key().as_ref()
              ],
              bump,
              space = TokenAccount::LEN,
              payer = requester,
              owner = Token::id()
    )]
    fee_destination: AccountInfo<'info>,

    margin_pool_program: Program<'info, JetMarginPool>,
    margin_program: Program<'info, JetMargin>,
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
                payer: self.requester.to_account_info(),
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

    fn create_deposit_metadata_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, RegisterToken<'info>> {
        CpiContext::new(
            self.margin_program.to_account_info(),
            RegisterToken {
                metadata: self.deposit_note_metadata.to_account_info(),
                system_program: self.system_program.to_account_info(),
                other: PositionTokenAccounts {
                    requester: self.requester.to_account_info(),
                    token_mint: self.deposit_note_mint.to_account_info(),
                    adapter_program: self.margin_pool_program.to_account_info(),
                    pyth_price: self.system_program.to_account_info(),
                    pyth_product: self.system_program.to_account_info(),
                },
            },
        )
    }

    fn create_loan_metadata_context(&self) -> CpiContext<'_, '_, '_, 'info, RegisterToken<'info>> {
        CpiContext::new(
            self.margin_program.to_account_info(),
            RegisterToken {
                metadata: self.loan_note_metadata.to_account_info(),
                system_program: self.system_program.to_account_info(),
                other: PositionTokenAccounts {
                    requester: self.requester.to_account_info(),
                    token_mint: self.loan_note_mint.to_account_info(),
                    adapter_program: self.margin_pool_program.to_account_info(),
                    pyth_price: self.system_program.to_account_info(),
                    pyth_product: self.system_program.to_account_info(),
                },
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
    jet_margin::cpi::register_token(
        ctx.accounts
            .create_deposit_metadata_context()
            .with_signer(&[&authority]),
        Some(jet_margin::PositionParams {
            position_kind: PositionKind::Deposit,
            value_modifier: 0,
            max_staleness: 0,
        }),
    )?;
    emit!(events::PositionTokenMetadataConfigured {
        requester: ctx.accounts.requester.key(),
        authority: ctx.accounts.authority.key(),
        metadata_account: ctx.accounts.deposit_note_metadata.key(),
    });

    jet_margin::cpi::register_token(
        ctx.accounts
            .create_loan_metadata_context()
            .with_signer(&[&authority]),
        Some(jet_margin::PositionParams {
            position_kind: PositionKind::Deposit,
            value_modifier: 0,
            max_staleness: 0,
        }),
    )?;
    emit!(events::PositionTokenMetadataConfigured {
        requester: ctx.accounts.requester.key(),
        authority: ctx.accounts.authority.key(),
        metadata_account: ctx.accounts.loan_note_metadata.key(),
    });

    Ok(())
}
