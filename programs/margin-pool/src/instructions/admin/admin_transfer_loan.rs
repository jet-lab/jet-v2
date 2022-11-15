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
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::MarginPool;

#[derive(Accounts)]
pub struct AdminTransferLoan<'info> {
    /// The administrative authority
    #[account(address = super::ADMINISTRATOR)]
    pub authority: Signer<'info>,

    /// The margin pool with the loan
    pub margin_pool: Account<'info, MarginPool>,

    /// The loan account to be moved from
    #[account(mut, token::authority = margin_pool)]
    pub source_loan_account: Account<'info, TokenAccount>,

    /// The loan account to be moved into
    #[account(mut, token::authority = margin_pool)]
    pub target_loan_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> AdminTransferLoan<'info> {
    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.source_loan_account.to_account_info(),
                to: self.target_loan_account.to_account_info(),
                authority: self.margin_pool.to_account_info(),
            },
        )
    }
}

pub fn admin_transfer_loan_handler(ctx: Context<AdminTransferLoan>, amount: u64) -> Result<()> {
    let source_seeds = ctx.accounts.margin_pool.signer_seeds()?;

    token::transfer(
        ctx.accounts
            .transfer_context()
            .with_signer(&[&source_seeds]),
        amount,
    )?;

    Ok(())
}
