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
#![allow(clippy::inconsistent_digit_grouping)]

use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::UnixTimestamp;

declare_id!("JPMRGNgRk3w2pzBM1RLNBnpGxQYsFQ3yXKpuk4tTXVZ");

mod adapter;
pub mod events;
mod instructions;
mod state;
pub(crate) mod syscall;
/// Utilities used only in this crate
pub(crate) mod util;

use instructions::*;
pub use state::*;
pub use util::Invocation;

pub use adapter::{AdapterResult, PositionChange, PriceChangeInfo};

/// The maximum confidence deviation allowed for an oracle price.
///
/// The confidence is measured as the percent of the confidence interval
/// value provided by the oracle as compared to the weighted average value
/// of the price.
#[constant]
pub const MAX_ORACLE_CONFIDENCE: u16 = 5_00;

/// The maximum number of seconds since the last price was by an oracle, before
/// rejecting the price as too stale.
#[constant]
pub const MAX_ORACLE_STALENESS: i64 = 30;

/// The maximum age to allow for a quoted price for a position (seconds)
#[constant]
pub const MAX_PRICE_QUOTE_AGE: u64 = 30;

/// The maximum amount of equity that can be deducted from an account during liquidation
/// as a fraction of the total dollar value that is expected to need to be liquidated
pub const LIQUIDATION_MAX_EQUITY_LOSS_BPS: u16 = 10_00;

/// The maximum c-ratio that an account can end a liquidation with.
///
/// Note: This is not a traditional c-ratio, because it's based on the ratio of
///       the effective_collateral / required_collateral.
pub const LIQUIDATION_MAX_COLLATERAL_RATIO: u16 = 125_00;

/// The threshold at which accounts can have all their debts closed. Accounts with
/// total exposure below this value can have their exposure reduced to zero.
pub const LIQUIDATION_CLOSE_THRESHOLD_USD: u64 = 100;

/// The maximum duration in seconds of a liquidation before another user may cancel it
#[constant]
pub const LIQUIDATION_TIMEOUT: UnixTimestamp = 60;

/// The maximum number of positions that a user can register.
/// This may be exceeded by a liquidator.
pub const MAX_USER_POSITIONS: usize = 24;

/// This crate documents the instructions used in the `margin` program of the [jet-v2 repo](https://github.com/jet-lab/jet-v2/). Handler functions are described for each instruction well as struct parameters (and their types and descriptions are listed) and any handler function parameters aside from parameters that exist in every instruction handler function. Parameters of events emitted for the purposes of data logging are also included.

#[program]
pub mod jet_margin {
    use super::*;

    /// Create a new margin account for a user
    ///
    /// ## close\_account.rs
    /// 
    /// This instruction does the following:
    /// 
    /// 1.  Let `account`be a reference to the margin account being closed.
    ///     
    /// 2.  Check if the loaded margin account has any open positions.
    ///     
    ///     a.  If open positions exist, then return `ErrorCode::AccountNotEmpty`.
    ///         
    /// 3.  Emit the `AccountClosed` event for data logging (see table below).
    ///     
    /// 4.  Load the margin account.
    ///     
    /// 5.  Return `Ok(())`.
    ///     
    /// 
    /// **Parameters of close\_account.rs:**
    /// 
    /// |     |     |
    /// | --- | --- |
    /// | **Name** | **Description** |
    /// | `owner` | The owner of the account being closed. |
    /// | `receiver` | The account to get any returned rent. |
    /// | `margin_account` | The account being closed. |
    /// 
    /// **Events emitted by close\_account.rs:**
    /// 
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | `AccountClosed` | The closed account (includes the margin account pubkey). |

    pub fn close_account(ctx: Context<CloseAccount>) -> Result<()> {
        close_account_handler(ctx)
    }

    /// Register a position for some token that will be custodied by margin.
    /// Currently this applies to anything other than a claim.
    /// 
    /// ## register\_position.rs
    /// 
    /// This instruction does the following:
    /// 
    /// 1.  Register a new position that belongs to the individual margin account, allocate account space for it, and set the parameters for that asset type.
    ///     
    /// 2.  Emit the `PositionRegistered` event for data logging (see table below).
    ///     
    /// 3.  Return `Ok(())`.
    ///     
    /// 
    /// **Parameters of register\_position.rs:**
    /// 
    /// |     |     |
    /// | --- | --- |
    /// | **Name** | **Description** |
    /// | `authority` | The authority that can change the margin account. |
    /// | `payer` | The address paying for rent. |
    /// | `margin_account` | The margin account to register position type with. |
    /// | `position_token_mint` | The mint for the position token being registered. |
    /// | `metadata` | The metadata account that references the correct oracle for the token. |
    /// | `token_account` | The token account to store hold the position assets in the custody of the margin account. |
    /// | `token_program` | The token program of the token accounts to store for this margin account. |
    /// | `rent` | The rent to open the account. |
    /// | `system_program` | The system program. |
    /// 
    /// **Events emitted by register\_position.rs:**
    /// 
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | `PositionRegistered` | The position registered (includes the margin account pubkey, the authority pubkey of that margin account, and the position itself). |

    pub fn register_position(ctx: Context<RegisterPosition>) -> Result<()> {
        register_position_handler(ctx)
    }

    /// Update the balance of a position stored in the margin account to
    /// match the actual balance stored by the SPL token acount.
    /// 
    /// ## update\_position\_balance.rs
    /// 
    /// This instruction does the following:
    /// 
    /// 1.  Let `margin_account` be a mutable reference to the margin account.
    ///     
    /// 2.  Let `token_account` be a reference to the token account.
    ///     
    /// 3.  Load a margin account position and update it with `token_account`, `account`, and `balance`.
    ///     
    /// 4.  Emit the `PositionBalanceUpdated` event for data logging (see table below).
    ///     
    /// 5.  Return `Ok(())`.
    ///     
    /// 
    /// **Parameters of update\_position\_balance.rs:**
    /// 
    /// |     |     |
    /// | --- | --- |
    /// | **Name** | **Description** |
    /// | `margin_account` | The margin account to update. |
    /// | `token_account` | The token account to update the balance for. |
    /// 
    /// **Events emitted by update\_position\_balance.rs:**
    /// 
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | `PositionBalanceUpdated` | The updated position (includes the token account, margin account pubkey, and token balance). |
    /// 
    pub fn update_position_balance(ctx: Context<UpdatePositionBalance>) -> Result<()> {
        update_position_balance_handler(ctx)
    }

    /// Update the metadata for a position stored in the margin account,
    /// in the case where the metadata has changed after the position was
    /// created.
    /// 
    /// ## refresh\_position\_metadata.rs

    /// This instruction does the following:
    /// 
    /// 1.  Read account token metadata.
    ///     
    /// 2.  Load the margin account.
    ///     
    /// 3.  Update the position with refreshed metadata.
    ///     
    /// 4.  Emit the `PositionMetadataRefreshed` event for data logging (see table below).
    ///     
    /// 5.  Return `Ok(())`.
    ///     
    /// 
    /// **Parameters of refresh\_position\_metadata.rs:**
    /// 
    /// |     |     |
    /// | --- | --- |
    /// | **Name** | **Description** |
    /// | `margin_account` | The margin account with the position to be refreshed. |
    /// | `metadata` | The metadata account for the token, which has been updated. |
    /// 
    /// **Events emitted by refresh\_position\_metadata.rs:**
    /// 
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | `PositionMetadataRefreshed` | The position of which metadata was refreshed (including the margin account pubkey and the `position` itself). |

    pub fn refresh_position_metadata(ctx: Context<RefreshPositionMetadata>) -> Result<()> {
        refresh_position_metadata_handler(ctx)
    }

    /// Close out a position, freeing up space in the account.
    ///
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
    ///     a.  Return the token account.
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
    
    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        close_position_handler(ctx)
    }

    /// Verify that the account is healthy, by validating the collateralization
    /// ration is above the minimum.
    /// 
    /// ## verify\_healthy.rs
    ///
    ///  This instruction does the following:
    ///  
    ///  1.  Let `account` be the loaded margin account.
    ///      
    ///  2.  Check if all positions for that margin account are healthy.
    ///      
    ///      1.  If there are unhealthy positions exist for this margin account, return `False`.
    ///          
    ///  3.  Emit the `VerifiedHealthy` event for data logging (see table below).
    ///      
    ///  4.  Return `Ok(())`.
    ///      
    ///  
    ///  **Parameters of verify\_healthy.rs:**
    ///  
    ///  |     |     |
    ///  | --- | --- |
    ///  | **Name** | **Description** |
    ///  | `margin_account` | The account to verify the health of. |
    ///  
    ///  **Events emitted by verify\_healthy.rs:**
    ///  
    ///  |     |     |
    ///  | --- | --- |
    ///  | **Event Name** | **Description** |
    ///  | `VerifiedHealthy` | The margin account pubkeys of verified healthy accounts. |

    pub fn verify_healthy(ctx: Context<VerifyHealthy>) -> Result<()> {
        verify_healthy_handler(ctx)
    }

    /// Perform an action by invoking other programs, allowing them to alter
    /// the balances of the token accounts belonging to this margin account.
    ///
    /// /// ## adapter\_invoke.rs
    ///
    /// This instruction does the following:
    ///
    /// 1.  If a read account has the `liquidation` parameter set to a pubkey:
    ///     
    ///     a.  This means that that margin account is already under liquidation by the liquidator at that pubkey.
    ///         
    ///     b.  Return `ErrorCode::Liquidating`.
    ///         
    /// 2.  Emit the `AdapterInvokeBegin` event for data logging (see table below).
    ///     
    /// 3.  Check if any positions that have changed via adapters.
    ///     
    ///     a.  For each changed position, emit each existing adapter position as an `event` (see table below).
    ///         
    /// 4.  Emit the `AdapterInvokeEnd` event for data logging (see table below).
    ///     
    /// 5.  Verify that margin accounts positions via adapter are healthy.
    ///     
    /// 6.  Return `Ok(())`.
    ///     
    ///
    /// **Parameters of adapter\_invoke.rs:**
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Name** | **Description** |
    /// | `owner` | The authority that owns the margin account. |
    /// | `margin_account` | The margin account to proxy an action for. |
    /// | `adapter_program` | The program to be invoked. |
    /// | `adapter_metadata` | The metadata about the proxy program. |
    ///
    /// **Events emitted by adapter\_invoke.rs:**
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | `AdapterInvokeBegin` | Marks the start of the adapter invocation (includes the margin account pubkey and the adapter program pubkey). |
    /// | `event` _(Note that each single event represents a different adapter position)_ | Each adapter position is emitted as an event (includes the margin account, the adapter program, the accounts, and a value of `true` for the field `signed`. |
    /// | `AdapterInvokeEnd` | Marks the ending of the adapter invocation (includes no data except for the event itself being emitted). |

    pub fn adapter_invoke<'info>(
        ctx: Context<'_, '_, '_, 'info, AdapterInvoke<'info>>,
        data: Vec<u8>,
    ) -> Result<()> {
        adapter_invoke_handler(ctx, data)
    }

    /// Perform an action by invoking other programs, allowing them only to
    /// refresh the state of the margin account to be consistent with the actual
    /// underlying prices or positions, but not permitting new position changes.
    ///
    /// ## accounting\_invoke.rs
    /// 
    /// This instruction does the following:
    /// 
    /// 1.  Emit `AccountingInvokeBegin` events for data logging (see table below).
    ///     
    /// 2.  Check if any positions that have changed via adapters.
    ///     
    ///     a.  For each changed position, emit each existing adapter position as an `event` (see table below).
    ///         
    /// 3.  Emit `AccountingInvokeEnd` event for data logging (see table below).
    ///     
    /// 4.  Return `Ok(())`.
    ///     
    /// 
    /// **Parameters of accounting\_invoke.rs:**
    /// 
    /// |     |     |
    /// | --- | --- |
    /// | **Name** | **Description** |
    /// | `margin_account` | The margin account to proxy an action for. |
    /// | `adapter_program` | The program to be invoked. |
    /// | `adapter_metadata` | The metadata about the proxy program. |
    /// 
    /// **Events emitted by accounting\_invoke.rs:**
    /// 
    /// |     |     |
    /// | --- | --- |
    /// | **Name** | **Description** |
    /// | `AccountingInvokeBegin` | Signify that the accounting invocation process has begun. |
    /// | `event` | Each adapter position is emitted as an event (includes the margin account, the adapter program, the remaining accounts, and a value of `false` for the field `signed`. |
    /// | `AccountingInvokeEnd` | The margin account to proxy an action for. |

    pub fn accounting_invoke<'info>(
        ctx: Context<'_, '_, '_, 'info, AccountingInvoke<'info>>,
        data: Vec<u8>,
    ) -> Result<()> {
        accounting_invoke_handler(ctx, data)
    }

    /// Begin liquidating an account
    ///
    /// ## liquidate\_begin.rs
    ///
    /// This instruction does the following:
    ///
    /// 1.  Read `liquidation` and `liquidator` from the account.
    ///     
    /// 2.  Let `account` be a mutable reference to the margin account.
    ///     
    /// 3.  Verify that the account is subject to liquidation, return `False` if not.
    ///     
    /// 4.  Verify that the account is not already being liquidated.
    ///     
    ///     a.  If the liquidator is already assigned to this margin account, do nothing.
    ///         
    ///     b.  Else if there is no liquidator assigned to the unhealthy account, the liquidator can claim this unhealthy account and begin the process of liquidation.
    ///         
    ///     c.  Otherwise return `ErrorCode::Liquidating` because it is already claimed by some other liquidator.
    ///        
    /// 5.  Record the valuation of the account.
    ///     
    /// 6.  Record the minimum valuation change of the account.
    ///     
    /// 7.  Emit the `LiquidationBegun` event for data logging (see table below).
    ///     
    /// 8.  Return `Ok(())`.
    ///     
    ///
    /// **Parameters of liquidate\_begin.rs:**
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Name** | **Description** |
    /// | `margin_account` | The account in need of liquidation. |
    /// | `payer` | The address paying rent. |
    /// | `liquidator` | The liquidator account performing the liquidation. |
    /// | `liquidator_metadata` | The metadata describing the liquidator. |
    /// | `liquidation` | The account to persist the state of liquidation. |
    /// | `system_program` | The system program. |
    ///
    /// **Events emitted by liquidate\_begin.rs:**
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | `LiquidationBegun` | The event marking the beginning of liquidation (Includes the margin account pubkey, the liquidator pubkey, the liquidation pubkey, the liquidation data, and the valuation of the margin account to be liquidated). |

    pub fn liquidate_begin(ctx: Context<LiquidateBegin>) -> Result<()> {
        liquidate_begin_handler(ctx)
    }

    /// Stop liquidating an account
    /// 
    /// ## liquidate\_end.rs
    /// 
    /// This instruction does the following:
    /// 
    /// 1.  Let `account` be a mutable reference to the margin account.
    ///     
    /// 2.  Let `start_time` be the time that the liquidation on this margin account began, if it exists
    ///     
    /// 3.  Let `timed_out` be the boolean representing the type of account:
    ///     
    ///     a.  If the liquidation is timed out, then this can be any account.
    ///         
    ///     b.  If the liquidation is not timed out, then this must be the liquidator, and it must be a signer.
    ///         
    /// 4.  Check if the entity trying to end the liquidation is not the liquidator.
    ///     
    ///     a.  If not, return `ErrorCode::UnauthorizedLiquidator`.
    ///         
    /// 5.  Record the end of the liquidation.
    ///     
    /// 6.  Emit the `LiquidationEnded` event for data logging (see table below).
    ///     
    /// 7.  Return `Ok(())`.
    ///     
    /// 
    /// **Parameters of liquidate\_end.rs:**
    /// 
    /// |     |     |
    /// | --- | --- |
    /// | **Name** | **Description** |
    /// | `authority` | The pubkey calling the instruction to end liquidation. |
    /// | `margin_account` | The account in need of liquidation. |
    /// | `liquidation` | The account to persist the state of liquidation. |
    ///
    /// **Events emitted by liquidate\_end.rs:**
    /// 
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | `LiquidationEnded` | The event marking the end of liquidation (Includes the margin account pubkey, the authority of the margin account pubkey, and the timed\_out boolean that is true if the liquidation has timed out). |

    pub fn liquidate_end(ctx: Context<LiquidateEnd>) -> Result<()> {
        liquidate_end_handler(ctx)
    }

    /// Perform an action by invoking another program, for the purposes of
    /// liquidating a margin account.
    /// 
    /// ## liquidator\_invoke.rs
    /// 
    /// This instruction does the following:
    /// 
    /// 1.  Load the margin account.
    ///     
    /// 2.  Let `start_value` be the valuation of the margin account before invoking the liquidator.
    ///     
    /// 3.  Emit the `LiquidatorInvokeBegin` event for data logging (see table below).
    ///     
    /// 4.  Loop through adapter and store positions, getting and storing as `margin_account`, `adapter_program`, `accounts` and `signed`.
    ///     
    /// 5.  Emit each adapter position as an `event` (see table below).
    ///     
    /// 6.  Let`liquidation` be a mutable copy of the liquidated account.
    ///     
    /// 7.  Let `end_value` be the valuation of the margin account after the liquidation attempt, after verifying that a liquidation did occur.
    ///     
    /// 8.  Emit the `LiquidatorInvokeEnd` event for data logging (see table below).
    ///     
    /// 9.  Return `Ok(())`.
    ///         
    /// 
    /// **Parameters of liquidator\_invoke.rs:**
    /// 
    /// |     |     |
    /// | --- | --- |
    /// | **Name** | **Description** |
    /// | `liquidator` | The liquidator processing the margin account. |
    /// | `liquidation` | The account to persist the state of liquidation. |
    /// | `margin_account` | The margin account to proxy an action for. |
    /// | `adapter_program` | The program to be invoked. |
    /// | `adapter_metadata` | The metadata about the proxy program. |
    /// 
    /// **Events emitted by liquidator\_invoke.rs:**
    /// 
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | `LiquidatorInvokeBegin` | Marks the beginning of this liquidation event (includes the margin account pubkey, the adapter program pubkey, and the liquidator pubkey that is liquidating that margin account or adapter position). |
    /// | `event` _(Note that each single event represents an different adapter position)_ | Each adapter position is emitted as an event (includes the margin account, the adapter program, the accounts, and a value of `true` for the `signed` field. |
    /// | `LiquidatorInvokeEnd` | Marks the ending of this liquidator event (includes the liquidation data and the valuation of the account after liquidation has been performed). |

    pub fn liquidator_invoke<'info>(
        ctx: Context<'_, '_, '_, 'info, LiquidatorInvoke<'info>>,
        data: Vec<u8>,
    ) -> Result<()> {
        liquidator_invoke_handler(ctx, data)
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

    /// 141003
    #[msg("the current instruction was not directly invoked by the margin program")]
    IndirectInvocation,

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

    /// 141017
    #[msg("registered position owner inconsistent with PositionTokenMetadata owner or token_kind")]
    InvalidPositionOwner,

    /// 141018
    #[msg("dependencies are not satisfied to auto-register a required but unregistered position")]
    PositionNotRegisterable,

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
    UnauthorizedLiquidator = 135_040,

    /// 141041
    #[msg("attempted to extract too much value during liquidation")]
    LiquidationLostValue,
}

pub fn write_adapter_result(margin_account: &MarginAccount, result: &AdapterResult) -> Result<()> {
    let mut adapter_result_data = vec![];
    result.serialize(&mut adapter_result_data)?;
    margin_account.invocation.verify_directly_invoked()?;
    anchor_lang::solana_program::program::set_return_data(&adapter_result_data);
    Ok(())
}
