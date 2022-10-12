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

mod instructions;
mod state;
mod util;
use instructions::*;

pub use state::{MarginPool, MarginPoolConfig, PoolFlags};
pub mod events;

declare_id!("JPPooLEqRo3NCSx82EdE2VZY5vUaSsgskpZPBHNGVLZ");

pub mod authority {
    use super::*;

    declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");
}

#[program]
mod jet_margin_pool {
    use super::*;

    /// Create a new pool for borrowing and lending
    /// 
    /// # Parameters
    ///
    /// TODO below fee_destination def
    /// * `fee_destination` - The account that collects fees on the created pool.
    ///
    /// # [Accounts](jet_margin::accounts::CreatePool)
    ///     
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_pool` | `read_only` | The pool to be created. |
    /// | `vault` | `read_only` | The token account holding the pool's deposited funds. |
    /// | `deposit_note_mint` | `read_only` | The mint for deposit notes. |
    /// | `loan_note_mint` | `read_only` | The mint for loan notes. |
    /// | `token_mint` | `read_only` | The mint for the token being custodied by the pool. |
    /// | `authority` | `read_only` | The authority to create pools, which must sign. |
    /// | `payer` | `Signer` | The payer of rent for new accounts. |
    /// | `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |
    /// | `system_program` | `read_only` | The [system native program](https://docs.solana.com/developing/runtime-facilities/programs#system-program). |
    /// | `rent` | `read_only` | The [rent sysvar](https://docs.solana.com/developing/runtime-facilities/sysvars#rent) to create the pool. |
    /// 
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::PoolCreated`] | The pool created. |
    /// 
    pub fn create_pool(ctx: Context<CreatePool>, fee_destination: Pubkey) -> Result<()> {
        instructions::create_pool_handler(ctx, fee_destination)
    }

 
    /// Accrue interest on the pool, and collect any fees.
    ///
    /// # Parameters
    ///
    /// * [`clock`](solana_program::clock::Clock) - The network time represented as the current slot.       
    /// ABOVE -- double check that the link works right for Clock
    ///
    /// # [Accounts](jet_margin::accounts::Collect)
    ///     
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_pool` | `writable` | The pool to be refreshed. |
    /// | `vault` | `writable` | The vault for the pool, where tokens are held. |
    /// | `fee_destination` | `writable` | The account to deposit the collected fees. |
    /// | `deposit_note_mint` | `writable` | The mint for the deposit notes. |
    /// | `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::Collect`] | The collected fees. |
    /// TODO make sure its ok I switched the function below (did this to match the instruction layout tree like the rest of them do)
    /// 
    pub fn collect(ctx: Context<Collect>) -> Result<()> {
        instructions::collect_handler(ctx)
    }

    /// Configure an existing pool
    ///
    /// TODO better / more comprehensive defiinition for config? "The data for configuring the respective pool" seems a bit too light
    /// * `config` - TODO An abritrary integer used to derive the new account address. This allows
    ///            a user to own multiple margin accounts, by creating new accounts with different
    ///            seed values.
    ///
    /// # [Accounts](jet_margin::accounts::Configure)
    ///     
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_pool` | `writable` | The pool to be configured. |
    /// | `authority` | `read_only` | The authoirty to modify the pool, which must sign. |
    /// | `pyth_product` | `read_only` | The pyth oracle for the margin pool being configured. |
    /// | `pyth_price` | `read_only` | The price data for the oracle. |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::PoolConfigured`] | The pool that was configured. |
    pub fn configure(ctx: Context<Configure>, config: Option<MarginPoolConfig>) -> Result<()> {
        instructions::configure_handler(ctx, config)
    }

    /// Deposit tokens into the pool in exchange for notes
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
    /// | [`events::Collect`] | The collected fees. |
    pub fn deposit(ctx: Context<Deposit>, change_kind: ChangeKind, amount: u64) -> Result<()> {
        instructions::deposit_handler(ctx, change_kind, amount)
    }

    /// Withdraw tokens from the pool, exchanging in previously received
    /// deposit notes.
    ///
    /// TODO: change the below, this is just a placeholder to c&p and adjust for each function
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
    /// | [`events::AccountCreated`] | The created account. |
    pub fn withdraw(ctx: Context<Withdraw>, change_kind: ChangeKind, amount: u64) -> Result<()> {
        instructions::withdraw_handler(ctx, change_kind, amount)
    }

    /// Borrow tokens using a margin account
    pub fn margin_borrow(
        ctx: Context<MarginBorrow>,
        change_kind: ChangeKind,
        amount: u64,
    ) -> Result<()> {
        instructions::margin_borrow_handler(ctx, change_kind, amount)
    }

    /// Repay a loan with a maximum amount.
    /// If the loan balance is lower than the amount, the excess is left in the
    /// deposit account.
    pub fn margin_repay(
        ctx: Context<MarginRepay>,
        change_kind: ChangeKind,
        amount: u64,
    ) -> Result<()> {
        instructions::margin_repay_handler(ctx, change_kind, amount)
    }

    /// Repay a margin account debt from an outside token account
    pub fn repay(ctx: Context<Repay>, change_kind: ChangeKind, amount: u64) -> Result<()> {
        instructions::repay_handler(ctx, change_kind, amount)
    }

    /// Update the pool position on a margin account
    pub fn margin_refresh_position(ctx: Context<MarginRefreshPosition>) -> Result<()> {
        instructions::margin_refresh_position_handler(ctx)
    }

    /// Creates the token account to track the loan notes,
    /// then requests margin to register the position
    pub fn register_loan(ctx: Context<RegisterLoan>) -> Result<()> {
        instructions::register_loan_handler(ctx)
    }

    /// Closes a previously opened loan token account
    ///     
    /// # [Accounts](jet_margin::accounts::CloseLoan)

    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_account` | `read_only` | The token account to store the loan notes representing the claim against the margin account. |
    /// | `loan_note_account` | `read_only` | The account that has the loan obligation that is being closed. |
    /// | `loan_note_mint` | `read_only` | The mint for the notes representing loans from the pool. |
    /// | `margin_pool` | `read_only` | The margin pool that the loan originates from. |
    /// TODO: double check my description for beneficiary 
    /// | `beneficiary` | `writable` | The destination for the  account closing the loan(?). |
    /// | `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |
    /// 
    pub fn close_loan(ctx: Context<CloseLoan>) -> Result<()> {
        instructions::close_loan_handler(ctx)
    }
}

/// Interface for changing the token value of an account through pool instructions
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
pub struct TokenChange {
    /// The kind of change to be applied
    pub kind: ChangeKind,
    /// The number of tokens applied in the change
    pub tokens: u64,
}

impl TokenChange {
    /// Sets a position's balance to the supplied value, increasing or decreasing
    /// the balance depending on the instruction type.
    ///
    /// Withdrawing with `set(0)` will withdraw all tokens in an account.
    /// Borrowing with `set(100_000)` will borrow additional tokens until the
    /// provided value is reached. If there are already 40_000 tokens borrowed,
    /// an additional 60_000 will be borrowed.
    pub const fn set(value: u64) -> Self {
        Self {
            kind: ChangeKind::SetTo,
            tokens: value,
        }
    }
    /// Shifts a position's balance by the supplied value, increasing or decreasing
    /// the balance depending on the instruction type.
    ///
    /// Withdrawing with `shift(100_000)` tokens will decerase a balance by the amount.
    /// Depositing with `shift(100_000)` tokens will increaes a balance by the amount.
    ///
    /// Refer to the various isntructions for the behaviour of when instructions can
    /// fail.
    pub const fn shift(value: u64) -> Self {
        Self {
            kind: ChangeKind::ShiftBy,
            tokens: value,
        }
    }

    /// The amount of the token change, expressed as tokens.
    ///
    /// [Amount] can also be notes when interacting with pools, however it is
    /// always set to tokens for `TokenChange`.
    pub fn amount(&self) -> Amount {
        Amount::tokens(self.tokens)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
#[repr(u8)]
pub enum ChangeKind {
    SetTo,
    ShiftBy,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
pub enum AmountKind {
    Tokens,
    Notes,
}

/// Represent an amount of some value (like tokens, or notes)
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
pub struct Amount {
    kind: AmountKind,
    value: u64,
}

impl Amount {
    pub const fn tokens(value: u64) -> Self {
        Self {
            kind: AmountKind::Tokens,
            value,
        }
    }

    pub const fn notes(value: u64) -> Self {
        Self {
            kind: AmountKind::Notes,
            value,
        }
    }

    pub fn value(&self) -> u64 {
        self.value
    }
}

#[error_code]
pub enum ErrorCode {
    /// 141100 - The pool is currently disabled
    #[msg("The pool is currently disabled")]
    Disabled = 135_100,

    /// 141101 - Interest accrual is too far behind
    #[msg("Interest accrual is too far behind")]
    InterestAccrualBehind,

    /// 141102 - The pool currently only allows deposits
    #[msg("The pool currently only allows deposits")]
    DepositsOnly,

    /// 141103 - There are not enough tokens in a pool to fulfil transaction
    #[msg("The pool does not have sufficient liquidity for the transaction")]
    InsufficientLiquidity,

    /// 141104 - An invalid amount has been supplied
    ///
    /// This is used when an `Amount` has an invalid value
    #[msg("An invalid amount has been supplied")]
    InvalidAmount,

    /// 141105 - The oracle is not reporting a valid price
    InvalidPoolPrice,

    /// 141106 - The oracle account is not valid
    InvalidPoolOracle,

    /// 141107 - Tried to set an invalid token value
    #[msg("An invalid `SetTo` value was given for a `TokenChange`")]
    InvalidSetTo,

    /// 141108 - Attempt repayment of more tokens than total outstanding
    RepaymentExceedsTotalOutstanding,
}
