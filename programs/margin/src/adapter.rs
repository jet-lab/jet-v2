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

use crate::{ErrorCode, MarginAccount, PriceInfo, MAX_ORACLE_CONFIDENCE, MAX_ORACLE_STALENESS};

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

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum AdapterResult {
    /// The balance of some account positions were changed by this instruction
    NewBalanceChange(Vec<Pubkey>),

    /// The invoked program has previously changed the user's actual balances through
    /// some other mechanism. This adapter invocation only serves an accounting role
    /// to bring the margin account's symbolic balances up to date with the reality of
    /// the underlying positions.
    PriorBalanceChange(Vec<Pubkey>),

    /// Indicates the price/value of some positions should change
    PriceChange(Vec<PriceChangeInfo>),
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct PriceChangeInfo {
    /// The mint for the position which has a price change
    pub mint: Pubkey,

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

pub fn invoke(
    ctx: &InvokeAdapter,
    account_metas: Vec<CompactAccountMeta>,
    data: Vec<u8>,
) -> Result<AdapterResult> {
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

    let (owner, seed, bump) = {
        let account = ctx.margin_account.load()?;
        (account.owner, account.user_seed, account.bump_seed[0])
    };

    program::invoke_signed(
        &instruction,
        &account_infos,
        &[&[owner.as_ref(), &seed, &[bump]]],
    )?;

    let result_data = match program::get_return_data() {
        None => return Err(ErrorCode::NoAdapterResult.into()),
        Some((program_id, _)) if program_id != ctx.adapter_program.key() => {
            return Err(ErrorCode::WrongProgramAdapterResult.into())
        }
        Some((_, data)) => data,
    };

    let result = AdapterResult::deserialize(&mut &result_data[..])?;
    let mut margin_account = ctx.margin_account.load_mut()?;

    match result {
        AdapterResult::NewBalanceChange(ref modified_accounts)
        | AdapterResult::PriorBalanceChange(ref modified_accounts) => {
            for modified in modified_accounts {
                let account_info = account_infos.iter().find(|a| a.key == modified).unwrap();
                let account =
                    TokenAccount::try_deserialize(&mut &**account_info.try_borrow_data()?)?;

                // sanity check that this account is actually owned by the margin program
                if account.owner != ctx.margin_account.key() {
                    msg!("position account {} not owned", modified);
                    return Err(ErrorCode::PositionNotOwned.into());
                }

                margin_account.set_position_balance(
                    &account.mint,
                    account_info.key,
                    account.amount,
                )?;
            }
        }

        AdapterResult::PriceChange(ref price_list) => {
            let clock = Clock::get()?;
            let max_confidence = Number128::from_bps(MAX_ORACLE_CONFIDENCE);

            for entry in price_list {
                let twap = Number128::from_decimal(entry.twap, entry.exponent);
                let confidence = Number128::from_decimal(entry.confidence, entry.exponent);

                let price = match (confidence, entry.publish_time) {
                    (c, _) if (c / twap) > max_confidence => PriceInfo::new_invalid(),
                    (_, publish_time)
                        if (clock.unix_timestamp - publish_time) > MAX_ORACLE_STALENESS =>
                    {
                        PriceInfo::new_invalid()
                    }
                    _ => PriceInfo::new_valid(
                        entry.exponent,
                        entry.value,
                        clock.unix_timestamp as u64,
                    ),
                };

                match margin_account.set_position_price(
                    &entry.mint,
                    ctx.adapter_program.key,
                    &price,
                ) {
                    Err(Error::AnchorError(e))
                        if e.error_code_number
                            == (ErrorCode::UnknownPosition as u32
                                + anchor_lang::error::ERROR_CODE_OFFSET) => {}
                    Err(e) => return Err(e),
                    Ok(()) => (),
                }
            }
        }
    }

    Ok(result)
}
