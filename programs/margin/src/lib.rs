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
use anchor_lang::solana_program::clock::UnixTimestamp;

declare_id!("JPMRGNgRk3w2pzBM1RLNBnpGxQYsFQ3yXKpuk4tTXVZ");

mod adapter;
mod instructions;
mod state;
/// Utilities used only in this crate
pub(crate) mod util;

use instructions::*;
pub use state::*;

pub use adapter::{AdapterResult, CompactAccountMeta, PositionChange, PriceChangeInfo};

/// The minimum collateral ratio that a margin account must maintain before
/// being subject to liquidation
#[constant]
pub const MIN_COLLATERAL_RATIO: u16 = 12_500;

/// The target c-ratio for a liquidation
#[constant]
pub const IDEAL_LIQUIDATION_COLLATERAL_RATIO: u16 = 13_000;

/// The maximum c-ratio that a liquidator is allowed to increase a margin account to
#[constant]
pub const MAX_LIQUIDATION_COLLATERAL_RATIO: u16 = 15_000;

/// The maximum confidence deviation allowed for an oracle price.
///
/// The confidence is measured as the percent of the confidence interval
/// value provided by the oracle as compared to the weighted average value
/// of the price.
#[constant]
pub const MAX_ORACLE_CONFIDENCE: u16 = 500;

/// The maximum number of seconds since the last price was by an oracle, before
/// rejecting the price as too stale.
pub const MAX_ORACLE_STALENESS: i64 = 10;

/// The maximum age to allow for a quoted price for a position (seconds)
#[constant]
pub const MAX_PRICE_QUOTE_AGE: u64 = 10;

/// The maximum loss of value to a margin account allowed during a liquidation.
/// This is the ratio of value lost to the value that should be liquidated,
/// represented in basis points
/// This applies to the arbitrary liquidate_invoke steps executed by a liquidator.
/// This does not include the liquidation fees, which would result in additional value extracted
#[constant]
pub const MAX_LIQUIDATION_VALUE_SLIPPAGE: u16 = 500;

/// The maximum reduction in c-ratio allowed during a liquidation, in bps
#[constant]
pub const MAX_LIQUIDATION_C_RATIO_SLIPPAGE: u16 = 500;

/// The maximum duration in seconds of a liquidation before another user may cancel it
#[constant]
pub const LIQUIDATION_TIMEOUT: UnixTimestamp = 60;

#[program]
pub mod jet_margin {
    use super::*;

    /// Create a new margin account for a user
    pub fn create_account(ctx: Context<CreateAccount>, seed: u16) -> Result<()> {
        create_account_handler(ctx, seed)
    }

    /// Close a user's margin account
    pub fn close_account(ctx: Context<CloseAccount>) -> Result<()> {
        close_account_handler(ctx)
    }

    /// Register a position for some token type to be stored by a margin account
    pub fn register_position(ctx: Context<RegisterPosition>) -> Result<()> {
        register_position_handler(ctx)
    }

    /// Update the balance of a position stored in the margin account to
    /// match the actual balance stored by the SPL token acount.
    pub fn update_position_balance(ctx: Context<UpdatePositionBalance>) -> Result<()> {
        update_position_balance_handler(ctx)
    }

    /// Close out a position, freeing up space in the account.
    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        close_position_handler(ctx)
    }

    /// Verify that the account is healthy, by validating the collateralization
    /// ration is above the minimum.
    pub fn verify_healthy(ctx: Context<VerifyHealthy>) -> Result<()> {
        verify_healthy_handler(ctx)
    }

    /// Perform an action by invoking other programs, allowing them to alter
    /// the balances of the token accounts belonging to this margin account.
    pub fn adapter_invoke<'info>(
        ctx: Context<'_, '_, '_, 'info, AdapterInvoke<'info>>,
        account_metas: Vec<CompactAccountMeta>,
        data: Vec<u8>,
    ) -> Result<()> {
        adapter_invoke_handler(ctx, account_metas, data)
    }

    /// Perform an action by invoking other programs, allowing them only to
    /// refresh the state of the margin account to be consistent with the actual
    /// underlying prices or positions, but not permitting new position changes.
    pub fn accounting_invoke<'info>(
        ctx: Context<'_, '_, '_, 'info, AccountingInvoke<'info>>,
        account_metas: Vec<CompactAccountMeta>,
        data: Vec<u8>,
    ) -> Result<()> {
        accounting_invoke_handler(ctx, account_metas, data)
    }

    /// Begin liquidating an account
    pub fn liquidate_begin(ctx: Context<LiquidateBegin>) -> Result<()> {
        liquidate_begin_handler(ctx)
    }

    /// Stop liquidating an account
    pub fn liquidate_end(ctx: Context<LiquidateEnd>) -> Result<()> {
        liquidate_end_handler(ctx)
    }

    /// Perform an action by invoking another program, for the purposes of
    /// liquidating a margin account.
    pub fn liquidator_invoke<'info>(
        ctx: Context<'_, '_, '_, 'info, LiquidatorInvoke<'info>>,
        account_metas: Vec<CompactAccountMeta>,
        data: Vec<u8>,
    ) -> Result<()> {
        liquidator_invoke_handler(ctx, account_metas, data)
    }
}

#[error_code]
pub enum ErrorCode {
    /// 141000 - An adapter did not return anything
    NoAdapterResult = 135_000,

    /// 141001
    #[msg("The program that set the result was not the adapter")]
    WrongProgramAdapterResult = 135_001,

    /// 141002
    #[msg("this invocation is not authorized by the necessary accounts")]
    UnauthorizedInvocation,

    /// 141010 - Account cannot record any additional positions
    #[msg("account cannot record any additional positions")]
    MaxPositions = 135_010,

    /// 141011 - Account has no record of the position
    #[msg("account has no record of the position")]
    UnknownPosition,

    /// 141012 - Attempting to close a position that has a balance
    #[msg("attempting to close a position that has a balance")]
    CloseNonZeroPosition,

    /// 141013 - Attempting to re-register a position
    #[msg("attempting to register an existing position")]
    PositionAlreadyRegistered,

    /// 141014 - Attempting to close a margin account that isn't empty
    #[msg("attempting to close non-empty margin account")]
    AccountNotEmpty,

    /// 141015 - Attempting to use a position not registered by the account
    #[msg("attempting to use unregistered position")]
    PositionNotRegistered,

    /// 141016 - Attempting to close a position that is required by the adapter
    #[msg("attempting to close a position that is required by the adapter")]
    CloseRequiredPosition,

    /// 141020 - The adapter providing a position change is not authorized for this asset
    #[msg("wrong adapter to modify the position")]
    InvalidPositionAdapter = 135_020,

    /// 141021 - A position price is not up-to-date
    #[msg("a position price is outdated")]
    OutdatedPrice,

    /// 141022 - An asset has an invalid price.
    #[msg("an asset price is currently invalid")]
    InvalidPrice,

    /// 141023 - A position balance is not up-to-date
    #[msg("a position balance is outdated")]
    OutdatedBalance,

    /// 141030 - The account is not healthy
    #[msg("the account is not healthy")]
    Unhealthy = 135_030,

    /// 141031 - The account is already healthy
    #[msg("the account is already healthy")]
    Healthy,

    /// 141032 - The account is being liquidated
    #[msg("the account is being liquidated")]
    Liquidating,

    /// 141033 - The account is not being liquidated
    #[msg("the account is not being liquidated")]
    NotLiquidating,

    /// 141034 - The account has stale positions
    StalePositions,

    /// 141040 - No permission to perform a liquidation action
    #[msg("the liquidator does not have permission to do this")]
    UnauthorizedLiquidator,

    /// 141041
    #[msg("attempted to extract too much value during liquidation")]
    LiquidationLostValue,

    /// 141042
    #[msg("reduced the c-ratio too far during liquidation")]
    LiquidationUnhealthy,

    /// 141043
    #[msg("increased the c-ratio too high during liquidation")]
    LiquidationTooHealthy,
}

pub fn write_adapter_result(result: &AdapterResult) -> Result<()> {
    let mut adapter_result_data = vec![0u8; 512];
    result.serialize(&mut &mut adapter_result_data[..])?;

    anchor_lang::solana_program::program::set_return_data(&adapter_result_data);
    Ok(())
}
