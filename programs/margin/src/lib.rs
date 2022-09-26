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

/// This crate documents the instructions used in the `margin` program of the
/// [jet-v2 repo](https://github.com/jet-lab/jet-v2/).
///
/// Handler functions are described for each instruction well as struct parameters
/// (and their types and descriptions are listed) and any handler function
/// parameters aside from parameters that exist in every instruction handler function.
///
/// Accounts associated with events emitted for the purposes of data logging are also included.

#[program]
pub mod jet_margin {
    use super::*;

    /// Create a new margin account for a user
    ///
    /// This instruction does the following:
    ///
    /// 1.  Create and load the margin account.
    ///     
    /// 2.  Initialize the margin account.
    ///     
    /// 3.  Emit the [`events::AccountCreated`] event (see Events table below) for data logging.
    ///     
    /// 4.  Return `Ok(())`.
    ///     
    ///
    /// **[Accounts](jet_margin::accounts::CreateAccount) expected with create\_account.rs:**
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `owner` | `signer` | The owner of the new margin account. |
    /// | `payer` | `signer` | The pubkey paying rent for the new margin account opening. |
    /// | `margin_account` | `writable` | The margin account to initialize for the owner. |
    /// | `system_program` | `read only` | The [system native program](https://docs.solana.com/developing/runtime-facilities/programs#system-program). |
    ///
    /// **Events emitted by create\_account.rs:**
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::AccountCreated`] | The created account. |

    pub fn create_account(ctx: Context<CreateAccount>, seed: u16) -> Result<()> {
        create_account_handler(ctx, seed)
    }

    /// Close a user's margin account
    ///
    /// This instruction does the following:
    ///
    /// 1.  Load the margin account.
    ///     
    /// 2.  Check if the loaded margin account has any open positions.
    ///     
    ///     a.  If open positions exist, then return [`ErrorCode::AccountNotEmpty`].
    ///         
    /// 3.  Emit the [`events::AccountClosed`] event (see Events table below) for data logging.
    ///
    /// 4. Close the account and return the rent to the receiver.
    ///     
    /// 5.  Return `Ok(())`.
    ///     
    ///
    /// **[Accounts](jet_margin::accounts::CloseAccount) expected with close\_account.rs:**
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `owner` | `signer` | The owner of the account being closed. |
    /// | `receiver` | `writable` | The account to get any returned rent. |
    /// | `margin_account` | `writable` | The account being closed. |
    ///
    /// **Events emitted by close\_account.rs:**
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::AccountClosed`] | The closed account. |

    pub fn close_account(ctx: Context<CloseAccount>) -> Result<()> {
        close_account_handler(ctx)
    }

    /// Register a position for some token that will be custodied by margin.
    /// Currently this applies to anything other than a claim.
    ///
    /// This instruction does the following:
    ///
    /// 1.  Register a new position that belongs to the individual margin account.
    ///     
    /// 2.  Emit the [`events::PositionRegistered`] event (see Events table below) for data logging.
    ///     
    /// 3.  Return `Ok(())`.
    ///     
    ///
    /// **[Accounts](jet_margin::accounts::RegisterPosition) expected with register\_position.rs:**
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Description** |
    /// | `authority` | `signer` | The authority that can change the margin account. |
    /// | `payer` | `signer` | The address paying for rent. |
    /// | `margin_account` | `writable` |  The margin account to register position type with. |
    /// | `position_token_mint` | `read only` | The mint for the position token being registered. |
    /// | `metadata` | `read only` | The metadata account that references the correct oracle for the token. |
    /// | `token_account` | `writable` | The token account to store hold the position assets in the custody of the margin account. |
    /// | `token_program` | `read only` | The [spl token program](https://spl.solana.com/token). |
    /// | `rent` | `read only` | The [rent sysvar](https://docs.solana.com/developing/runtime-facilities/sysvars#rent). The rent to open the account. |
    /// | `system_program` | `read only` | The [system native program](https://docs.solana.com/developing/runtime-facilities/programs#system-program). |
    ///
    /// **Events emitted by register\_position.rs:**
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::PositionRegistered`] | The position registered. |

    pub fn register_position(ctx: Context<RegisterPosition>) -> Result<()> {
        register_position_handler(ctx)
    }

    /// Update the balance of a position stored in the margin account to
    /// match the actual balance stored by the SPL token acount.
    ///
    /// This instruction does the following:
    ///
    /// 1.  Load the margin account.
    ///     
    /// 2.  Load the token account.
    ///     
    /// 3.  Update the margin account position.
    ///     
    /// 4.  Emit the [`events::PositionBalanceUpdated`] event (see Events table below) for data logging.
    ///     
    /// 5.  Return `Ok(())`.
    ///     
    ///
    /// **[Accounts](jet_margin::accounts::UpdatePositionBalance) expected with update\_position\_balance.rs:**
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | | **Type** | **Description** |
    /// | `margin_account` | `writable` | The margin account to update. |
    /// | `token_account` | `read only` | The token account to update the balance for. |
    ///
    /// **Events emitted by update\_position\_balance.rs:**
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::PositionBalanceUpdated`] | The updated position. |
    ///
    pub fn update_position_balance(ctx: Context<UpdatePositionBalance>) -> Result<()> {
        update_position_balance_handler(ctx)
    }

    /// Update the metadata for a position stored in the margin account,
    /// in the case where the metadata has changed after the position was
    /// created.
    ///
    /// This instruction does the following:
    ///
    /// 1.  Load account token metadata.
    ///     
    /// 2.  Load the margin account.
    ///     
    /// 3.  Update the position with refreshed metadata.
    ///     
    /// 4.  Emit the [`events::PositionMetadataRefreshed`] event (see Events table below) for data logging.
    ///     
    /// 5.  Return `Ok(())`.
    ///     
    ///
    /// **[Accounts](jet_margin::accounts::RefreshPositionMetadata) expected with refresh\_position\_metadata.rs:**
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_account` | `writable` | The margin account with the position to be refreshed. |
    /// | `metadata` | `read only` | The metadata account for the token, which has been updated. |
    ///
    /// **Events emitted by refresh\_position\_metadata.rs:**
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::PositionMetadataRefreshed`] | The position of which metadata was refreshed. |

    pub fn refresh_position_metadata(ctx: Context<RefreshPositionMetadata>) -> Result<()> {
        refresh_position_metadata_handler(ctx)
    }

    /// Close out a position, freeing up space in the account.
    ///
    /// This instruction does the following:
    ///
    /// 1.  Load the margin account.
    ///
    /// 2.  Verify the authority of the margin account.
    ///
    /// 3.  Unregister the existing position from the margin account.
    ///
    /// 4.  If the token account authority is the same as the margin account authority, close the token account.
    ///
    /// 5.  Emit the [`events::PositionClosed`] event (see Events table below) for data logging.
    ///
    /// 6.  Return `Ok(())`.
    ///
    ///
    /// **[Accounts](jet_margin::accounts::ClosePosition) expected with close\_position.rs:**
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `authority` | `signer` | The authority that can change the margin account. |
    /// | `receiver` | `writable` | The receiver for the rent released. |
    /// | `margin_account` | `writable` | The margin account with the position to close. |
    /// | `position_token_mint` | `read only` | The mint for the position token being deregistered. |
    /// | `token_account` | `writable` | The token account for the position being closed. |
    /// | `token_program` | `read only` | The [spl token program](https://spl.solana.com/token). |
    ///
    /// **Events emitted by close\_position.rs:**
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::PositionClosed`] | The closed position. |

    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        close_position_handler(ctx)
    }

    /// Verify that the account is healthy, by validating the collateralization
    /// ration is above the minimum.
    ///
    /// This instruction does the following:
    ///
    /// 1.  Load the margin account.
    ///
    /// 2.  Check if positions for that margin account are healthy.
    ///     
    ///     a.  If unhealthy positions exist for this margin account, return `False`.
    ///
    /// 3.  Emit the [`events::VerifiedHealthy`] event (see Events table below) for data logging.
    ///
    /// 4.  Return `Ok(())`.
    ///
    ///
    /// **[Accounts](jet_margin::accounts::VerifyHealthy) expected with verify\_healthy.rs:**
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_account` | `read only` | The account to verify the health of. |
    ///
    /// **Events emitted by verify\_healthy.rs:**
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// |[`events::VerifiedHealthy`] | The pubkeys of margin accounts with verified healthy accounts. |

    pub fn verify_healthy(ctx: Context<VerifyHealthy>) -> Result<()> {
        verify_healthy_handler(ctx)
    }

    /// Perform an action by invoking other programs, allowing them to alter
    /// the balances of the token accounts belonging to this margin account.
    ///
    /// This instruction does the following:
    ///
    /// 1.  Check if the margin account is being liquidated:
    ///         
    ///     a.  If the margin account is being liquidated, return [`ErrorCode::Liquidating`].
    ///     
    /// 2.  Emit [`events::AdapterInvokeBegin`] event (see Events table below) for data logging.
    ///
    /// 3.  Check if any margin account positions that have changed via adapters.
    ///     
    ///     a.  For each position that changed, emit each existing adapter position as [`events::PositionEvent`].
    ///            
    /// 4. Emit [`events::AdapterInvokeEnd`] event (see Events table below) for data logging.
    ///     
    /// 5. Verify the margin account positions are healthy.
    ///     
    /// 6.  Return `Ok(())`.
    ///
    ///     
    ///
    /// **[Accounts](jet_margin::accounts::AdapterInvoke) expected with adapter\_invoke.rs:**
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `owner` | `signer` | The authority that owns the margin account. |
    /// | `margin_account` | `writable` | The margin account to proxy an action for. |
    /// | `adapter_program` | `read only` | The program to be invoked. |
    /// | `adapter_metadata` | `read only` | The metadata about the proxy program. |
    ///
    /// **Events emitted by adapter\_invoke.rs:**
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::AdapterInvokeBegin`] | Marks the start of the adapter invocation (includes the margin account pubkey and the adapter program pubkey). |
    // TODO: Better wording for PositionEvent below (made an attempt, let's see what Qiqi thinks)
    /// | [`events::PositionEvent`] _(Note that each single event represents a different adapter position)_ | The [PositionEvent](events::PositionEvent) of each adapter. |
    /// | [`events::AdapterInvokeEnd`] | Marks the ending of the adapter invocation (includes no data except for the event itself being emitted). |

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
    /// This instruction does the following:
    ///     
    /// 1.  Emit [`events::AccountingInvokeBegin`] event (see Events table below) for data logging.
    ///
    /// 2.  Invoke the adapter program to update the position for the margin account passed in.
    ///    
    ///     a.  For each position that changed, emit each existing adapter position as [`events::PositionEvent`] (see Events table below).
    ///            
    /// 3. Emit [`events::AccountingInvokeEnd`] event (see Events table below) for data logging.
    ///
    /// 4.  Return `Ok(())`.
    ///     
    ///
    /// **[Accounts](jet_margin::accounts::AccountingInvoke) expected with accounting\_invoke.rs:**
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** |  **Description** |
    /// | `margin_account` | `writable` | The margin account to proxy an action for. |
    /// | `adapter_program` | `read only` | The program to be invoked. |
    /// | `adapter_metadata` | `read only` | The metadata about the proxy program. |
    ///
    /// **Events emitted by accounting\_invoke.rs:**
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Name** | **Description** |
    /// | [`events::AccountingInvokeBegin`] | Signify that the accounting invocation process has begun. |
    // TODO: Better wording for PositionEvent below (made an attempt, let's see what Qiqi thinks)
    /// | [`events::PositionEvent`] _(Note that each single event represents an different adapter position)_ | The [PositionEvent](events::PositionEvent) of each adapter. |
    /// | [`events::AccountingInvokeEnd`] | Signify that the accounting invocation process has ended. |

    pub fn accounting_invoke<'info>(
        ctx: Context<'_, '_, '_, 'info, AccountingInvoke<'info>>,
        data: Vec<u8>,
    ) -> Result<()> {
        accounting_invoke_handler(ctx, data)
    }

    /// Begin liquidating an account
    ///
    /// This instruction does the following:
    ///
    /// 1. Create the liquidation account that persists the state of liquidation.
    ///
    /// 2. Load the liquidator and the margin account.
    ///     
    /// 3. Verify that the account is subject to liquidation.
    ///     
    /// 4. Verify that the account is not already being liquidated.
    ///     
    ///     a.  If the liquidator is already assigned to this margin account, do nothing.
    ///         
    ///     b.  If there is no liquidator assigned to this margin account, this liquidator can claim this unhealthy margin account.
    ///         
    ///     c.  Otherwise the margin account is already claimed, so return [`ErrorCode::Liquidating`].
    ///        
    /// 5. Record the current valuation of the margin account.
    ///     
    /// 6. Calculate the minimum collateral amount that must be liquidated.
    ///     
    /// 7. Begin the liquidation.
    ///  
    /// 8. Emit the [`events::LiquidationBegun`] event (see Events table below) for data logging.
    ///
    /// 9. Return `Ok(())`.
    ///     
    ///
    /// **[Accounts](jet_margin::accounts::LiquidateBegin) expected with liquidate\_begin.rs:**
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_account` | `writable` | The account in need of liquidation. |
    /// | `payer` | `signer` | The address paying rent. |
    /// | `liquidator` | `signer` | The liquidator account performing the liquidation. |
    /// | `liquidator_metadata` | `read only` | The metadata describing the liquidator. |
    /// | `liquidation` | `writable` | The account to persist the state of liquidation. |
    /// | `system_program` | `read only` | The [system native program](https://docs.solana.com/developing/runtime-facilities/programs#system-program). |
    ///
    /// **Events emitted by liquidate\_begin.rs:**
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::LiquidationBegun`] | The event marking the beginning of liquidation. |

    pub fn liquidate_begin(ctx: Context<LiquidateBegin>) -> Result<()> {
        liquidate_begin_handler(ctx)
    }

    /// Stop liquidating an account
    ///
    /// This instruction does the following:
    ///
    /// 1. Load the margin account.
    ///     
    /// 2. Load the start time from the liquidation account.
    ///     
    /// 3.Check if the current liquidation has timed out. Reference [`LIQUIDATION_TIMEOUT`](`jet_margin::LIQUIDATION_TIMEOUT`).
    ///         
    /// 4. If the liquidator is not timed out, and the liquidator is not the authorized liquidator, return [`ErrorCode::UnauthorizedLiquidator`].
    ///         
    /// 5. End the liquidation by setting the `liquidation` and `liquidator` fields of [`MarginAccount`](`jet_margin::MarginAccount`).
    ///     
    /// 6. Emit the [`events::LiquidationEnded`] event (see Events table below) for data logging.
    ///     
    /// 7. Return `Ok(())`.
    ///     
    ///
    /// **[Accounts](jet_margin::accounts::LiquidateEnd) expected with liquidate\_end.rs:**
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `authority` | `signer` | The pubkey calling the instruction to end liquidation. |
    /// | `margin_account` | `writable` | The account in need of liquidation. |
    /// | `liquidation` | `writable` | The account to persist the state of liquidation. |
    ///
    /// **Events emitted by liquidate\_end.rs:**
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::LiquidationEnded`] | The event marking the end of liquidation. |

    pub fn liquidate_end(ctx: Context<LiquidateEnd>) -> Result<()> {
        liquidate_end_handler(ctx)
    }

    /// Perform an action by invoking another program, for the purposes of
    /// liquidating a margin account.
    ///
    /// This instruction does the following:
    ///
    /// 1. Load the margin account.
    ///     
    /// 2. Load the current valuation of the margin account.
    ///     
    /// 3. Emit the [`events::LiquidatorInvokeBegin`] event (see Events table below) for data logging.
    ///     
    /// 4. Check if any margin account positions that have changed via adapters.
    ///     
    ///    a.  For each position that changed, emit each existing adapter position as [`events::PositionEvent`].
    ///     
    /// 5. Load liquidation account.
    ///
    /// 6. Update and verify the liquidation acount.
    ///     
    /// 7. Emit the [`events::LiquidatorInvokeEnd`] event (see Events table below) for data logging.
    ///     
    /// 8. Return `Ok(())`.
    ///         
    ///
    /// **[Accounts](jet_margin::accounts::LiquidatorInvoke) expected with liquidator\_invoke.rs:**
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `liquidator` | `signer` | The liquidator processing the margin account. |
    /// | `liquidation` | `writable` | The account to persist the state of liquidation. |
    /// | `margin_account` | `writable` | The margin account to proxy an action for. |
    /// | `adapter_program` | `read only` | The program to be invoked. |
    /// | `adapter_metadata` | `read only` | The metadata about the proxy program. |
    ///
    /// **Events emitted by liquidator\_invoke.rs:**
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::LiquidatorInvokeBegin`] | Marks the beginning of this liquidation event. |
    // TODO: Better wording for PositionEvent below (made an attempt, let's see what Qiqi thinks)
    /// | [`events::PositionEvent`] _(Note that each single event represents an different adapter position)_ | The [PositionEvent](events::PositionEvent) of each adapter. |
    /// | [`events::LiquidatorInvokeEnd`] | Marks the ending of this liquidator event. |

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

    /// 141041 - The liquidation attempted to extract too much value
    #[msg("attempted to extract too much value during liquidation")]
    LiquidationLostValue,
}

/// Writes the result of position changes from an adapter invocation.
pub fn write_adapter_result(margin_account: &MarginAccount, result: &AdapterResult) -> Result<()> {
    let mut adapter_result_data = vec![];
    result.serialize(&mut adapter_result_data)?;
    margin_account.invocation.verify_directly_invoked()?;
    anchor_lang::solana_program::program::set_return_data(&adapter_result_data);
    Ok(())
}
