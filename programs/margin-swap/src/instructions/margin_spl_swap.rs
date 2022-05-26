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

use anchor_spl::token::Token;

use crate::*;

#[derive(Accounts)]
pub struct MarginSplSwap<'info> {
    /// The margin account being executed on
    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The account with the source deposit to be exchanged from
    /// CHECK:
    #[account(mut)]
    pub source_account: AccountInfo<'info>,

    /// The destination account to send the deposit that is exchanged into
    /// CHECK:
    #[account(mut)]
    pub destination_account: AccountInfo<'info>,

    /// Temporary account for moving tokens
    /// CHECK:
    #[account(mut)]
    pub transit_source_account: AccountInfo<'info>,

    /// Temporary account for moving tokens
    /// CHECK:
    #[account(mut)]
    pub transit_destination_account: AccountInfo<'info>,

    /// The accounts relevant to the swap pool used for the exchange
    pub swap_info: SwapInfo<'info>,

    /// The accounts relevant to the source margin pool
    pub source_margin_pool: MarginPoolInfo<'info>,

    /// The accounts relevant to the destination margin pool
    pub destination_margin_pool: MarginPoolInfo<'info>,

    pub margin_pool_program: Program<'info, JetMarginPool>,

    pub token_program: Program<'info, Token>,
}

impl<'info> MarginSplSwap<'info> {
    fn withdraw_source_context(&self) -> CpiContext<'_, '_, '_, 'info, Withdraw<'info>> {
        CpiContext::new(
            self.margin_pool_program.to_account_info(),
            Withdraw {
                margin_pool: self.source_margin_pool.margin_pool.to_account_info(),
                vault: self.source_margin_pool.vault.to_account_info(),
                deposit_note_mint: self.source_margin_pool.deposit_note_mint.to_account_info(),
                depositor: self.margin_account.to_account_info(),
                source: self.source_account.to_account_info(),
                destination: self.transit_source_account.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    fn deposit_destination_context(&self) -> CpiContext<'_, '_, '_, 'info, Deposit<'info>> {
        CpiContext::new(
            self.margin_pool_program.to_account_info(),
            Deposit {
                margin_pool: self.destination_margin_pool.margin_pool.to_account_info(),
                vault: self.destination_margin_pool.vault.to_account_info(),
                deposit_note_mint: self
                    .destination_margin_pool
                    .deposit_note_mint
                    .to_account_info(),
                depositor: self.margin_account.to_account_info(),
                source: self.transit_destination_account.to_account_info(),
                destination: self.destination_account.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}

#[derive(Accounts)]
pub struct SwapInfo<'info> {
    /// CHECK:
    pub swap_pool: UncheckedAccount<'info>,

    /// CHECK:
    pub authority: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub vault_into: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub vault_from: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub token_mint: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub fee_account: UncheckedAccount<'info>,

    /// The address of the swap program, currently limited to spl_token_swap
    pub swap_program: Program<'info, SplTokenSwap>,
}

pub fn margin_spl_swap_handler(
    ctx: Context<MarginSplSwap>,
    amount_in: u64,
    minimum_amount_out: u64,
) -> Result<()> {
    jet_margin_pool::cpi::withdraw(
        ctx.accounts.withdraw_source_context(),
        Amount::tokens(amount_in),
    )?;

    let swap_ix = spl_token_swap::instruction::swap(
        ctx.accounts.swap_info.swap_program.key,
        ctx.accounts.token_program.key,
        ctx.accounts.swap_info.swap_pool.key,
        ctx.accounts.swap_info.authority.key,
        &ctx.accounts.margin_account.key(),
        ctx.accounts.transit_source_account.key,
        ctx.accounts.swap_info.vault_into.key,
        ctx.accounts.swap_info.vault_from.key,
        ctx.accounts.transit_destination_account.key,
        ctx.accounts.swap_info.token_mint.key,
        ctx.accounts.swap_info.fee_account.key,
        None,
        spl_token_swap::instruction::Swap {
            amount_in,
            minimum_amount_out,
        },
    )?;

    invoke(
        &swap_ix,
        &[
            ctx.accounts.swap_info.swap_pool.to_account_info(),
            ctx.accounts.swap_info.authority.to_account_info(),
            ctx.accounts.margin_account.to_account_info(),
            ctx.accounts.transit_source_account.to_account_info(),
            ctx.accounts.swap_info.vault_into.to_account_info(),
            ctx.accounts.swap_info.vault_from.to_account_info(),
            ctx.accounts.transit_destination_account.to_account_info(),
            ctx.accounts.swap_info.token_mint.to_account_info(),
            ctx.accounts.swap_info.fee_account.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
    )?;

    let destination_amount = token::accessor::amount(&ctx.accounts.transit_destination_account)?;

    jet_margin_pool::cpi::deposit(
        ctx.accounts.deposit_destination_context(),
        destination_amount,
    )?;

    Ok(())
}
