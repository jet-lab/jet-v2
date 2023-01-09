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
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{events, ErrorCode, MarginAccount, SignerSeeds};

// FIXME: no transfer support for liquidators

#[derive(Accounts)]
pub struct TransferDeposit<'info> {
    /// The authority that owns the margin account
    pub owner: Signer<'info>,

    /// The margin account that the deposit account is associated with
    #[account(mut, has_one = owner)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The authority for the source account
    pub source_owner: AccountInfo<'info>,

    /// The source account to transfer tokens from
    #[account(mut)]
    pub source: Account<'info, TokenAccount>,

    /// The destination account to transfer tokens in
    #[account(mut)]
    pub destination: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn transfer_deposit_handler(ctx: Context<TransferDeposit>, amount: u64) -> Result<()> {
    let mut margin_account = ctx.accounts.margin_account.load_mut()?;
    let source_owner = &ctx.accounts.source_owner;

    let position = match margin_account.get_position(&ctx.accounts.source.mint) {
        None => return err!(ErrorCode::PositionNotRegistered),
        Some(pos) => pos,
    };

    let position = if position.address == ctx.accounts.source.key() {
        let seeds = margin_account.signer_seeds_owned();
        drop(margin_account);

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.source.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                    authority: ctx.accounts.margin_account.to_account_info(),
                },
                &[&seeds.signer_seeds()],
            ),
            amount,
        )?;

        let source = &mut ctx.accounts.source;
        let mut margin_account = ctx.accounts.margin_account.load_mut()?;

        source.reload()?;
        margin_account.set_position_balance(&source.mint, &source.key(), source.amount)?
    } else {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.source.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                    authority: source_owner.to_account_info(),
                },
            ),
            amount,
        )?;

        let destination = &mut ctx.accounts.destination;

        destination.reload()?;
        margin_account.set_position_balance(
            &destination.mint,
            &destination.key(),
            destination.amount,
        )?
    };

    emit!(events::PositionBalanceUpdated { position });

    Ok(())
}
