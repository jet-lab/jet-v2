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

use crate::{
    events::{PositionClosed, PositionEvent, PositionRegistered, PositionTouched},
    util::{ErrorMessage, Require},
    AccountPositionKey, AdapterPositionFlags, Approver, ErrorCode, MarginAccount, SignerSeeds,
};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program},
};
use anchor_spl::token::{Mint, TokenAccount};
use jet_metadata::PositionTokenMetadata;

pub struct InvokeAdapter<'a, 'info> {
    /// The margin account to proxy an action for
    pub margin_account: &'a AccountLoader<'info, MarginAccount>,

    /// The program to be invoked
    pub adapter_program: &'a AccountInfo<'info>,

    /// The accounts to be passed through to the adapter
    pub accounts: &'a [AccountInfo<'info>],

    /// The transaction was signed by the authority of the margin account.
    /// Thus, the invocation should be signed by the margin account.
    pub signed: bool,
}

impl InvokeAdapter<'_, '_> {
    /// those who approve of the requests within the adapter result
    fn adapter_result_approvals(&self) -> Vec<Approver> {
        let mut ret = Vec::new();
        if self.signed {
            ret.push(Approver::MarginAccountAuthority);
        }
        ret.push(Approver::Adapter(self.adapter_program.key()));

        ret
    }
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

    /// Register a new position, or assert that a position is registered
    /// if the position cannot be registered, instruction fails
    /// if the position is already registered, instruction succeeds without taking action
    Register(Pubkey),

    /// Close a position, or assert that a position is closed
    /// if the position cannot be closed, instruction fails
    /// if the position does not exist, instruction succeeds without taking action
    Close(Pubkey),
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
pub fn invoke<'info>(ctx: &InvokeAdapter<'_, 'info>, data: Vec<u8>) -> Result<Vec<PositionEvent>> {
    let signer = ctx.margin_account.load()?.signer_seeds_owned();

    let accounts = ctx
        .accounts
        .iter()
        .map(|info| AccountMeta {
            pubkey: info.key(),
            is_signer: if info.key() == ctx.margin_account.key() {
                ctx.signed
            } else {
                info.is_signer
            },
            is_writable: info.is_writable,
        })
        .collect::<Vec<AccountMeta>>();

    let instruction = Instruction {
        program_id: ctx.adapter_program.key(),
        accounts,
        data,
    };

    ctx.margin_account.load_mut()?.invocation.start();
    if ctx.signed {
        program::invoke_signed(&instruction, ctx.accounts, &[&signer.signer_seeds()])?;
    } else {
        program::invoke(&instruction, ctx.accounts)?;
    }
    ctx.margin_account.load_mut()?.invocation.end();

    handle_adapter_result(ctx)
}

fn handle_adapter_result(ctx: &InvokeAdapter) -> Result<Vec<PositionEvent>> {
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

    Ok(events.into_values().collect())
}

fn update_balances(ctx: &InvokeAdapter) -> Result<BTreeMap<Pubkey, PositionEvent>> {
    let mut touched_positions: BTreeMap<Pubkey, PositionEvent> = BTreeMap::new();

    let mut margin_account = ctx.margin_account.load_mut()?;
    for account_info in ctx.accounts {
        if account_info.owner == &TokenAccount::owner() {
            let data = &mut &**account_info.try_borrow_data()?;
            if let Ok(account) = TokenAccount::try_deserialize(data) {
                match margin_account.set_position_balance(
                    &account.mint,
                    account_info.key,
                    account.amount,
                ) {
                    Ok(position) => {
                        touched_positions.insert(
                            account.mint,
                            PositionTouched {
                                margin_account: ctx.margin_account.key(),
                                position,
                            }
                            .into(),
                        );
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
) -> Result<Option<PositionEvent>> {
    let mut margin_account = ctx.margin_account.load_mut()?;
    let mut key = margin_account.get_position_key(&mint);
    let mut position = key.and_then(|k| margin_account.get_position_by_key_mut(&k));
    let mut net_registration = 0isize;
    if let Some(ref p) = position {
        if p.adapter != ctx.adapter_program.key() {
            return err!(ErrorCode::InvalidPositionAdapter);
        }
    }
    for change in changes {
        position = key.and_then(|k| margin_account.get_position_by_key_mut(&k));
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
                None => {
                    key = Some(register_position(
                        &mut margin_account,
                        ctx.accounts,
                        ctx.adapter_result_approvals().as_slice(),
                        mint,
                        token_account,
                    )?);
                    net_registration += 1;
                }
            },
            PositionChange::Close(token_account) => match position {
                Some(pos) => {
                    if pos.address != token_account {
                        msg!("position registered for this mint with a different token account");
                        return err!(PositionNotRegisterable);
                    }
                    margin_account.unregister_position(
                        &mint,
                        &token_account,
                        ctx.adapter_result_approvals().as_slice(),
                    )?;
                    key = None;
                    net_registration -= 1;
                }
                None => (),
            },
        }
    }
    Ok(match net_registration {
        0 => key
            .and_then(|k| margin_account.get_position_by_key(&k))
            .map(|p| {
                PositionTouched {
                    margin_account: ctx.margin_account.key(),
                    position: *p,
                }
                .into()
            }),
        n if n > 0 => Some(
            PositionRegistered {
                position: *key
                    .and_then(|k| margin_account.get_position_by_key(&k))
                    .require()?,
                margin_account: ctx.margin_account.key(),
                authority: ctx.adapter_program.key(),
            }
            .into(),
        ),
        _ => Some(
            PositionClosed {
                margin_account: ctx.margin_account.key(),
                authority: ctx.adapter_program.key(),
                token: mint,
            }
            .into(),
        ),
    })
}

fn register_position(
    margin_account: &mut MarginAccount,
    remaining_accounts: &[AccountInfo],
    approvals: &[Approver],
    mint_address: Pubkey,
    token_account_address: Pubkey,
) -> Result<AccountPositionKey> {
    let mut metadata: Result<Account<PositionTokenMetadata>> = err!(PositionNotRegisterable);
    let mut token_account: Result<Account<TokenAccount>> = err!(PositionNotRegisterable);
    let mut mint: Result<Account<Mint>> = err!(PositionNotRegisterable);
    for info in remaining_accounts {
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

    if mint.key() != token_account.mint {
        msg!("token account has the wrong mint");
        return err!(PositionNotRegisterable);
    }

    let key = margin_account.register_position(
        mint.key(),
        mint.decimals,
        token_account.key(),
        metadata.adapter_program,
        metadata.token_kind.into(),
        metadata.value_modifier,
        metadata.max_staleness,
        approvals,
    )?;

    margin_account.set_position_balance(
        &mint_address,
        &token_account_address,
        token_account.amount,
    )?;

    Ok(key)
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, mem::size_of};

    use anchor_lang::Discriminator;

    use super::*;

    impl std::fmt::Debug for PositionEvent {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("PositionEvent").finish()
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
            PositionChange::Close(Pubkey::default()),
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
            accounts: &[],
            signed: true,
        };

        for change in all_change_types() {
            let required = match change {
                PositionChange::Price(_) => false,
                PositionChange::Flags(_, _) => true,
                PositionChange::Register(_) => true,
                PositionChange::Close(_) => false,
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
                PositionChange::Close(_x)
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
