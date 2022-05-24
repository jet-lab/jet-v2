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

use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program},
};
use anchor_spl::token::TokenAccount;

use jet_proto_math::Number128;

use crate::{
    AccountPosition, AdapterPositionFlags, ErrorCode, MarginAccount, PriceInfo,
    MAX_ORACLE_CONFIDENCE, MAX_ORACLE_STALENESS,
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

    /// The margin program will fail the current instruction if this position is not registered.
    ///
    /// Example: This instruction involves an action by the owner of the margin
    /// account that increases a claim balance in their account, so the margin
    /// program must verify that the claim is registered as a position before
    /// allowing the instruction to complete successfully.
    ExpectPosition,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct PriceChangeInfo {
    /// The current price of the asset
    pub value: i64,

    /// The current confidence value for the asset price
    pub confidence: u64,

    /// The recent average price
    pub twap: i64,

    /// The slot number that the price was published in
    pub slot: u64,

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
    let (instruction, account_infos) = construct_invocation(ctx, account_metas, data);

    let ma = ctx.margin_account.load()?;
    program::invoke_signed(
        &instruction,
        &account_infos,
        &[&[ma.owner.as_ref(), &ma.user_seed, &[ma.bump_seed[0]]]],
    )?;

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

    let result = match program::get_return_data() {
        None => AdapterResult::default(),
        Some((program_id, _)) if program_id != ctx.adapter_program.key() => {
            return Err(ErrorCode::WrongProgramAdapterResult.into())
        }
        Some((_, data)) => AdapterResult::deserialize(&mut &data[..])?,
    };

    let mut margin_account = ctx.margin_account.load_mut()?;
    for (mint, changes) in result.position_changes {
        let position = margin_account.get_position_mut(&mint)?;
        for change in changes {
            match change {
                PositionChange::Price(px) => update_price(ctx, position, px)?,
                PositionChange::Flags(flags, true) => position.flags |= flags,
                PositionChange::Flags(flags, false) => position.flags &= !flags,
                PositionChange::ExpectPosition => (), // get_position_mut verified that the position exists
            }
        }
    }

    Ok(())
}

fn update_balances(ctx: &InvokeAdapter) -> Result<()> {
    for account_info in ctx.remaining_accounts {
        if account_info.owner == &TokenAccount::owner() {
            let account = TokenAccount::try_deserialize(&mut &**account_info.try_borrow_data()?)?;
            if account.owner == ctx.margin_account.key() {
                if let Err(err) = ctx.margin_account.load_mut()?.set_position_balance(
                    &account.mint,
                    account_info.key,
                    account.amount,
                ) {
                    match err {
                        Error::AnchorError(a)
                            if a.error_code_number == ErrorCode::PositionNotRegistered.into() => {}
                        _ => Err(err)?,
                    }
                }
            }
        }
    }

    Ok(())
}

fn update_price(
    ctx: &InvokeAdapter,
    position: &mut AccountPosition,
    entry: PriceChangeInfo,
) -> Result<()> {
    let clock = Clock::get()?;
    let max_confidence = Number128::from_bps(MAX_ORACLE_CONFIDENCE);

    let twap = Number128::from_decimal(entry.twap, entry.exponent);
    let confidence = Number128::from_decimal(entry.confidence, entry.exponent);

    let price = match (confidence, entry.slot) {
        (c, _) if (c / twap) > max_confidence => PriceInfo::new_invalid(),
        (_, slot) if (clock.slot - slot) > MAX_ORACLE_STALENESS => PriceInfo::new_invalid(),
        _ => PriceInfo::new_valid(entry.exponent, entry.value, clock.unix_timestamp as u64),
    };

    match position.set_price(ctx.adapter_program.key, &price) {
        Err(Error::AnchorError(e))
            if e.error_code_number
                == (ErrorCode::UnknownPosition as u32 + anchor_lang::error::ERROR_CODE_OFFSET) =>
        {
            Ok(())
        }
        Err(e) => Err(e),
        Ok(()) => Ok(()),
    }
}
