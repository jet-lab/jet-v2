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

use anchor_spl::dex;
use anchor_spl::dex::serum_dex::matching::{OrderType, Side};
use anchor_spl::dex::serum_dex::state::MarketState;
use anchor_spl::dex::serum_dex::{instruction::SelfTradeBehavior, state::OpenOrders};
use anchor_spl::token::Token;
use jet_margin_pool::ChangeKind;
use jet_proto_math::Number128;

use crate::*;

#[derive(Accounts)]
pub struct SerumSwap<'info> {
    /// The margin account being executed on
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The base token account for source deposit to be exchanged from/into
    /// CHECK: The account is validated by the spl_token program on mint/burn
    #[account(mut)]
    pub pool_deposit_note_base: AccountInfo<'info>,

    /// The quote token account for source deposit to be exchanged from/into
    /// CHECK: The account is validated by the spl_token program on mint/burn
    #[account(mut)]
    pub pool_deposit_note_quote: AccountInfo<'info>,

    /// Temporary SPL account to send/receive base tokens
    /// CHECK: The account is validated by the spl_token program on mint/burn
    #[account(mut)]
    pub transit_base_token_account: AccountInfo<'info>,

    /// Temporary SPL account to send/receive quote tokens
    /// CHECK: The account is validated by the spl_token program on mint/burn
    #[account(mut)]
    pub transit_quote_token_account: AccountInfo<'info>,

    /// The accounts relevant to the swap pool used for the exchange
    pub swap_info: SerumSwapInfo<'info>,

    /// The accounts relevant to the base margin pool (e.g. SOL)
    pub margin_pool_base: MarginPoolInfo<'info>,

    /// The accounts relevant to the quote margin pool (e.g. USDC)
    pub margin_pool_quote: MarginPoolInfo<'info>,

    pub margin_pool_program: Program<'info, JetMarginPool>,

    pub token_program: Program<'info, Token>,

    pub rent: Sysvar<'info, Rent>,
}

impl<'info> SerumSwap<'info> {
    /// Withdraw from the base pool, when selling (SwapDirection::Ask)
    fn withdraw_base_source_context(&self) -> CpiContext<'_, '_, '_, 'info, Withdraw<'info>> {
        CpiContext::new(
            self.margin_pool_program.to_account_info(),
            Withdraw {
                margin_pool: self.margin_pool_base.margin_pool.to_account_info(),
                vault: self.margin_pool_base.vault.to_account_info(),
                deposit_note_mint: self.margin_pool_base.deposit_note_mint.to_account_info(),
                depositor: self.margin_account.to_account_info(),
                source: self.pool_deposit_note_base.to_account_info(),
                destination: self.transit_base_token_account.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    fn withdraw_quote_source_context(&self) -> CpiContext<'_, '_, '_, 'info, Withdraw<'info>> {
        CpiContext::new(
            self.margin_pool_program.to_account_info(),
            Withdraw {
                margin_pool: self.margin_pool_quote.margin_pool.to_account_info(),
                vault: self.margin_pool_quote.vault.to_account_info(),
                deposit_note_mint: self.margin_pool_quote.deposit_note_mint.to_account_info(),
                depositor: self.margin_account.to_account_info(),
                source: self.pool_deposit_note_quote.to_account_info(),
                destination: self.transit_quote_token_account.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    fn new_order_v3_context(
        &self,
        swap_direction: SwapDirection,
    ) -> CpiContext<'_, '_, '_, 'info, dex::NewOrderV3<'info>> {
        // Use the correct account depending on the swap direction
        let order_payer_token_account = match swap_direction {
            SwapDirection::Bid => self.transit_quote_token_account.to_account_info(),
            SwapDirection::Ask => self.transit_base_token_account.to_account_info(),
        };
        CpiContext::new(
            self.swap_info.serum_program.to_account_info(),
            dex::NewOrderV3 {
                market: self.swap_info.market.to_account_info(),
                open_orders: self.swap_info.open_orders.to_account_info(),
                request_queue: self.swap_info.request_queue.to_account_info(),
                event_queue: self.swap_info.event_queue.to_account_info(),
                market_bids: self.swap_info.market_bids.to_account_info(),
                market_asks: self.swap_info.market_asks.to_account_info(),
                order_payer_token_account,
                open_orders_authority: self.swap_info.open_orders_authority.to_account_info(),
                coin_vault: self.swap_info.base_vault.to_account_info(),
                pc_vault: self.swap_info.quote_vault.to_account_info(),
                token_program: self.token_program.to_account_info(),
                rent: self.rent.to_account_info(),
            },
        )
    }

    fn settle_funds_context(&self) -> CpiContext<'_, '_, '_, 'info, dex::SettleFunds<'info>> {
        CpiContext::new(
            self.swap_info.serum_program.to_account_info(),
            dex::SettleFunds {
                market: self.swap_info.market.to_account_info(),
                open_orders: self.swap_info.open_orders.to_account_info(),
                open_orders_authority: self.swap_info.open_orders_authority.to_account_info(),
                coin_vault: self.swap_info.base_vault.to_account_info(),
                pc_vault: self.swap_info.quote_vault.to_account_info(),
                coin_wallet: self.transit_base_token_account.to_account_info(),
                pc_wallet: self.transit_quote_token_account.to_account_info(),
                vault_signer: self.swap_info.vault_signer.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    fn deposit_base_destination_context(&self) -> CpiContext<'_, '_, '_, 'info, Deposit<'info>> {
        CpiContext::new(
            self.margin_pool_program.to_account_info(),
            Deposit {
                margin_pool: self.margin_pool_base.margin_pool.to_account_info(),
                vault: self.margin_pool_base.vault.to_account_info(),
                deposit_note_mint: self.margin_pool_base.deposit_note_mint.to_account_info(),
                depositor: self.margin_account.to_account_info(),
                source: self.transit_base_token_account.to_account_info(),
                destination: self.pool_deposit_note_base.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    fn deposit_quote_destination_context(&self) -> CpiContext<'_, '_, '_, 'info, Deposit<'info>> {
        CpiContext::new(
            self.margin_pool_program.to_account_info(),
            Deposit {
                margin_pool: self.margin_pool_quote.margin_pool.to_account_info(),
                vault: self.margin_pool_quote.vault.to_account_info(),
                deposit_note_mint: self.margin_pool_quote.deposit_note_mint.to_account_info(),
                depositor: self.margin_account.to_account_info(),
                source: self.transit_quote_token_account.to_account_info(),
                destination: self.pool_deposit_note_quote.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct SerumSwapInfo<'info> {
    /// The Serum market to place the swap in
    /// CHECK:
    pub market: AccountInfo<'info>,

    /// CHECK:
    pub authority: UncheckedAccount<'info>,

    /// CHECK:
    pub open_orders: UncheckedAccount<'info>,

    /// This would be the margin account, however the liquidator can also
    /// invoke this instruction, and it would use its own open orders accounts
    /// per market, and thus should be able to sign as the authority.
    /// CHECK:
    pub open_orders_authority: UncheckedAccount<'info>,

    /// CHECK:
    pub base_vault: UncheckedAccount<'info>,

    /// CHECK:
    pub quote_vault: UncheckedAccount<'info>,

    /// CHECK:
    pub request_queue: UncheckedAccount<'info>,

    /// CHECK:
    pub event_queue: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub market_bids: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub market_asks: UncheckedAccount<'info>,

    /// CHECK:
    pub vault_signer: AccountInfo<'info>,

    pub serum_program: Program<'info, dex::Dex>,
}

/// Swap tokens by executing a market trade on Serum.
///
/// The instruction preserves the amount of tokens in the transit accounts before
/// the swap occurs.
/// Consider the below scenario:
///
/// * User has 1000b and 10q tokens in their transit account, and wants to swap 2000q
/// * User withdraws 2000q from a pool
/// * User swaps and settles funds, receives 900b for 1960q
/// * User deposits the above tokens, remaining with 1000b and 10q
///
/// | Action Taken | Base   | Quote  |
/// |--------------|--------|--------|
/// |              |   1000 |     10 |
/// | Withdrawal   |   1000 |   2010 |
/// | Settle Funds |   1900 |     50 |
/// | Deposit      |   1000 |     10 |
///
pub fn serum_swap_handler(
    ctx: Context<SerumSwap>,
    amount_in: u64,
    minimum_amount_out: u64,
    swap_direction: SwapDirection,
) -> Result<()> {
    // Get the balance of tokens before the trade.
    // If the transit token accounts have balances, these balances will be preserved.
    // This protects a swapping user (e.g. liquidator) by preserving their tokens during the swap.
    let base_before = token::accessor::amount(&ctx.accounts.transit_base_token_account)?;
    let quote_before = token::accessor::amount(&ctx.accounts.transit_quote_token_account)?;

    // Withdraw tokens from the pool into the transit accounts
    jet_margin_pool::cpi::withdraw(
        match swap_direction {
            SwapDirection::Bid => ctx.accounts.withdraw_quote_source_context(),
            SwapDirection::Ask => ctx.accounts.withdraw_base_source_context(),
        },
        ChangeKind::ShiftBy,
        amount_in,
    )?;

    // Check the number of tokens withdrawn
    let base_after_withdrawal = token::accessor::amount(&ctx.accounts.transit_base_token_account)?;
    let quote_after_withdrawal =
        token::accessor::amount(&ctx.accounts.transit_quote_token_account)?;

    // Get market parameters
    let (base_lot_size, quote_lot_size) = {
        let market_info = ctx.accounts.swap_info.market.to_account_info();
        let market = MarketState::load(&market_info, &dex::ID).map_err(ProgramError::from)?;
        (market.coin_lot_size, market.pc_lot_size)
    };

    // Build order parameters
    let (side, limit_price, max_coin_qty, max_native_pc_qty) = match swap_direction {
        SwapDirection::Ask => {
            // Purchase as much of the quote as possible for the given base
            let max_coin_qty = amount_in.checked_div(base_lot_size).unwrap();
            let max_coin_qty = NonZeroU64::new(max_coin_qty).unwrap();
            let max_native_pc_qty = NonZeroU64::new(u64::MAX).unwrap();
            (
                Side::Ask,
                NonZeroU64::new(1).unwrap(),
                max_coin_qty,
                max_native_pc_qty,
            )
        }
        SwapDirection::Bid => {
            // Purchase as much of the base as possible for the given quote
            let max_coin_qty = NonZeroU64::new(u64::MAX).unwrap();
            let max_native_pc_qty = amount_in.checked_div(quote_lot_size).unwrap();
            let max_native_pc_qty = NonZeroU64::new(max_native_pc_qty).unwrap();
            (
                Side::Bid,
                NonZeroU64::new(u64::MAX).unwrap(),
                max_coin_qty,
                max_native_pc_qty,
            )
        }
    };

    dex::new_order_v3(
        ctx.accounts.new_order_v3_context(swap_direction),
        side,
        limit_price,
        max_coin_qty,
        max_native_pc_qty,
        SelfTradeBehavior::DecrementTake,
        OrderType::ImmediateOrCancel,
        // We do not need to cancel orders, so a static ID is fine
        0,
        u16::MAX,
    )?;

    // Settle funds
    dex::settle_funds(ctx.accounts.settle_funds_context())?;

    let base_after = token::accessor::amount(&ctx.accounts.transit_base_token_account)?;
    let quote_after = token::accessor::amount(&ctx.accounts.transit_quote_token_account)?;

    // If bidding, quote will decrease and base increase.
    let (tokens_sold, tokens_bought) = match swap_direction {
        SwapDirection::Bid => {
            // Increasing, after >= before
            let base = base_after.checked_sub(base_after_withdrawal).unwrap();
            let quote = quote_after_withdrawal.checked_sub(quote_after).unwrap();
            (quote, base)
        }
        SwapDirection::Ask => {
            // Decreasing, before >= after
            let base = base_after_withdrawal.checked_sub(base_after).unwrap();
            let quote = quote_after.checked_sub(quote_after_withdrawal).unwrap();
            (base, quote)
        }
    };

    // If the base or quote deltas are 0, fail transaction
    if tokens_bought == 0 || tokens_sold == 0 {
        return err!(SwapError::SwapDidNotComplete);
    }

    // Check slippage using a ratio of swapped tokens
    let (expected_rate, actual_rate) = match swap_direction {
        SwapDirection::Bid => (
            Number128::from_decimal(minimum_amount_out, 0) / Number128::from_decimal(amount_in, 0),
            Number128::from_decimal(tokens_bought, 0) / Number128::from_decimal(tokens_sold, 0),
        ),
        SwapDirection::Ask => (
            Number128::from_decimal(amount_in, 0) / Number128::from_decimal(minimum_amount_out, 0),
            Number128::from_decimal(tokens_sold, 0) / Number128::from_decimal(tokens_bought, 0),
        ),
    };

    if actual_rate < expected_rate {
        msg!(
            "Exceeded the maximum slippage, minimum rate {}, actual rate {}",
            expected_rate,
            actual_rate
        );
        return err!(SwapError::ExceededSlippage);
    }

    // Deposit excess tokens to destination pools. If the trade was partially filled,
    // there will be tokens on both accounts.
    let base_deposit = base_after.saturating_sub(base_before);
    let quote_deposit = quote_after.saturating_sub(quote_before);

    if base_deposit > 0 {
        jet_margin_pool::cpi::deposit(
            ctx.accounts.deposit_base_destination_context(),
            ChangeKind::ShiftBy,
            base_deposit,
        )?;
    }

    if quote_deposit > 0 {
        jet_margin_pool::cpi::deposit(
            ctx.accounts.deposit_quote_destination_context(),
            ChangeKind::ShiftBy,
            quote_deposit,
        )?;
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, AnchorSerialize, AnchorDeserialize)]
#[repr(u8)]
pub enum SwapDirection {
    Bid = 0,
    Ask = 1,
}

#[derive(Accounts)]
pub struct InitOpenOrders<'info> {
    /// The owner of the open orders account, expected to be the margin account
    /// or a liquidator.
    pub owner: Signer<'info>,

    /// CHECK: The account is validated by `serum_dex::init_open_orders`
    #[account(mut)]
    pub market: AccountInfo<'info>,

    /// The address paying for rent
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: New account that is checked and owned by Serum
    #[account(
        init,
        seeds = [
            owner.key().as_ref(),
            market.key().as_ref(),
            b"open_orders",
        ],
        bump,
        payer = payer,
        owner = dex::Dex::id(),
        space = 12 + std::mem::size_of::<OpenOrders>(), // serum padding = 12
    )]
    pub open_orders: AccountInfo<'info>,

    pub serum_program: Program<'info, dex::Dex>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitOpenOrders<'info> {
    fn init_open_oders_context(&self) -> CpiContext<'_, '_, '_, 'info, dex::InitOpenOrders<'info>> {
        CpiContext::new(
            self.serum_program.to_account_info(),
            dex::InitOpenOrders {
                open_orders: self.open_orders.to_account_info(),
                authority: self.owner.to_account_info(),
                market: self.market.to_account_info(),
                rent: self.rent.to_account_info(),
            },
        )
    }
}

pub fn init_open_orders_handler(ctx: Context<InitOpenOrders>) -> Result<()> {
    dex::init_open_orders(ctx.accounts.init_open_oders_context())
}

#[derive(Accounts)]
pub struct CloseOpenOrders<'info> {
    /// The owner of the open orders account, expected to be the margin account
    /// or a liquidator.
    #[account(signer)]
    pub owner: AccountLoader<'info, MarginAccount>,

    /// CHECK: New account that is checked and owned by Serum
    #[account(
        seeds = [
            owner.key().as_ref(),
            market.key().as_ref(),
            b"open_orders",
        ],
        bump,
    )]
    pub open_orders: AccountInfo<'info>,

    /// CHECK: The account is validated by `serum_dex::init_open_orders`
    #[account(mut)]
    pub market: AccountInfo<'info>,

    /// The destination account to send SOL to
    /// CHECK: Account only needs to be able to receive SOL
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,

    pub serum_program: Program<'info, dex::Dex>,
}

impl<'info> CloseOpenOrders<'info> {
    fn close_open_oders_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, dex::CloseOpenOrders<'info>> {
        CpiContext::new(
            self.serum_program.to_account_info(),
            dex::CloseOpenOrders {
                open_orders: self.open_orders.to_account_info(),
                authority: self.owner.to_account_info(),
                market: self.market.to_account_info(),
                destination: self.destination.to_account_info(),
            },
        )
    }
}

pub fn close_open_orders_handler(ctx: Context<CloseOpenOrders>) -> Result<()> {
    dex::close_open_orders(ctx.accounts.close_open_oders_context())
}
