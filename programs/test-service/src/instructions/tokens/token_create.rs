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
use anchor_spl::token::{Mint, Token};

use pyth_sdk_solana::state::{AccKey, AccountType, PriceAccount, ProductAccount, MAGIC, VERSION};

use crate::{
    seeds::{TOKEN_INFO, TOKEN_MINT, TOKEN_PYTH_PRICE, TOKEN_PYTH_PRODUCT},
    state::TokenInfo,
    util::{load_pyth_account, write_pyth_product_attributes},
};

#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone)]
pub struct TokenCreateParams {
    /// The symbol string for the token
    pub symbol: String,

    /// The name or description of the token
    ///
    /// Used to derive the mint address
    pub name: String,

    /// The decimals for the mint
    pub decimals: u8,

    /// The authority over the token
    pub authority: Pubkey,

    /// The authority to set prices
    pub oracle_authority: Pubkey,

    /// The maximum amount of the token a user can request to mint in a
    /// single instruction.
    pub max_amount: u64,

    /// the symbol of the mainnet product from which the price will be derived
    pub source_symbol: String,

    /// multiplied by the mainnet price to get the price of this asset
    pub price_ratio: f64,
}

#[derive(Accounts)]
#[instruction(params: TokenCreateParams)]
pub struct TokenCreate<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    #[account(init,
              seeds = [
                TOKEN_MINT,
                params.name.as_bytes()
              ],
              bump,
              mint::decimals = params.decimals,
              mint::authority = info,
              payer = payer)]
    mint: Account<'info, Mint>,

    #[account(init,
              seeds = [
                TOKEN_INFO,
                mint.key().as_ref()
              ],
              bump,
              space = TokenInfo::SIZE,
              payer = payer
    )]
    info: Box<Account<'info, TokenInfo>>,

    #[account(init,
              seeds = [
                TOKEN_PYTH_PRICE,
                mint.key().as_ref()
              ],
              bump,
              space = std::mem::size_of::<PriceAccount>(),
              payer = payer
    )]
    pyth_price: AccountInfo<'info>,

    #[account(init,
              seeds = [
                TOKEN_PYTH_PRODUCT,
                mint.key().as_ref()
              ],
              bump,
              space = std::mem::size_of::<ProductAccount>(),
              payer = payer
    )]
    pyth_product: AccountInfo<'info>,

    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

pub fn token_create_handler(ctx: Context<TokenCreate>, params: TokenCreateParams) -> Result<()> {
    let info = &mut ctx.accounts.info;

    info.bump_seed = *ctx.bumps.get("info").unwrap();
    info.symbol = params.symbol.clone();
    info.name = params.name;
    info.mint = ctx.accounts.mint.key();
    info.authority = params.authority;
    info.pyth_price = ctx.accounts.pyth_price.key();
    info.pyth_product = ctx.accounts.pyth_product.key();
    info.oracle_authority = params.oracle_authority;
    info.max_request_amount = params.max_amount;
    info.source_symbol = params.source_symbol.clone();
    info.price_ratio = params.price_ratio;

    let mut pyth_product = load_pyth_account::<ProductAccount>(&ctx.accounts.pyth_product)?;
    let mut pyth_price = load_pyth_account::<PriceAccount>(&ctx.accounts.pyth_price)?;

    pyth_product.magic = MAGIC;
    pyth_product.ver = VERSION;
    pyth_product.atype = AccountType::Product as u32;
    pyth_product.px_acc = AccKey {
        val: ctx.accounts.pyth_price.key().to_bytes(),
    };

    write_pyth_product_attributes(
        &mut pyth_product.attr,
        &[
            ("asset_type", "Crypto"),
            ("quote_currency", "USD"),
            ("base", params.symbol.as_str()),
        ],
    );

    pyth_price.magic = MAGIC;
    pyth_price.ver = VERSION;
    pyth_price.atype = AccountType::Price as u32;
    pyth_price.prod = AccKey {
        val: ctx.accounts.pyth_product.key().to_bytes(),
    };

    Ok(())
}
