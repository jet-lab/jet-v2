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

use std::convert::TryInto;

use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program},
};
use anchor_spl::token::TokenAccount;

use crate::{
    util::Require, AccountPosition, AdapterPositionFlags, ErrorCode, MarginAccount, SignerSeeds,
};

pub struct InvokeAdapter<'a, 'info> {
    /// The margin account to proxy an action for
    pub margin_account: &'a AccountLoader<'info, MarginAccount>,

    /// The program to be invoked
    pub adapter_program: &'a AccountInfo<'info>,

    /// The accounts to be passed through to the adapter
    pub remaining_accounts: &'a [AccountInfo<'info>],
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CompactAccountMeta {
    pub is_signer: u8,
    pub is_writable: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct AdapterResult {
    /// keyed by token mint, same as position
    pub position_changes: Vec<(Pubkey, Vec<PositionChange>)>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum PositionChange {
    /// The price/value of the position has already changed,
    /// so the margin account must update its price
    Price(PriceChangeInfo),

    /// Flags that are true here will be set to the bool in the position
    /// Flags that are false here will be unchanged in the position
    Flags(AdapterPositionFlags, bool),

    /// The margin program will fail the current instruction if this position is
    /// not registered at the provided address.
    ///
    /// Example: This instruction involves an action by the owner of the margin
    /// account that increases a claim balance in their account, so the margin
    /// program must verify that the claim is registered as a position before
    /// allowing the instruction to complete successfully.
    Expect(Pubkey),
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct PriceChangeInfo {
    /// The current price of the asset
    pub value: i64,

    /// The current confidence value for the asset price
    pub confidence: u64,

    /// The recent average price
    pub twap: i64,

    /// The time that the price was published at
    pub publish_time: i64,

    /// The exponent for the price values
    pub exponent: i32,
}

/// Executes an unpermissioned invocation with the requested data
pub fn invoke(
    ctx: &InvokeAdapter,
    account_metas: Vec<CompactAccountMeta>,
    data: Vec<u8>,
) -> Result<()> {
    let (instruction, account_infos) = construct_invocation(ctx, account_metas, data);

    program::invoke(&instruction, &account_infos)?;

    handle_adapter_result(ctx)
}

/// Invoke with the requested data, and sign with the margin account
pub fn invoke_signed(
    ctx: &InvokeAdapter,
    account_metas: Vec<CompactAccountMeta>,
    data: Vec<u8>,
) -> Result<()> {
    let signer = ctx.margin_account.load()?.signer_seeds_owned();
    let (instruction, account_infos) = construct_invocation(ctx, account_metas, data);

    program::invoke_signed(&instruction, &account_infos, &[&signer.signer_seeds()])?;

    handle_adapter_result(ctx)
}

fn construct_invocation<'info>(
    ctx: &InvokeAdapter<'_, 'info>,
    account_metas: Vec<CompactAccountMeta>,
    data: Vec<u8>,
) -> (Instruction, Vec<AccountInfo<'info>>) {
    let mut accounts = vec![AccountMeta {
        pubkey: ctx.margin_account.key(),
        is_signer: true,
        is_writable: true,
    }];
    let mut account_infos = vec![ctx.margin_account.to_account_info()];

    accounts.extend(
        account_metas
            .into_iter()
            .zip(ctx.remaining_accounts.iter())
            .map(|(meta, account_info)| AccountMeta {
                pubkey: account_info.key(),
                is_signer: meta.is_signer != 0,
                is_writable: meta.is_writable != 0,
            }),
    );

    account_infos.extend(ctx.remaining_accounts.iter().cloned());

    let instruction = Instruction {
        program_id: ctx.adapter_program.key(),
        accounts,
        data,
    };

    (instruction, account_infos)
}

fn handle_adapter_result(ctx: &InvokeAdapter) -> Result<()> {
    update_balances(ctx)?;

    match program::get_return_data() {
        None => (),
        Some((program_id, _)) if program_id != ctx.adapter_program.key() => (),
        Some((program_id, data)) => {
            let result = AdapterResult::deserialize(&mut &data[..])?;
            let mut margin_account = ctx.margin_account.load_mut()?;
            for (mint, changes) in result.position_changes {
                let position = margin_account.get_position_mut(&mint);
                match position {
                    Some(p) if p.adapter != program_id => {
                        return err!(ErrorCode::InvalidPositionAdapter)
                    }
                    _ => apply_changes(position, changes)?,
                }
            }
        }
    };

    Ok(())
}

fn update_balances(ctx: &InvokeAdapter) -> Result<()> {
    let mut margin_account = ctx.margin_account.load_mut()?;
    for account_info in ctx.remaining_accounts {
        if account_info.owner == &TokenAccount::owner() {
            let data = &mut &**account_info.try_borrow_data()?;
            if let Ok(account) = TokenAccount::try_deserialize(data) {
                match margin_account.set_position_balance(
                    &account.mint,
                    account_info.key,
                    account.amount,
                ) {
                    Ok(()) | Err(ErrorCode::PositionNotRegistered) => (),
                    Err(err) => return Err(err.into()),
                }
            }
        }
    }

    Ok(())
}

fn apply_changes(
    mut position: Option<&mut AccountPosition>,
    changes: Vec<PositionChange>,
) -> Result<()> {
    for change in changes {
        match change {
            PositionChange::Price(px) => {
                if let Some(pos) = &mut position {
                    pos.set_price(&px.try_into()?)?;
                }
            }
            PositionChange::Flags(flags, true) => position.require_mut()?.flags |= flags,
            PositionChange::Flags(flags, false) => position.require_mut()?.flags &= !flags,
            PositionChange::Expect(pubkey) => {
                if position.require_mut()?.address != pubkey {
                    return Err(error!(ErrorCode::PositionNotRegistered));
                }
            }
        }
    }

    Ok(())
}
