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

use std::num::NonZeroU64;

use anchor_lang::prelude::ToAccountMetas;
use anchor_lang::InstructionData;
use anchor_spl::dex;
use dex::serum_dex::instruction::SelfTradeBehavior as DexSelfTradeBehavior;
use dex::serum_dex::matching::{OrderType as DexOrderType, Side as DexSide};
use jet_margin_swap::instructions::SwapDirection;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::sysvar::{rent::Rent, SysvarId};

use super::MarginPoolIxBuilder;

/// All the pubkeys that are used by a Serum V3 market
#[derive(Clone)]
pub struct SerumMarketV3 {
    pub market: Pubkey,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub request_queue: Pubkey,
    pub event_queue: Pubkey,
    pub base_token: Pubkey,
    pub quote_token: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub vault_signer: Pubkey,
}

/// Utility for creating instructions to interact with the margin
/// pools program for a specific pool.
#[derive(Clone)]
pub struct MarginSerumIxBuilder {
    pub market: SerumMarketV3,
}

impl MarginSerumIxBuilder {
    pub fn new(market: SerumMarketV3) -> Self {
        Self { market }
    }

    /// Execute a Serum swap
    pub fn serum_swap(
        &self,
        margin_account: Pubkey,
        open_orders: Pubkey,
        transit_base_account: Pubkey,
        transit_quote_account: Pubkey,
        pool_deposit_note_base: Pubkey,
        pool_deposit_note_quote: Pubkey,
        amount_in: u64,
        minimum_amount_out: u64,
        swap_direction: SwapDirection,
    ) -> Instruction {
        let pool_base = MarginPoolIxBuilder::new(self.market.base_token);
        let pool_quote = MarginPoolIxBuilder::new(self.market.quote_token);

        let accounts = jet_margin_swap::accounts::SerumSwap {
            margin_account,
            pool_deposit_note_base,
            pool_deposit_note_quote,
            transit_base_token_account: transit_base_account,
            transit_quote_token_account: transit_quote_account,
            swap_info: jet_margin_swap::accounts::SerumSwapInfo {
                market: self.market.market,
                authority: margin_account,
                open_orders,
                open_orders_authority: margin_account,
                base_vault: self.market.base_vault,
                quote_vault: self.market.quote_vault,
                request_queue: self.market.request_queue,
                event_queue: self.market.event_queue,
                market_bids: self.market.bids,
                market_asks: self.market.asks,
                vault_signer: self.market.vault_signer,
                serum_program: dex::ID,
            },
            margin_pool_base: jet_margin_swap::accounts::MarginPoolInfo {
                margin_pool: pool_base.address,
                vault: pool_base.vault,
                deposit_note_mint: pool_base.deposit_note_mint,
            },
            margin_pool_quote: jet_margin_swap::accounts::MarginPoolInfo {
                margin_pool: pool_quote.address,
                vault: pool_quote.vault,
                deposit_note_mint: pool_quote.deposit_note_mint,
            },
            margin_pool_program: jet_margin_pool::id(),
            token_program: anchor_spl::token::ID,
            rent: Rent::id(),
        }
        .to_account_metas(None);

        Instruction {
            program_id: jet_margin_swap::ID,
            data: jet_margin_swap::instruction::SerumSwap {
                amount_in,
                minimum_amount_out,
                swap_direction,
            }
            .data(),
            accounts,
        }
    }

    // /// Instruction to place a new Serum order
    // ///
    // /// # Params
    // ///
    // /// `margin_account` - The margin account with the order to be placed
    // /// `open_orders` - The margin account's Serum OpenOrders account for this market
    // /// `transit_account` - The account to transfer from the pool and to the Serum market, can be an ATA
    // /// `deposit_note` - Margin account position where pool deposit notes will be withdrawn
    // /// `order_note` - Margin account position where Serum open order notes will be minted to
    // /// `order` - Serum order parameters
    // pub fn new_order_v3(
    //     &self,
    //     margin_account: Pubkey,
    //     open_orders: Pubkey,
    //     transit_account: Pubkey,
    //     deposit_note: Pubkey,
    //     order_note: Pubkey,
    //     order: OrderParams,
    // ) -> Instruction {
    //     let (pool, order_note_mint) = match order.side {
    //         OrderSide::Bid => {
    //             let pool = MarginPoolIxBuilder::new(self.market.quote_token);
    //             (pool, self.info.quote_note_mint)
    //         }
    //         OrderSide::Ask => {
    //             let pool = MarginPoolIxBuilder::new(self.market.base_token);
    //             (pool, self.info.base_note_mint)
    //         }
    //     };
    //     let accounts = ix_accounts::NewOrderV3 {
    //         margin_account,
    //         market: self.market.market,
    //         market_info: self.info.market_info,
    //         open_orders_account: open_orders,
    //         request_queue: self.market.request_queue,
    //         event_queue: self.market.event_queue,
    //         bids: self.market.bids,
    //         asks: self.market.asks,
    //         transit_order_payer: transit_account,
    //         source_margin_pool: MarginPoolInfo {
    //             margin_pool: pool.address,
    //             vault: pool.vault,
    //             deposit_note_mint: pool.deposit_note_mint,
    //         },
    //         deposit_note,
    //         order_note,
    //         order_note_mint,
    //         base_vault: self.market.base_vault,
    //         quote_vault: self.market.quote_vault,
    //         margin_pool_program: jet_margin_pool::id(),
    //         token_program: anchor_spl::token::ID,
    //         rent: Rent::id(),
    //         serum_program: Dex::id(),
    //     }
    //     .to_account_metas(None);

    //     Instruction {
    //         program_id: jet_margin_serum::ID,
    //         data: ix_data::NewOrderV3 { order }.data(),
    //         accounts,
    //     }
    // }

    // /// Instruction to {}
    // ///
    // /// # Params
    // ///
    // /// `depositor` - The authority for the source tokens
    // pub fn cancel_order_v2(
    //     &self,
    //     margin_account: Pubkey,
    //     open_orders: Pubkey,
    //     side: u8,
    //     order_id: u128,
    // ) -> Instruction {
    //     let accounts = ix_accounts::CancelOrderV2 {
    //         margin_account,
    //         serum_program: Dex::id(),
    //         market: self.market.market,
    //         market_bids: self.market.bids,
    //         market_asks: self.market.asks,
    //         open_orders_account: open_orders,
    //         event_queue: self.market.event_queue,
    //     }
    //     .to_account_metas(None);

    //     Instruction {
    //         program_id: jet_margin_serum::ID,
    //         data: ix_data::CancelOrderV2 { side, order_id }.data(),
    //         accounts,
    //     }
    // }

    // /// Instruction to {}
    // ///
    // /// # Params
    // ///
    // /// `depositor` - The authority for the source tokens
    // pub fn close_open_orders(&self, margin_account: Pubkey, open_orders: Pubkey) -> Instruction {
    //     let accounts = ix_accounts::CloseOpenOrders {
    //         margin_account,
    //         serum_program: Dex::id(),
    //         open_orders,
    //         destination: margin_account,
    //         market: self.market.market,
    //     }
    //     .to_account_metas(None);

    //     Instruction {
    //         program_id: jet_margin_serum::ID,
    //         data: ix_data::CloseOpenOrders.data(),
    //         accounts,
    //     }
    // }

    // /// Instruction to {}
    // ///
    // /// # Params
    // ///
    // /// `depositor` - The authority for the source tokens
    // #[allow(clippy::too_many_arguments)]
    // pub fn settle_funds(
    //     &self,
    //     margin_account: Pubkey,
    //     open_orders: Pubkey,
    //     base_wallet: Pubkey,
    //     quote_wallet: Pubkey,
    //     base_note: Pubkey,
    //     quote_note: Pubkey,
    //     base_deposit_note: Pubkey,
    //     quote_deposit_note: Pubkey,
    //     base_margin_pool: &MarginPoolIxBuilder,
    //     quote_margin_pool: &MarginPoolIxBuilder,
    // ) -> Instruction {
    //     let accounts = ix_accounts::SettleFunds {
    //         margin_account,
    //         market: self.market.market,
    //         market_info: self.info.market_info,
    //         open_orders_account: open_orders,
    //         base_vault: self.market.base_vault,
    //         quote_vault: self.market.quote_vault,
    //         base_wallet,
    //         quote_wallet,
    //         base_order_note_mint: self.info.base_note_mint,
    //         quote_order_note_mint: self.info.quote_note_mint,
    //         base_note,
    //         quote_note,
    //         base_deposit_note,
    //         quote_deposit_note,
    //         base_margin_pool: MarginPoolInfo2 {
    //             margin_pool: base_margin_pool.address,
    //             vault: base_margin_pool.vault,
    //             deposit_note_mint: base_margin_pool.deposit_note_mint,
    //         },
    //         quote_margin_pool: MarginPoolInfo2 {
    //             margin_pool: quote_margin_pool.address,
    //             vault: quote_margin_pool.vault,
    //             deposit_note_mint: quote_margin_pool.deposit_note_mint,
    //         },
    //         vault_signer: self.market.vault_signer,
    //         margin_pool_program: jet_margin_pool::id(),
    //         serum_program: Dex::id(),
    //         token_program: anchor_spl::token::ID,
    //     }
    //     .to_account_metas(None);

    //     Instruction {
    //         program_id: jet_margin_serum::ID,
    //         data: ix_data::SettleFunds.data(),
    //         accounts,
    //     }
    // }

    // pub fn refresh_open_orders(
    //     &self,
    //     margin_account: Pubkey,
    //     base_oracle: Pubkey,
    //     quote_oracle: Pubkey,
    // ) -> Instruction {
    //     let accounts = ix_accounts::RefreshOpenOrders {
    //         margin_account,
    //         market_info: self.info.market_info,
    //         base_token_price_oracle: base_oracle,
    //         quote_token_price_oracle: quote_oracle,
    //     }
    //     .to_account_metas(None);

    //     Instruction {
    //         program_id: jet_margin_serum::ID,
    //         accounts,
    //         data: ix_data::RefreshOpenOrders.data(),
    //     }
    // }
}

#[derive(Clone, Debug)]
pub struct OrderParams {
    pub side: OrderSide,
    pub limit_price: NonZeroU64,
    pub max_base_qty: NonZeroU64,
    pub max_native_quote_qty_including_fees: NonZeroU64,
    pub self_trade_behavior: SelfTradeBehavior,
    pub order_type: OrderType,
    pub client_order_id: u64,
    pub limit: u16,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum OrderSide {
    Bid = 0,
    Ask = 1,
}

impl From<OrderSide> for DexSide {
    fn from(value: OrderSide) -> Self {
        match value {
            OrderSide::Bid => DexSide::Bid,
            OrderSide::Ask => DexSide::Ask,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum SelfTradeBehavior {
    DecrementTake = 0,
    CancelProvide = 1,
    AbortTransaction = 2,
}

impl From<SelfTradeBehavior> for DexSelfTradeBehavior {
    fn from(value: SelfTradeBehavior) -> Self {
        match value {
            SelfTradeBehavior::DecrementTake => DexSelfTradeBehavior::DecrementTake,
            SelfTradeBehavior::CancelProvide => DexSelfTradeBehavior::CancelProvide,
            SelfTradeBehavior::AbortTransaction => DexSelfTradeBehavior::AbortTransaction,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum OrderType {
    Limit = 0,
    ImmediateOrCancel = 1,
    PostOnly = 2,
}

impl From<OrderType> for DexOrderType {
    fn from(value: OrderType) -> Self {
        match value {
            OrderType::Limit => DexOrderType::Limit,
            OrderType::ImmediateOrCancel => DexOrderType::ImmediateOrCancel,
            OrderType::PostOnly => DexOrderType::PostOnly,
        }
    }
}
