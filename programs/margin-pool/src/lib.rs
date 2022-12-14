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
    /// * `fee_destination` - The address of the account to deposit collected fees, represented as deposit notes.
    ///
    /// **[Accounts](jet_margin::accounts::CreatePool) expected with create\_pool.rs:**
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
    /// | [`events::PoolCreated`] | Marks the creation of the pool. |
    pub fn create_pool(ctx: Context<CreatePool>, fee_destination: Pubkey) -> Result<()> {
        instructions::create_pool_handler(ctx, fee_destination)
    }

    /// Accrue interest on the pool, and collect any fees.
    ///
    /// # Parameters
    ///
    /// * [`clock`](solana_program::clock::Clock) - The network time represented as the current slot.       
    /// TODO: ABOVE -- double check that the link (also) works right for Clock in docs
    ///
    /// **[Accounts](jet_margin::accounts::Collect) expected with collect.rs:**
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
    /// | [`events::Collect`] | Marks the collection of the fees. |
    pub fn collect(ctx: Context<Collect>) -> Result<()> {
        instructions::collect_handler(ctx)
    }

    /// Configure an existing pool
    ///
    /// * `config` - The data with which to configure the respective pool.
    ///
    /// **[Accounts](jet_margin::accounts::Configure) expected with configure.rs:**
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
    /// | [`events::PoolConfigured`] | Marks the configuration of the pool. |
    pub fn configure(ctx: Context<Configure>, config: Option<MarginPoolConfig>) -> Result<()> {
        instructions::configure_handler(ctx, config)
    }

    /// Deposit tokens into the pool in exchange for notes
    ///
    /// * `change` - Contains `change_kind` and `amount`, which specify the pool operation type (in this case a deposit) and amount of tokens.
    ///
    /// **[Accounts](jet_margin::accounts::Deposit) expected with deposit.rs:**
    ///     
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_pool` | `writable` | The pool to deposit into. |
    /// | `vault` | `writable` | The vault for the pool, where tokens are held. |
    /// | `deposit_note_mint` | `writable` | The mint for deposit notes. |
    /// | `depositor` | `writable` | The address with authority to deposit the tokens. |
    /// | `source` | `writable` | The source of the tokens to be deposited. |
    /// | `destination` | `writable` | The destination of the deposit notes. |
    /// | `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::Deposit`] | Marks the deposit. |
    pub fn deposit(ctx: Context<Deposit>, change_kind: ChangeKind, amount: u64) -> Result<()> {
        instructions::deposit_handler(ctx, change_kind, amount)
    }

    /// Withdraw tokens from the pool, exchanging in previously received
    /// deposit notes.
    ///
    /// # Parameters
    ///
    /// * `change` - Contains `change_kind` and `amount`, which specify the pool operation type (in this case a withdraw) and amount of tokens.
    ///
    /// * [`clock`](solana_program::clock::Clock) - The network time represented as the current slot.       
    /// TODO: ABOVE -- double check that the link works right for Clock in docs
    /// TODO: I don't see anything populating in the docs..hmmm...
    ///
    /// **[Accounts](jet_margin::accounts::Withdraw) expected with withdraw.rs:**
    ///     
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `depositor` | `signer` | The address with authority to withdraw the deposit. |
    /// | `margin_pool` | `writable` | The pool to withdraw from. |
    /// | `vault` | `writable` | The vault for the pool, where tokens are held. |
    /// | `deposit_note_mint` | `writable` | The mint for the deposit notes. |
    /// | `source` | `writable` | The source of the deposit notes to be redeemed. |
    /// | `destination` | `writable` | The destination of the tokens withdrawn. |
    /// | `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::Withdraw`] | Marks the withdrawal. |
    pub fn withdraw(ctx: Context<Withdraw>, change_kind: ChangeKind, amount: u64) -> Result<()> {
        instructions::withdraw_handler(ctx, change_kind, amount)
    }

    /// Borrow tokens using a margin account
    ///
    /// # Parameters
    ///
    /// * `change` - Contains `change_kind` and `amount`, which specify the pool operation type (in this case a margin borrow) and amount of tokens.
    ///
    /// **[Accounts](jet_margin::accounts::MarginBorrow) expected with margin\_borrow.rs:**
    ///     
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_account` | `read_only` | The margin account being executed on. |
    /// | `margin_pool` | `writable` | The pool to borrow from. |
    /// | `loan_note_mint` | `writable` | The mint for the notes representing loans from the pool. |
    /// | `deposit_note_mint` | `writable` | The mint for the notes representing deposit into the pool. |
    /// | `loan_account` | `writable` | The account to receive the loan notes. |
    /// | `deposit_account` | `writable` | The account to receive the borrowed tokens (as deposit notes). |
    /// | `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::MarginBorrow`] | Marks the margin borrow. |
    pub fn margin_borrow(
        ctx: Context<MarginBorrow>,
        change_kind: ChangeKind,
        amount: u64,
    ) -> Result<()> {
        instructions::margin_borrow_handler(ctx, change_kind, amount)
    }

    /// Repay a margin account debt from an outside token account
    ///
    /// # Parameters
    ///
    /// * `change` - Contains `change_kind` and `amount`, which specify the pool operation type (in this case a margin repay) and amount of tokens.
    ///
    /// **[Accounts](jet_margin::accounts::MarginRepay) expected with margin\_repay.rs:**
    ///     
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_account` | `Signer` | The margin account being executed on. |
    /// | `margin_pool` | `writable` | The pool with the outstanding loan. |
    /// | `loan_note_mint` | `writable` | The mint for the notes representing loans from the pool. |
    /// | `deposit_note_mint` | `writable` | The mint for the notes representing deposit into the pool. |
    /// | `loan_account` | `writable` | The account with the loan notes. |
    /// | `deposit_account` | `writable` | The account with the deposit to pay off the loan with. |
    /// | `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::MarginRepay`] | Marks the margin repay. |
    pub fn margin_repay(
        ctx: Context<MarginRepay>,
        change_kind: ChangeKind,
        amount: u64,
    ) -> Result<()> {
        instructions::margin_repay_handler(ctx, change_kind, amount)
    }

    /// Repay a margin account debt from an outside token account
    ///
    /// # Parameters
    ///
    /// * `change` - Contains `change_kind` and `amount`, which specify the pool operation type (in this case a repay) and amount.

    ///
    /// **[Accounts](jet_margin::accounts::Repay) expected with repay.rs:**
    ///     
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_pool` | `writable` | The pool with the outstanding loan. |
    /// | `loan_note_mint` | `writable` | The mint for the notes representing loans from the pool. |
    /// | `vault` | `writable` | The vault responsible for storing the pool's tokens. |
    /// | `loan_account` | `writable` | The account with the loan notes. |
    /// | `repayment_token_account` | `writable` | The token account repaying the debt. |
    /// | `repayment_account_authority` | `Signer` | Signing authority for the repaying token account. |
    /// | `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::Repay`] | Marks the repay. |
    pub fn repay(ctx: Context<Repay>, change_kind: ChangeKind, amount: u64) -> Result<()> {
        instructions::repay_handler(ctx, change_kind, amount)
    }

    /// Update the pool position on a margin account
    ///
    /// # Parameters
    ///
    /// **[Accounts](jet_margin::accounts::MarginRefreshPosition) expected with margin\_refresh\_position.rs:**

    ///     
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_account` | `read_only` | The margin account being executed on. |
    /// | `margin_pool` | `read_only` | The pool to be refreshed. |
    /// | `token_price_oracle` | `read_only` | The pyth price account for the pool's token. |
    pub fn margin_refresh_position(ctx: Context<MarginRefreshPosition>) -> Result<()> {
        instructions::margin_refresh_position_handler(ctx)
    }

    /// Creates the token account to track the loan notes,
    /// then requests margin to register the position
    ///
    /// **[Accounts](jet_margin::accounts::RegisterLoan) expected with register\_loan.rs:**
    ///     
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_account` | `Signer` | The margin account authority. |
    /// | `position_token_metadata` | `read_only` | This will be required for margin to register the position, so requiring it here makes it easier for clients to ensure that it will be sent. |
    /// | `loan_note_account` | `read_only` | The token account to store the loan notes representing the claim against the margin account. |
    /// | `loan_note_mint` | `read_only` | The mint for the notes representing loans from the pool. |
    /// | `margin_pool` | `read_only` | The margin pool that will be used for the loan. |
    /// | `payer` | `writable` | The payer of rent for the loan transaction. |
    /// | `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |
    /// | `system_program` | `read_only` | The [system native program](https://docs.solana.com/developing/runtime-facilities/programs#system-program). |
    /// | `rent` | `read_only` | The [rent sysvar](https://docs.solana.com/developing/runtime-facilities/sysvars#rent) to create the pool. |
    ///
    pub fn register_loan(ctx: Context<RegisterLoan>) -> Result<()> {
        instructions::register_loan_handler(ctx)
    }

    /// Closes a previously opened loan token account
    ///     
    /// **[Accounts](jet_margin::accounts::CloseLoan) expected with close\_loan.rs:**
    ///
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `margin_account` | `read_only` | The token account to store the loan notes representing the claim against the margin account. |
    /// | `loan_note_account` | `read_only` | The account that has the loan obligation that is being closed. |
    /// | `loan_note_mint` | `read_only` | The mint for the notes representing loans from the pool. |
    /// | `margin_pool` | `read_only` | The margin pool that the loan originates from. |
    /// | `beneficiary` | `writable` | The authority permitting the closure of the loan. |
    /// | `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |
    pub fn close_loan(ctx: Context<CloseLoan>) -> Result<()> {
        instructions::close_loan_handler(ctx)
    }

    /// Administrative function for moving loans between accounts
    ///
    /// **[Accounts](jet_margin::accounts::AdminTransferLoan) expected with admin\_transfer\_loan.rs:**
    ///     
    /// |     |     |     |
    /// | --- | --- | --- |
    /// | **Name** | **Type** | **Description** |
    /// | `authority` | `Signer` | The administrative authority. |
    /// | `margin_pool` | `read_only` | The margin pool with the loan. |
    /// | `source_loan_account` | `writable` | The loan account to be moved from. |
    /// | `target_loan_account` | `writable` | The loan account to be moved into. |
    /// | `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |
    ///
    /// # Events
    ///
    /// |     |     |
    /// | --- | --- |
    /// | **Event Name** | **Description** |
    /// | [`events::LoanTransferred`] | Marks the transferral of the loan. |
    pub fn admin_transfer_loan(ctx: Context<AdminTransferLoan>, amount: u64) -> Result<()> {
        instructions::admin_transfer_loan_handler(ctx, amount)
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
