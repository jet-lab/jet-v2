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

use std::{collections::BTreeMap, convert::TryInto};

use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program},
};
use anchor_spl::token::{Mint, TokenAccount};
use jet_metadata::{PositionTokenMetadata, TokenKind};

use crate::{
    events::{self, PositionTouched},
    util::{ErrorMessage, Require},
    AdapterPositionFlags, ErrorCode, MarginAccount, SignerSeeds,
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
    /// not registered at the provided address, unless the position token metadata
    /// account is included in the instruction, then the margin program will try
    /// to register the position.
    ///
    /// Example: This instruction involves an action by the owner of the margin
    /// account that increases a claim balance in their account, so the margin
    /// program must verify that the claim is registered as a position before
    /// allowing the instruction to complete successfully.
    Register(Pubkey),
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

/// Invoke a margin adapter with the requested data
/// * `signed` - sign with the margin account
pub fn invoke<'info>(
    ctx: &InvokeAdapter<'_, 'info>,
    account_metas: Vec<CompactAccountMeta>,
    data: Vec<u8>,
    signed: bool,
) -> Result<BTreeMap<Pubkey, PositionTouched>> {
    let signer = ctx.margin_account.load()?.signer_seeds_owned();

    let mut accounts = vec![AccountMeta {
        pubkey: ctx.margin_account.key(),
        is_signer: signed,
        is_writable: true,
    }];
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

    let mut account_infos = vec![ctx.margin_account.to_account_info()];
    account_infos.extend(ctx.remaining_accounts.iter().cloned());

    let instruction = Instruction {
        program_id: ctx.adapter_program.key(),
        accounts,
        data,
    };

    ctx.margin_account.load_mut()?.invocation.start();
    if signed {
        program::invoke_signed(&instruction, &account_infos, &[&signer.signer_seeds()])?;
    } else {
        program::invoke(&instruction, &account_infos)?;
    }
    ctx.margin_account.load_mut()?.invocation.end();

    handle_adapter_result(ctx)
}

fn handle_adapter_result(ctx: &InvokeAdapter) -> Result<BTreeMap<Pubkey, PositionTouched>> {
    let mut events = update_balances(ctx)?;

    match program::get_return_data() {
        None => (),
        Some((program_id, _)) if program_id != ctx.adapter_program.key() => (),
        Some((_, data)) => {
            let result = AdapterResult::deserialize(&mut &data[..])?;
            for (mint, changes) in result.position_changes {
                if let Some(event) = apply_changes(ctx, mint, changes)? {
                    events.insert(mint, event);
                }
            }
        }
    };

    // clear return data after reading it
    program::set_return_data(&[]);

    Ok(events)
}

fn update_balances(ctx: &InvokeAdapter) -> Result<BTreeMap<Pubkey, PositionTouched>> {
    let mut touched_positions = BTreeMap::new();

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
                    Ok(position) => {
                        touched_positions.insert(account.mint, PositionTouched { position });
                    }
                    Err(ErrorCode::PositionNotRegistered) => (),
                    Err(err) => return Err(err.into()),
                }
            }
        }
    }

    Ok(touched_positions)
}

fn apply_changes(
    ctx: &InvokeAdapter,
    mint: Pubkey,
    changes: Vec<PositionChange>,
) -> Result<Option<PositionTouched>> {
    let mut margin_account = ctx.margin_account.load_mut()?;
    let mut position = margin_account.get_position_mut(&mint);
    if let Some(ref p) = position {
        if p.adapter != ctx.adapter_program.key() {
            return err!(ErrorCode::InvalidPositionAdapter);
        }
    }
    for change in changes {
        let position = &mut position;
        match change {
            PositionChange::Price(px) => {
                if let Some(pos) = position {
                    pos.set_price(&px.try_into()?)?;
                }
            }
            PositionChange::Flags(flags, true) => position.require_mut()?.flags |= flags,
            PositionChange::Flags(flags, false) => position.require_mut()?.flags &= !flags,
            PositionChange::Register(token_account) => match position {
                Some(pos) => {
                    if pos.address != token_account {
                        msg!("position already registered for this mint with a different token account");
                        return err!(PositionNotRegisterable);
                    }
                }
                None => register_position(ctx, mint, token_account)?,
            },
        }
    }

    Ok(position.map(|p| PositionTouched { position: *p }))
}

fn register_position(
    ctx: &InvokeAdapter,
    mint_address: Pubkey,
    token_account_address: Pubkey,
) -> Result<()> {
    let mut metadata: Result<Account<PositionTokenMetadata>> = err!(PositionNotRegisterable);
    let mut token_account: Result<Account<TokenAccount>> = err!(PositionNotRegisterable);
    let mut mint: Result<Account<Mint>> = err!(PositionNotRegisterable);
    for info in ctx.remaining_accounts {
        if info.key == &token_account_address {
            token_account = Ok(Account::<TokenAccount>::try_from(info)?);
        } else if info.key == &mint_address {
            mint = Ok(Account::<Mint>::try_from(info)?);
        } else if info.owner == &PositionTokenMetadata::owner() {
            if let Ok(ptm) = Account::<PositionTokenMetadata>::try_from(info) {
                if ptm.position_token_mint == mint_address {
                    metadata = Ok(ptm);
                }
            }
        }
    }
    let metadata = metadata.log_on_error("position token metadata not found for mint")?;
    let token_account = token_account.log_on_error("position token mint not found")?;
    let mint = mint.log_on_error("position token account not found")?;

    if metadata.adapter_program != ctx.adapter_program.key() {
        msg!("adapter not authorized by metadata for this mint");
        return err!(PositionNotRegisterable);
    }
    if mint.key() != token_account.mint {
        msg!("token account has the wrong mint");
        return err!(PositionNotRegisterable);
    }
    if metadata.token_kind != TokenKind::Claim {
        msg!("adapters may only register claims");
        return err!(PositionNotRegisterable);
    }

    let position = ctx.margin_account.load_mut()?.register_position(
        mint.key(),
        mint.decimals,
        token_account.key(),
        metadata.adapter_program,
        metadata.token_kind.into(),
        metadata.value_modifier,
        metadata.max_staleness,
    )?;

    emit!(events::PositionRegistered {
        margin_account: ctx.margin_account.key(),
        authority: ctx.adapter_program.key(), //todo norman??
        position,
    });

    Ok(())
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, mem::size_of};

    use anchor_lang::Discriminator;

    use super::*;

    impl std::fmt::Debug for PositionTouched {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("PositionTouched")
                .field("position", &self.position)
                .finish()
        }
    }

    fn all_change_types() -> Vec<PositionChange> {
        vec![
            PositionChange::Price(PriceChangeInfo {
                value: 0,
                confidence: 0,
                twap: 0,
                publish_time: 0,
                exponent: 0,
            }),
            PositionChange::Flags(AdapterPositionFlags::empty(), true),
            PositionChange::Register(Pubkey::default()),
        ]
    }

    #[test]
    fn position_change_types_are_required_when_appropriate() {
        let mut data = [0u8; 100];
        let mut lamports = 0u64;
        let default = Pubkey::default();
        let margin = MarginAccount::owner();
        let adapter = AccountInfo::new(
            &default,
            false,
            false,
            &mut lamports,
            &mut data,
            &default,
            false,
            0,
        );
        let mut data = [0u8; 8 + size_of::<MarginAccount>()];
        data[..8].copy_from_slice(&MarginAccount::discriminator());
        let mut lamports = 0u64;
        let margin_account = AccountInfo::new(
            &default,
            false,
            true,
            &mut lamports,
            &mut data,
            &margin,
            false,
            0,
        );
        let ctx = InvokeAdapter {
            margin_account: &AccountLoader::try_from(&margin_account).unwrap(),
            adapter_program: &adapter,
            remaining_accounts: &[],
        };

        for change in all_change_types() {
            let required = match change {
                PositionChange::Price(_) => false,
                PositionChange::Flags(_, _) => true,
                PositionChange::Register(_) => true,
            };
            if required {
                apply_changes(&ctx, Pubkey::default(), vec![change]).unwrap_err();
            } else {
                apply_changes(&ctx, Pubkey::default(), vec![change]).unwrap();
            }
        }
    }

    #[test]
    fn ensure_that_tests_check_all_change_types() {
        assert_contains_all_variants! {
            all_change_types() =>
                PositionChange::Price(_x)
                PositionChange::Flags(_x, _y)
                PositionChange::Register(_x)
        }
    }

    macro_rules! assert_contains_all_variants {
        ($iterable:expr => $($type:ident::$var:ident $(($($_:ident),*))? )+ ) => {
            let mut index: HashMap<&str, usize> = HashMap::new();
            $(index.insert(stringify!($var), 1);)+
            for item in $iterable {
                match item {
                    $($type::$var $(($($_),*))? => index.insert(stringify!($var), 0)),+
                };
            }
            let sum: usize = index.values().sum();
            if sum > 0 {
                assert!(false);
            }
        };
    }
    use assert_contains_all_variants;
}
