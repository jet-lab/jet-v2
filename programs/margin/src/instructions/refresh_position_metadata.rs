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

use jet_metadata::PositionTokenMetadata;

use crate::{events, MarginAccount, Permissions, Permit};

#[derive(Accounts)]
pub struct RefreshPositionMetadata<'info> {
    /// The margin account with the position to be refreshed
    #[account(mut)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The metadata account for the token, which has been updated
    pub metadata: Account<'info, PositionTokenMetadata>,

    /// permit that authorizes the refresher
    pub permit: Account<'info, Permit>,

    /// account that is authorized to refresh position metadata
    pub refresher: Signer<'info>,
}

/// Refresh the metadata for a position
pub fn refresh_position_metadata_handler(ctx: Context<RefreshPositionMetadata>) -> Result<()> {
    ctx.accounts.permit.validate(
        Pubkey::default(), // todo airspace must come from the margin account once they are airspace-scoped
        ctx.accounts.refresher.key(),
        Permissions::REFRESH_POSITION_METADATA,
    )?;
    let metadata = &ctx.accounts.metadata;
    let mut account = ctx.accounts.margin_account.load_mut()?;

    let position = account.refresh_position_metadata(
        &metadata.position_token_mint,
        metadata.token_kind.into(),
        metadata.value_modifier,
        metadata.max_staleness,
    )?;

    emit!(events::PositionMetadataRefreshed {
        margin_account: ctx.accounts.margin_account.key(),
        position,
    });

    Ok(())
}
