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
use anchor_spl::token;
use instructions::*;

use jet_proto_math::Number;
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
    pub fn create_pool(ctx: Context<CreatePool>, fee_destination: Pubkey) -> Result<()> {
        instructions::create_pool_handler(ctx, fee_destination)
    }

    /// Configure an existing pool
    pub fn configure(ctx: Context<Configure>, config: Option<MarginPoolConfig>) -> Result<()> {
        instructions::configure_handler(ctx, config)
    }

    /// Accrue interest on the pool, and collect any fees.
    pub fn collect(ctx: Context<Collect>) -> Result<()> {
        instructions::collect_handler(ctx)
    }

    /// Deposit tokens into the pool in exchange for notes
    pub fn deposit(ctx: Context<Deposit>, change_kind: ChangeKind, amount: u64) -> Result<()> {
        instructions::deposit_handler(ctx, change_kind, amount)
    }

    /// Withdraw tokens from the pool, exchanging in previously received
    /// deposit notes.
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
    pub fn close_loan(ctx: Context<CloseLoan>) -> Result<()> {
        instructions::close_loan_handler(ctx)
    }
}

/// Interface for changing the token value of an account through pool instructions
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
pub struct TokenChange {
    pub kind: ChangeKind,
    pub tokens: u64,
}

impl TokenChange {
    pub const fn set(value: u64) -> Self {
        Self {
            kind: ChangeKind::SetTo,
            tokens: value,
        }
    }
    pub const fn shift(value: u64) -> Self {
        Self {
            kind: ChangeKind::ShiftBy,
            tokens: value,
        }
    }

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
    InvalidPrice,

    /// 141106 - The oracle account is not valid
    InvalidOracle,

    /// 141107 - Tried to set an invalid token value
    #[msg("An invalid `SetTo` value was given for a `TokenChange`")]
    InvalidSetTo,

    /// 141108 - Attempt repayment of more tokens than total outstanding
    RepaymentExceedsTotalOutstanding,

    /// 141109 - Accounting violation already existed before this instruction
    PriorAccountingViolation,

    /// 141110 - Accounting violation introduced in this instruction
    NewAccountingViolation,
}

pub fn check_balances(
    pool: &MarginPool,
    vault: Option<&AccountInfo>,
    deposit_notes: Option<&AccountInfo>,
    loan_notes: Option<&AccountInfo>,
    error: Result<()>,
) -> Result<()> {
    if let Some(vault) = vault {
        let amount = token::accessor::amount(vault)?;
        if amount != pool.deposit_tokens {
            msg!("token vault {} pool {}", amount, pool.deposit_tokens);
            return error;
        }
    }
    if let Some(deposit_notes) = deposit_notes {
        let amount = supply(deposit_notes)?;
        if amount != pool.deposit_notes {
            msg!("deposit_notes mint {} pool {}", amount, pool.deposit_notes);
            return error;
        }
    }
    if let Some(loan_notes) = loan_notes {
        let amount = supply(loan_notes)?;
        if amount != pool.loan_notes {
            msg!("loan_notes mint {} pool {}", amount, pool.loan_notes);
            return error;
        }
    }

    Ok(())
}
pub fn check_exchange_rates(
    pool: &MarginPool,
    deposit_note_exchange_rate_before_accrual: Number,
    loan_note_exchange_rate_before_accrual: Number,
    deposit_note_exchange_rate_after_accrual: Number,
    loan_note_exchange_rate_after_accrual: Number,
    error: Result<()>,
) -> Result<()> {
    if deposit_note_exchange_rate_after_accrual < deposit_note_exchange_rate_before_accrual {
        msg!(
            "Scenario 1: deposit_note_exchange_rate before accrual {} after {}",
            deposit_note_exchange_rate_before_accrual,
            deposit_note_exchange_rate_after_accrual
        );
        return error;
    }
    if loan_note_exchange_rate_after_accrual < loan_note_exchange_rate_before_accrual {
        msg!(
            "Scenario 1: loan_note_exchange_rate before accrual {} after {}",
            loan_note_exchange_rate_before_accrual,
            loan_note_exchange_rate_after_accrual
        );
        return error;
    }
    if pool.deposit_note_exchange_rate() < deposit_note_exchange_rate_after_accrual {
        msg!(
            "Scenario 2: deposit_note_exchange_rate after accrual {} end {}",
            deposit_note_exchange_rate_after_accrual,
            pool.deposit_note_exchange_rate()
        );
        return error;
    }
    if pool.loan_note_exchange_rate() > loan_note_exchange_rate_after_accrual
        // When no notes
        && pool.loan_note_exchange_rate() != Number::ONE
    {
        msg!(
            "Scenario 2: loan_note_exchange_rate after accrual {} end {}",
            loan_note_exchange_rate_after_accrual,
            pool.loan_note_exchange_rate()
        );
        return error;
    }

    Ok(())
}

pub fn supply(account: &AccountInfo) -> Result<u64> {
    let bytes = account.try_borrow_data()?;
    let mut amount_bytes = [0u8; 8];
    amount_bytes.copy_from_slice(&bytes[36..44]);
    Ok(u64::from_le_bytes(amount_bytes))
}
