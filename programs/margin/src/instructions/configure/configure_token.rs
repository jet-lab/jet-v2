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

use anchor_lang::{prelude::*, AccountsClose};

use anchor_spl::token::Mint;
use jet_airspace::state::Airspace;

use crate::{
    events::TokenConfigured, seeds::TOKEN_CONFIG_SEED, ErrorCode, TokenAdmin, TokenConfig,
    TokenKind,
};

#[derive(AnchorDeserialize, AnchorSerialize, Debug, Eq, PartialEq, Clone)]
pub struct TokenConfigUpdate {
    /// The underlying token represented, if any
    pub underlying_mint: Pubkey,

    /// The administration authority for the token
    pub admin: TokenAdmin,

    /// Description of this token
    pub token_kind: TokenKind,

    /// A modifier to adjust the token value, based on the kind of token
    pub value_modifier: u16,

    /// The maximum staleness (seconds) that's acceptable for balances of this token
    pub max_staleness: u64,
}

#[derive(Accounts)]
pub struct ConfigureToken<'info> {
    /// The authority allowed to make changes to configuration
    pub authority: Signer<'info>,

    /// The airspace being modified
    #[account(has_one = authority)]
    pub airspace: Account<'info, Airspace>,

    /// The payer for any rent costs, if required
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The mint for the token being configured
    pub mint: Account<'info, Mint>,

    /// The config account to be modified
    #[account(init_if_needed,
              seeds = [
                TOKEN_CONFIG_SEED,
                airspace.key().as_ref(),
                mint.key().as_ref()
              ],
              bump,
              payer = payer,
              space = 8 + std::mem::size_of::<TokenConfig>(),
    )]
    pub token_config: Account<'info, TokenConfig>,

    pub system_program: Program<'info, System>,
}

pub fn configure_token_handler(
    ctx: Context<ConfigureToken>,
    updated_config: Option<TokenConfigUpdate>,
) -> Result<()> {
    let config = &mut ctx.accounts.token_config;

    emit!(TokenConfigured {
        airspace: ctx.accounts.airspace.key(),
        mint: ctx.accounts.mint.key(),
        update: updated_config.clone(),
    });

    let updated_config = match updated_config {
        Some(update) => update,
        None => return config.close(ctx.accounts.payer.to_account_info()),
    };

    if config.underlying_mint != Pubkey::default()
        && updated_config.underlying_mint != config.underlying_mint
    {
        msg!("underlying mint cannot be changed");
        return err!(ErrorCode::InvalidConfig);
    }

    config.mint = ctx.accounts.mint.key();
    config.airspace = ctx.accounts.airspace.key();
    config.underlying_mint = updated_config.underlying_mint;
    config.admin = updated_config.admin;
    config.token_kind = updated_config.token_kind;
    config.value_modifier = updated_config.value_modifier;
    config.max_staleness = updated_config.max_staleness;

    config.validate()?;

    Ok(())
}
