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

declare_id!("JPPooLEqRo3NCSx82EdE2VZY5vUaSsgskpZPBHNGVLZ");

pub mod authority {
    use super::*;

    declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");
}

#[program]
mod jet_margin_pool {
    use super::*;

    /// Create a new pool for borrowing and lending
    pub fn create_pool(ctx: Context<CreatePool>) -> Result<()> {
        instructions::create_pool_handler(ctx)
    }

    /// Configure an existing pool
    pub fn configure(
        ctx: Context<Configure>,
        fee_destination: Option<Pubkey>,
        config: Option<MarginPoolConfig>,
    ) -> Result<()> {
        instructions::configure_handler(ctx, fee_destination, config)
    }

    /// Accrue interest on the pool, and collect any fees.
    pub fn collect(ctx: Context<Collect>) -> Result<()> {
        instructions::collect_handler(ctx)
    }

    /// Deposit tokens into the pool in exchange for notes
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit_handler(ctx, amount)
    }

    /// Withdraw tokens from the pool, exchanging in previously received
    /// deposit notes.
    pub fn withdraw(ctx: Context<Withdraw>, amount: Amount) -> Result<()> {
        instructions::withdraw_handler(ctx, amount)
    }

    /// Borrow tokens using a margin account
    pub fn margin_borrow(ctx: Context<MarginBorrow>, amount: u64) -> Result<()> {
        instructions::margin_borrow_handler(ctx, amount)
    }

    /// Repay a loan with a maximum amount.
    /// If the loan balance is lower than the amount, the excess is left in the
    /// deposit account.
    pub fn margin_repay(ctx: Context<MarginRepay>, max_amount: Amount) -> Result<()> {
        instructions::margin_repay_handler(ctx, max_amount)
    }

    /// Withdraw tokens from the pool, exchanging in notes stored by a
    /// margin account.
    pub fn margin_withdraw(ctx: Context<MarginWithdraw>, amount: Amount) -> Result<()> {
        instructions::margin_withdraw_handler(ctx, amount)
    }

    /// Update the pool position on a margin account
    pub fn margin_refresh_position(ctx: Context<MarginRefreshPosition>) -> Result<()> {
        instructions::margin_refresh_position_handler(ctx)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
pub enum AmountKind {
    Tokens,
    Notes,
}

/// Represent an amount of some value (like tokens, or notes)
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
pub struct Amount {
    pub kind: AmountKind,
    pub value: u64,
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
    /// This is used when a `TokenAmount` has an invalid value
    #[msg("An invalid amount has been supplied")]
    InvalidAmount,

    /// 141105 - The oracle is not reporting a valid price
    InvalidPrice,

    /// 141106 - The oracle account is not valid
    InvalidOracle,

    /// 141107 - Attempt repayment of more tokens than total outstanding
    RepaymentExceedsTotalOutstanding,
}
