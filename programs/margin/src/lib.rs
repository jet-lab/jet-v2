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

pub mod events;
pub mod seeds;

mod adapter;
mod instructions;
mod state;
pub mod syscall;
/// Utilities used only in this crate
pub(crate) mod util;

use instructions::*;
pub use state::*;
pub use util::Invocation;

pub use adapter::{AdapterResult, PositionChange, PriceChangeInfo};
pub use instructions::TokenConfigUpdate;

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
    /// # Parameters
    ///
    /// * `seed` - An abritrary integer used to derive the new account address. This allows
    ///            a user to own multiple margin accounts, by creating new accounts with different
    ///            seed values.
    ///
    /// # [Accounts](jet_margin::accounts::CreateAccount)
    ///     
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `owner` | `signer` | The owner of the new margin account. |
    /// | `payer` | `signer` | The pubkey paying rent for the new margin account opening. |
    /// | `margin_account` | `writable` | The margin account to initialize for the owner. |
    /// | `system_program` | `read_only` | The [system native program](https://docs.solana.com/developing/runtime-facilities/programs#system-program). |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::AccountCreated`] | Marks the creation of the account. |

    pub fn create_account(ctx: Context<CreateAccount>, seed: u16) -> Result<()> {
        create_account_handler(ctx, seed)
    }

    /// Close a user's margin account
    ///
    /// The margin account must have zero positions remaining to be closed.
    ///
    /// # [Accounts](jet_margin::accounts::CloseAccount)
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `owner` | `signer` | The owner of the account being closed. |
    /// | `receiver` | `writable` | The account to get any returned rent. |
    /// | `margin_account` | `writable` | The account being closed. |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::AccountClosed`] | Marks the closure of the account. |

    pub fn close_account(ctx: Context<CloseAccount>) -> Result<()> {
        close_account_handler(ctx)
    }

    /// Register a position for deposits of tokens returned by adapter programs (e.g. margin-pool).
    ///     
    /// This will create a token account to hold the adapter provided tokens which represent
    /// a user's deposit with that adapter.
    ///
    /// This instruction may fail if the account has reached it's maximum number of positions.
    ///
    /// # [Accounts](jet_margin::accounts::RegisterPosition)
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `authority` | `signer` | The authority that can change the margin account. |
    /// | `payer` | `signer` | The address paying for rent. |
    /// | `margin_account` | `writable` |  The margin account to register position type with. |
    /// | `position_token_mint` | `read_only` | The mint for the position token being registered. |
    /// | `metadata` | `read_only` | The metadata account that references the correct oracle for the token. |
    /// | `token_account` | `writable` | The token account to store hold the position assets in the custody of the margin account. |
    /// | `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |
    /// | `rent` | `read_only` | The [rent sysvar](https://docs.solana.com/developing/runtime-facilities/sysvars#rent). The rent to open the account. |
    /// | `system_program` | `read_only` | The [system native program](https://docs.solana.com/developing/runtime-facilities/programs#system-program). |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::PositionRegistered`] | Marks the registration of the position. |
    pub fn register_position(ctx: Context<RegisterPosition>) -> Result<()> {
        register_position_handler(ctx)
    }

    /// Update the balance of a position stored in the margin account to match the actual
    /// stored by the SPL token account.
    ///
    /// When a user deposits tokens directly (without invoking this program), there's no
    /// update within the user's margin account to account for the new token balance. This
    /// instruction allows udating the margin account state to reflect the current available
    /// balance of collateral.
    ///
    /// # [Accounts](jet_margin::accounts::UpdatePositionBalance)
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_account` | `writable` | The margin account to update. |
    /// | `token_account` | `read_only` | The token account to update the balance for. |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::PositionBalanceUpdated`] | Marks the updating of the position balance. |
    ///
    pub fn update_position_balance(ctx: Context<UpdatePositionBalance>) -> Result<()> {
        update_position_balance_handler(ctx)
    }

    /// Update the metadata for a position stored in the margin account,
    /// in the case where the metadata has changed after the position was
    /// created.
    ///
    /// # [Accounts](jet_margin::accounts::RefreshPositionMetadata)
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_account` | `writable` | The margin account with the position to be refreshed. |
    /// | `metadata` | `read_only` | The metadata account for the token, which has been updated. |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::PositionMetadataRefreshed`] | Marks the refreshing of position metadata. |
    /// 
    pub fn refresh_position_metadata(ctx: Context<RefreshPositionMetadata>) -> Result<()> {
        refresh_position_metadata_handler(ctx)
    }

    /// Close out a position, removing it from the account.
    ///
    /// Since there is a finite number of positions a single account can maintain it may be
    /// necessary for a user to close out old positions to take new ones.
    ///
    /// # [Accounts](jet_margin::accounts::ClosePosition)
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `authority` | `signer` | The authority that can change the margin account. |
    /// | `receiver` | `writable` | The receiver for the rent released. |
    /// | `margin_account` | `writable` | The margin account with the position to close. |
    /// | `position_token_mint` | `read_only` | The mint for the position token being deregistered. |
    /// | `token_account` | `writable` | The token account for the position being closed. |
    /// | `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::PositionClosed`] | Marks the closure of the position. |
    /// 
    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        close_position_handler(ctx)
    }

    /// Verify that the account is healthy, by validating the collateralization
    /// ration is above the minimum.
    ///
    /// There's no real reason to call this instruction, outside of wanting to simulate
    /// the health check for a margin account.
    ///
    ///
    /// # [Accounts](jet_margin::accounts::VerifyHealthy)
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_account` | `read_only` | The account to verify the health of. |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::VerifiedHealthy`] | Marks the verification of the position. |
    /// 
    pub fn verify_healthy(ctx: Context<VerifyHealthy>) -> Result<()> {
        verify_healthy_handler(ctx)
    }

    /// Perform an action by invoking other programs, allowing them to alter
    /// the balances of the token accounts belonging to this margin account.
    ///
    /// This provides the margin account as a signer to any invoked instruction, and therefore
    /// grants the adapter authority over any tokens held by the margin account.
    ///
    /// This validates the invoked program by expecting an `adapter_metadata` account,
    /// which must exist for the instruction to be considered valid. The configuration
    /// for allowing adapter programs is controlled by protocol governance.
    ///
    /// All extra accounts passed in are used as the input accounts when invoking
    /// the provided adapter porgram.
    ///
    /// # Parameters
    ///
    /// * `data` - The instruction data to pass to the adapter program
    ///
    /// # [Accounts](jet_margin::accounts::AdapterInvoke)
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `owner` | `signer` | The authority that owns the margin account. |
    /// | `margin_account` | `writable` | The margin account to proxy an action for. |
    /// | `adapter_program` | `read_only` | The program to be invoked. |
    /// | `adapter_metadata` | `read_only` | The metadata about the proxy program. |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::AdapterInvokeBegin`] | Marks the start of the adapter invocation (includes the margin account pubkey and the adapter program pubkey). |
    /// | [`events::PositionEvent`] _(Note that each single event represents a different adapter position)_ | The [PositionEvent](events::PositionEvent) marks the change in position. |
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
    /// This is a permissionless way of updating the value of positions on a margin
    /// account which require some adapter to provide the update. Unlike `adapter_invoke`,
    /// this instruction will not provider the margin account as a signer to invoked programs,
    /// and they thefore do not have authority to modify any token balances held by the account.
    ///     
    /// All extra accounts passed in are used as the input accounts when invoking
    /// the provided adapter porgram.
    ///
    /// # Parameters
    ///
    /// * `data` - The instruction data to pass to the adapter program
    ///
    /// # [Accounts](jet_margin::accounts::AccountingInvoke)
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** |  **Description** |
    /// | `margin_account` | `writable` | The margin account to proxy an action for. |
    /// | `adapter_program` | `read_only` | The program to be invoked. |
    /// | `adapter_metadata` | `read_only` | The metadata about the proxy program. |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Name** | **Description** |
    /// | [`events::AccountingInvokeBegin`] | Signify that the accounting invocation process has begun. |
    /// | [`events::PositionEvent`] _(Note that each single event represents an different adapter position)_ | The [PositionEvent](events::PositionEvent) marks the change in position. |
    /// | [`events::AccountingInvokeEnd`] | Signify that the accounting invocation process has ended. |
    pub fn accounting_invoke<'info>(
        ctx: Context<'_, '_, '_, 'info, AccountingInvoke<'info>>,
        data: Vec<u8>,
    ) -> Result<()> {
        accounting_invoke_handler(ctx, data)
    }

    /// Begin liquidating an account
    ///
    /// The account will enter a state preventing the owner from taking any action,
    /// until the liquidator process is complete.
    ///
    /// Requires the `liquidator_metadata` account, which restricts the signer to
    /// those approved by protocol governance.
    ///
    /// # [Accounts](jet_margin::accounts::LiquidateBegin)
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_account` | `writable` | The account in need of liquidation. |
    /// | `payer` | `signer` | The address paying rent. |
    /// | `liquidator` | `signer` | The liquidator account performing the liquidation. |
    /// | `liquidator_metadata` | `read_only` | The metadata describing the liquidator. |
    /// | `liquidation` | `writable` | The account to persist the state of liquidation. |
    /// | `system_program` | `read_only` | The [system native program](https://docs.solana.com/developing/runtime-facilities/programs#system-program). |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::LiquidationBegun`] | Marks the beginning of the liquidation. |
    pub fn liquidate_begin(ctx: Context<LiquidateBegin>) -> Result<()> {
        liquidate_begin_handler(ctx)
    }

    /// End the liquidation state for an account
    ///
    /// Normally must be signed by the liquidator that started the liquidation state. Can be
    /// signed by anyone after the [timeout period](jet_margin::LIQUIDATION_TIMEOUT) has elapsed.
    ///
    /// # [Accounts](jet_margin::accounts::LiquidateEnd)
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `authority` | `signer` | The pubkey calling the instruction to end liquidation. |
    /// | `margin_account` | `writable` | The account in need of liquidation. |
    /// | `liquidation` | `writable` | The account to persist the state of liquidation. |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::LiquidationEnded`] | Marks the ending of the liquidation. |
    pub fn liquidate_end(ctx: Context<LiquidateEnd>) -> Result<()> {
        liquidate_end_handler(ctx)
    }

    /// Perform an action by invoking another program, for the purposes of
    /// liquidating a margin account.
    ///
    /// Requires the account already be in the liquidation state, and the signer must
    /// be the same liquidator that started the liquidation state.      
    ///
    /// # [Accounts](jet_margin::accounts::LiquidatorInvoke)
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `liquidator` | `signer` | The liquidator processing the margin account. |
    /// | `liquidation` | `writable` | The account to persist the state of liquidation. |
    /// | `margin_account` | `writable` | The margin account to proxy an action for. |
    /// | `adapter_program` | `read_only` | The program to be invoked. |
    /// | `adapter_metadata` | `read_only` | The metadata about the proxy program. |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::LiquidatorInvokeBegin`] | Marks the beginning of this liquidation event. |
    /// | [`events::PositionEvent`] _(Note that each single event represents an different adapter position)_ | The [PositionEvent](events::PositionEvent) describing the change in position. |
    /// | [`events::LiquidatorInvokeEnd`] | Marks the ending of this liquidator event. |
    pub fn liquidator_invoke<'info>(
        ctx: Context<'_, '_, '_, 'info, LiquidatorInvoke<'info>>,
        data: Vec<u8>,
    ) -> Result<()> {
        liquidator_invoke_handler(ctx, data)
    }

    /// Update the config for a token position stored in the margin account,
    /// in the case where the token config has changed after the position was
    /// created.
    pub fn refresh_position_config(ctx: Context<RefreshPositionConfig>) -> Result<()> {
        refresh_position_config_handler(ctx)
    }

    /// Refresh the price/balance for a deposit position
    pub fn refresh_deposit_position(ctx: Context<RefreshDepositPosition>) -> Result<()> {
        refresh_deposit_position_handler(ctx)
    }

    /// Create a new account for holding SPL token deposits directly by a margin account.
    pub fn create_deposit_position(ctx: Context<CreateDepositPosition>) -> Result<()> {
        create_deposit_position_handler(ctx)
    }

    /// Transfer tokens into or out of a token account being used for deposits.
    pub fn transfer_deposit(ctx: Context<TransferDeposit>, amount: u64) -> Result<()> {
        transfer_deposit_handler(ctx, amount)
    }

    /// Set the configuration for a token, which allows it to be used as a position in a margin
    /// account.
    ///
    /// The configuration for a token only applies for the associated airspace, and changing any
    /// configuration requires the airspace authority to sign.
    ///
    /// The account storing the configuration will be funded if not already. If a `None` is provided as
    /// the updated configuration, then the account will be defunded.
    pub fn configure_token(
        ctx: Context<ConfigureToken>,
        update: Option<TokenConfigUpdate>,
    ) -> Result<()> {
        configure_token_handler(ctx, update)
    }

    /// Set the configuration for an adapter.
    ///
    /// The configuration for a token only applies for the associated airspace, and changing any
    /// configuration requires the airspace authority to sign.
    ///
    /// The account storing the configuration will be funded if not already. If a `None` is provided as
    /// the updated configuration, then the account will be defunded.
    pub fn configure_adapter(ctx: Context<ConfigureAdapter>, is_adapter: bool) -> Result<()> {
        configure_adapter_handler(ctx, is_adapter)
    }

    /// Set the configuration for a liquidator.
    ///
    /// The configuration for a token only applies for the associated airspace, and changing any
    /// configuration requires the airspace authority to sign.
    ///
    /// The account storing the configuration will be funded if not already. If a `None` is provided as
    /// the updated configuration, then the account will be defunded.
    pub fn configure_liquidator(
        ctx: Context<ConfigureLiquidator>,
        is_liquidator: bool,
    ) -> Result<()> {
        configure_liquidator_handler(ctx, is_liquidator)
    }

    /// Allow governing address to transfer any position from one margin account to another
    ///
    /// This is provided as a mechanism to allow for manually fixing issues that occur in the
    /// protocol due to bad user assets.
    pub fn admin_transfer_position(ctx: Context<AdminTransferPosition>, amount: u64) -> Result<()> {
        admin_transfer_position_handler(ctx, amount)
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

    /// 141050 - The airspace does not match
    #[msg("attempting to mix entities from different airspaces")]
    WrongAirspace = 135_050,

    /// 141051 - Attempting to use or set configuration that is not valid
    #[msg("attempting to use or set invalid configuration")]
    InvalidConfig = 135_051,

    /// 141051 - Attempting to use or set an oracle that is not valid
    #[msg("attempting to use or set invalid configuration")]
    InvalidOracle = 135_052,
}

/// Writes the result of position changes from an adapter invocation.
pub fn write_adapter_result(margin_account: &MarginAccount, result: &AdapterResult) -> Result<()> {
    let mut adapter_result_data = vec![];
    result.serialize(&mut adapter_result_data)?;
    margin_account.invocation.verify_directly_invoked()?;
    anchor_lang::solana_program::program::set_return_data(&adapter_result_data);
    Ok(())
}
