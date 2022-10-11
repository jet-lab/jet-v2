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
use anchor_spl::token::{spl_token::native_mint::ID as NATIVE_MINT_ID, Mint, Token};

use pyth_sdk_solana::state::{AccKey, AccountType, PriceAccount, ProductAccount, MAGIC, VERSION};

use crate::{
    seeds::{TOKEN_INFO, TOKEN_PYTH_PRICE, TOKEN_PYTH_PRODUCT},
    state::TokenInfo,
    util::{load_pyth_account, write_pyth_product_attributes},
};

#[derive(Accounts)]
pub struct TokenInitNative<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    #[account(address = NATIVE_MINT_ID)]
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

pub fn token_init_native_handler(
    ctx: Context<TokenInitNative>,
    oracle_authority: Pubkey,
) -> Result<()> {
    let info = &mut ctx.accounts.info;

    info.name = "SOL".to_owned();
    info.symbol = "SOL".to_owned();
    info.mint = ctx.accounts.mint.key();
    info.authority = Pubkey::default();
    info.pyth_price = ctx.accounts.pyth_price.key();
    info.pyth_product = ctx.accounts.pyth_product.key();
    info.oracle_authority = oracle_authority;
    info.max_request_amount = 0;

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
            ("base", "SOL"),
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
