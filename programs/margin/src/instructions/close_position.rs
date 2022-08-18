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
use anchor_spl::token::{self, CloseAccount, Mint, Token, TokenAccount};

use crate::{events, Approver, MarginAccount, SignerSeeds};

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    /// The authority that can change the margin account
    pub authority: Signer<'info>,

    /// The receiver for the rent released
    /// CHECK:
    #[account(mut)]
    pub receiver: AccountInfo<'info>,

    /// The margin account with the position to close
    #[account(mut)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The mint for the position token being deregistered
    pub position_token_mint: Account<'info, Mint>,

    /// The token account for the position being closed
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

/// ## close\_position.rs
/// 
/// This instruction does the following:
/// 
/// 1.  Let `account` be a mutable reference to the margin account.
/// 
/// 2.  Verify the authority of `account`.
/// 
/// 3.  Record unregistering (closing) the position in question of `account`, which involves passing the token mint account, token account, and margin account authority.
/// 
/// 4.  If the token account authority of the account is the same as the authority.
/// 
///     1.  Return the token account.
/// 
/// 5.  Emit the `PositionClosed` event for data logging (see table below):
/// 
/// 6.  Return `Ok(())`.
/// 
/// 
/// **Parameters of close\_position.rs:**
/// 
/// |     |     |
/// | --- | --- |
/// | **Name** | **Description** |
/// | `authority` | The authority that can change the margin account. |
/// | `receiver` | The receiver for the rent released. |
/// | `margin_account` | The margin account with the position to close. |
/// | `position_token_mint` | The mint for the position token being deregistered. |
/// | `token_account` | The token account for the position being closed. |
/// | `token_program` | The token program for the position being closed. |
/// 
/// **Events emitted by close\_position.rs:**
/// 
/// |     |     |
/// | --- | --- |
/// | **Event Name** | **Description** |
/// | `PositionClosed` | The closed position (includes the margin account authority’s pubkey and the relevant token pool’s note mint pubkey). |

impl<'info> ClosePosition<'info> {
    fn close_token_account_ctx(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            CloseAccount {
                account: self.token_account.to_account_info(),
                authority: self.margin_account.to_account_info(),
                destination: self.receiver.to_account_info(),
            },
        )
    }
}

pub fn close_position_handler(ctx: Context<ClosePosition>) -> Result<()> {
    {
        let mut account = ctx.accounts.margin_account.load_mut()?;
        account.verify_authority(ctx.accounts.authority.key())?;

        account.unregister_position(
            &ctx.accounts.position_token_mint.key(),
            &ctx.accounts.token_account.key(),
            &[Approver::MarginAccountAuthority],
        )?;
    }

    if ctx.accounts.token_account.owner == ctx.accounts.margin_account.key() {
        let account = ctx.accounts.margin_account.load()?;
        token::close_account(
            ctx.accounts
                .close_token_account_ctx()
                .with_signer(&[&account.signer_seeds()]),
        )?;
    }

    emit!(events::PositionClosed {
        authority: ctx.accounts.authority.key(),
        token: ctx.accounts.position_token_mint.key(),
    });

    Ok(())
}
