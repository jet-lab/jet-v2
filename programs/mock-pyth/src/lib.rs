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
use bytemuck::{cast_slice_mut, from_bytes_mut, try_cast_slice_mut, Pod, Zeroable};
use std::cell::RefMut;

use borsh::{BorshDeserialize, BorshSerialize};

#[cfg(feature = "mainnet-beta")]
declare_id!("GWXu4vLvXFN87dePFvM7Ejt8HEALEG9GNmwimNKHZrXG");
#[cfg(not(feature = "mainnet-beta"))]
declare_id!("FT9EZnpdo3tPfUCGn8SBkvN9DMpSStAg3YvAqvYrtSvL");

#[program]
pub mod pyth {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, price: i64, expo: i32, conf: u64) -> Result<()> {
        let product_account = &ctx.accounts.product;

        let mut product_data = Product::load(product_account).unwrap();

        product_data.magic = MAGIC;
        product_data.ver = VERSION;
        product_data.atype = AccountType::Product;

        product_data.px_acc = *ctx.accounts.price.to_account_info().key;

        //TODO JV2M-359
        //TODO set the quote_currency to USD

        let price_account = &ctx.accounts.price;

        let mut price_data = Price::load(price_account).unwrap();

        price_data.magic = MAGIC;
        price_data.ver = VERSION;
        price_data.atype = AccountType::Price;

        price_data.agg.price = price;
        price_data.agg.conf = conf;
        price_data.agg.status = PriceStatus::Trading;

        price_data.ema_price = Rational {
            val: price,
            numer: price,
            denom: 1,
        };
        price_data.expo = expo;
        price_data.ptype = PriceType::Price;

        Ok(())
    }

    pub fn update_price(ctx: Context<UpdatePrice>, price: i64, conf: u64) -> Result<()> {
        let oracle = &ctx.accounts.price;
        let mut price_oracle = Price::load(oracle).unwrap();

        let clock = Clock::get().unwrap();

        price_oracle.agg.price = price;
        price_oracle.agg.conf = conf;
        price_oracle.agg.status = PriceStatus::Trading;
        price_oracle.agg.pub_slot = clock.slot;

        price_oracle.ema_price = Rational {
            val: price,
            numer: price,
            denom: 1,
        };
        price_oracle.ema_conf = Rational {
            val: conf as i64,
            numer: conf as i64,
            denom: 1,
        };

        price_oracle.last_slot = clock.slot;
        price_oracle.valid_slot = clock.slot;
        price_oracle.timestamp = clock.unix_timestamp;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(mut)]
    /// CHECK: Only used for testing.
    pub price: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    /// CHECK: Only used for testing.
    pub product: AccountInfo<'info>,

    #[account(mut)]
    /// CHECK: Only used for testing.
    pub price: AccountInfo<'info>,
}

pub const MAGIC: u32 = 0xa1b2c3d4;
pub const VERSION_2: u32 = 2;
pub const VERSION: u32 = VERSION_2;
pub const PROD_ACCT_SIZE: usize = 512;
pub const PROD_HDR_SIZE: usize = 48;
pub const PROD_ATTR_SIZE: usize = PROD_ACCT_SIZE - PROD_HDR_SIZE;

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct AccKey {
    pub val: [u8; 32],
}

/// The type of Pyth account determines what data it contains
#[derive(Copy, Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
#[repr(C)]
pub enum AccountType {
    Unknown,
    Mapping,
    Product,
    Price,
}

impl Default for AccountType {
    fn default() -> Self {
        AccountType::Unknown
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub enum CorpAction {
    NoCorpAct,
}

impl Default for CorpAction {
    fn default() -> Self {
        CorpAction::NoCorpAct
    }
}

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct PriceComp {
    publisher: AccKey,
    agg: PriceInfo,
    latest: PriceInfo,
}

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct PriceInfo {
    pub price: i64,
    pub conf: u64,
    pub status: PriceStatus,
    pub corp_act: CorpAction,
    pub pub_slot: u64,
}

/// Represents availability status of a price feed.
#[derive(Copy, Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
#[repr(C)]
pub enum PriceStatus {
    /// The price feed is not currently updating for an unknown reason.
    Unknown,
    /// The price feed is updating as expected.
    Trading,
    /// The price feed is not currently updating because trading in the product has been halted.
    Halted,
    /// The price feed is not currently updating because an auction is setting the price.
    Auction,
}

impl Default for PriceStatus {
    fn default() -> Self {
        PriceStatus::Unknown
    }
}

/// The type of prices associated with a product -- each product may have multiple price feeds of
/// different types.
#[derive(Copy, Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
#[repr(C)]
pub enum PriceType {
    Unknown,
    Price,
}

impl Default for PriceType {
    fn default() -> Self {
        PriceType::Unknown
    }
}

/// An number represented as both `value` and also in rational as `numer/denom`.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
#[repr(C)]
pub struct Rational {
    pub val: i64,
    pub numer: i64,
    pub denom: i64,
}

/// Price accounts represent a continuously-updating price feed for a product.
#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Price {
    /// pyth magic number
    pub magic: u32,
    /// program version
    pub ver: u32,
    /// account type
    pub atype: AccountType,
    /// price account size
    pub size: u32,
    /// price or calculation type
    pub ptype: PriceType,
    /// price exponent
    pub expo: i32,
    /// number of component prices
    pub num: u32,
    /// number of quoters that make up aggregate
    pub num_qt: u32,
    /// slot of last valid (not unknown) aggregate price
    pub last_slot: u64,
    /// valid slot-time of agg. price
    pub valid_slot: u64,
    /// exponentially moving average price
    pub ema_price: Rational,
    /// exponentially moving average confidence interval
    pub ema_conf: Rational,
    /// unix timestamp of aggregate price
    pub timestamp: i64,
    /// min publishers for valid price
    pub min_pub: u8,
    /// space for future derived values
    pub drv2: u8,
    /// space for future derived values
    pub drv3: u16,
    /// space for future derived values
    pub drv4: u32,
    /// product account key
    pub prod: AccKey,
    /// next Price account in linked list
    pub next: AccKey,
    /// valid slot of previous update
    pub prev_slot: u64,
    /// aggregate price of previous update with TRADING status
    pub prev_price: i64,
    /// confidence interval of previous update with TRADING status
    pub prev_conf: u64,
    /// unix timestamp of previous aggregate with TRADING status
    pub prev_timestamp: i64,
    /// aggregate price info
    pub agg: PriceInfo,
    /// price components one per quoter
    pub comp: [PriceComp; 32],
}

impl Price {
    #[inline]
    pub fn load<'a>(price_feed: &'a AccountInfo) -> Result<RefMut<'a, Price>> {
        let account_data = RefMut::map(price_feed.try_borrow_mut_data().unwrap(), |data| *data);
        let state = RefMut::map(account_data, |data| {
            from_bytes_mut(cast_slice_mut::<u8, u8>(try_cast_slice_mut(data).unwrap()))
        });
        Ok(state)
    }
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for Price {}

#[cfg(target_endian = "little")]
unsafe impl Pod for Price {}

/// Product accounts contain metadata for a single product, such as its symbol ("Crypto.BTC/USD")
/// and its base/quote currencies.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Product {
    /// pyth magic number
    pub magic: u32,
    /// program version
    pub ver: u32,
    /// account type
    pub atype: AccountType,
    /// price account size
    pub size: u32,
    /// first price account in list
    pub px_acc: Pubkey,
    /// key/value pairs of reference attr.
    pub attr: [u8; PROD_ATTR_SIZE],
}

impl Product {
    #[inline]
    pub fn load<'a>(product: &'a AccountInfo) -> Result<RefMut<'a, Product>> {
        let account_data = RefMut::map(product.try_borrow_mut_data().unwrap(), |data| *data);
        let state = RefMut::map(account_data, |data| {
            from_bytes_mut(cast_slice_mut::<u8, u8>(try_cast_slice_mut(data).unwrap()))
        });
        Ok(state)
    }
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for Product {}

#[cfg(target_endian = "little")]
unsafe impl Pod for Product {}
