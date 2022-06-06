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

use borsh::{
    BorshDeserialize,
    BorshSerialize,
};

#[cfg(feature = "mainnet-beta")]
declare_id!("GWXu4vLvXFN87dePFvM7Ejt8HEALEG9GNmwimNKHZrXG");
#[cfg(not(feature = "mainnet-beta"))]
declare_id!("FT9EZnpdo3tPfUCGn8SBkvN9DMpSStAg3YvAqvYrtSvL");

#[program]
pub mod pyth {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, price: i64, expo: i32, conf: u64) -> Result<()> {

        let product_account = &ctx.accounts.product;

        let mut productData = Product::load(product_account).unwrap();

        productData.magic = MAGIC;
        productData.ver = VERSION;
        productData.atype = AccountType::Product;

        productData.px_acc = *ctx.accounts.price.to_account_info().key;

        //TODO set the quote_currency to USD

        let price_account = &ctx.accounts.price;

        let mut priceData = Price::load(price_account).unwrap();

        priceData.magic = MAGIC;
        priceData.ver = VERSION;
        priceData.atype = AccountType::Price;

        priceData.agg.price = price;
        priceData.agg.conf = conf;
        priceData.agg.status = PriceStatus::Trading;

        priceData.twap = price;
        priceData.expo = expo;
        priceData.ptype = PriceType::Price;

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

        price_oracle.twap = price;

        price_oracle.curr_slot = clock.slot;
        price_oracle.valid_slot = clock.slot;

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






pub const MAGIC               : u32   = 0xa1b2c3d4;
pub const VERSION_2           : u32   = 2;
pub const VERSION             : u32   = VERSION_2;
pub const PROD_ACCT_SIZE      : usize = 512;
pub const PROD_HDR_SIZE       : usize = 48;
pub const PROD_ATTR_SIZE      : usize = PROD_ACCT_SIZE - PROD_HDR_SIZE;

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct AccKey {
    pub val: [u8; 32],
}

/// The type of Pyth account determines what data it contains
#[derive(Copy, Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
#[repr(C)]
pub enum AccountType
{
  Unknown,
  Mapping,
  Product,
  Price
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



#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Price {
    pub magic: u32,         // Pyth magic number.
    pub ver: u32,           // Program version.
    pub atype: AccountType, // Account type.
    pub size: u32,          // Price account size.
    pub ptype: PriceType,   // Price or calculation type.
    pub expo: i32,          // Price exponent.
    pub num: u32,           // Number of component prices.
    pub unused: u32,
    pub curr_slot: u64,        // Currently accumulating price slot.
    pub valid_slot: u64,       // Valid slot-time of agg. price.
    pub twap: i64,             // Time-weighted average price.
    pub avol: u64,             // Annualized price volatility.
    pub drv0: i64,             // Space for future derived values.
    pub drv1: i64,             // Space for future derived values.
    pub drv2: i64,             // Space for future derived values.
    pub drv3: i64,             // Space for future derived values.
    pub drv4: i64,             // Space for future derived values.
    pub drv5: i64,             // Space for future derived values.
    pub prod: AccKey,          // Product account key.
    pub next: AccKey,          // Next Price account in linked list.
    pub agg_pub: AccKey,       // Quoter who computed last aggregate price.
    pub agg: PriceInfo,        // Aggregate price info.
    pub comp: [PriceComp; 32], // Price components one per quoter.
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
    pub magic:  u32,
    /// program version
    pub ver:    u32,
    /// account type
    pub atype:  AccountType,
    /// price account size
    pub size:   u32,
    /// first price account in list
    pub px_acc: Pubkey,
    /// key/value pairs of reference attr.
    pub attr:   [u8; PROD_ATTR_SIZE],
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
