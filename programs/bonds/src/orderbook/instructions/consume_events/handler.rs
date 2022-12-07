use agnostic_orderbook::{
    instruction::consume_events,
    state::{
        event_queue::{FillEvent, OutEvent},
        Side,
    },
};
use anchor_lang::prelude::*;
use jet_program_common::traits::{SafeAdd, SafeSub, TryAddAssign};
use num_traits::FromPrimitive;

use crate::{
    bond_token_manager::BondTokenManager,
    events::{skip_err, AssetsUpdated, DebtUpdated, ObligationCreated, OrderFilled, OrderRemoved},
    margin::state::{Obligation, ObligationFlags},
    orderbook::state::{fp32_mul, CallbackFlags, CallbackInfo, FillInfo, OutInfo},
    tickets::state::SplitTicket,
    utils::map,
    BondsError,
};

use super::{queue, ConsumeEvents, FillAccounts, OutAccounts, PreparedEvent};

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, ConsumeEvents<'info>>,
    num_events: u32,
    seeds: Vec<Vec<u8>>,
) -> Result<()> {
    let mut num_iters = 0;
    for event in queue(&ctx, seeds)?.take(num_events as usize) {
        match event? {
            PreparedEvent::Fill(accounts, info) => handle_fill(&ctx, *accounts, &info),
            PreparedEvent::Out(accounts, info) => handle_out(&ctx, *accounts, &info),
        }?;

        num_iters += 1;
    }
    if num_iters == 0 {
        return err!(BondsError::NoEvents);
    }

    agnostic_orderbook::instruction::consume_events::process::<CallbackInfo>(
        ctx.program_id,
        consume_events::Accounts {
            market: &ctx.accounts.orderbook_market_state.to_account_info(),
            event_queue: &ctx.accounts.event_queue.to_account_info(),
        },
        consume_events::Params {
            number_of_entries_to_consume: num_iters,
        },
    )?;

    Ok(())
}

fn handle_fill<'info>(
    ctx: &Context<'_, '_, '_, 'info, ConsumeEvents<'info>>,
    accounts: FillAccounts<'info>,
    fill: &FillInfo,
) -> Result<()> {
    let manager = &ctx.accounts.bond_manager;
    let FillAccounts {
        maker,
        maker_adapter,
        taker_adapter,
        mut loan,
    } = accounts;
    let FillInfo {
        event,
        maker_info,
        taker_info,
    } = fill;
    if let Some(mut adapter) = maker_adapter {
        if let Err(e) = adapter.push_event(*event, Some(maker_info), Some(taker_info)) {
            skip_err!(
                "Failed to push event to adapter {}. Error: {:?}",
                adapter.key(),
                e
            );
        }
    }
    if let Some(mut adapter) = taker_adapter {
        if let Err(e) = adapter.push_event(*event, Some(maker_info), Some(taker_info)) {
            skip_err!(
                "Failed to push event to adapter {}. Error: {:?}",
                adapter.key(),
                e
            );
        }
    }
    let FillEvent {
        taker_side,
        quote_size,
        base_size,
        maker_order_id,
        ..
    } = *event;
    let maker_side = Side::from_u8(taker_side).unwrap().opposite();
    let fill_timestamp = taker_info.order_submitted_timestamp();
    let mut margin_user = maker_info
        .flags
        .contains(CallbackFlags::MARGIN)
        .then(|| maker.margin_user())
        .transpose()?;

    match maker_side {
        Side::Bid => {
            map!(margin_user.assets.reduce_order(quote_size));
            let lend_duration = manager.load()?.lend_duration;
            let maturation_timestamp = fill_timestamp.safe_add(lend_duration)?;
            if maker_info.flags.contains(CallbackFlags::AUTO_STAKE) {
                map!(margin_user.assets.stake_tickets(base_size)?);
                let principal = quote_size;
                let interest = base_size.safe_sub(principal)?;

                **loan.as_mut().unwrap().auto_stake()? = SplitTicket {
                    owner: maker.pubkey(),
                    bond_manager: manager.key(),
                    order_tag: maker_info.order_tag,
                    maturation_timestamp,
                    struck_timestamp: fill_timestamp,
                    principal,
                    interest,
                };
                if let Some(margin_user) = margin_user {
                    emit!(AssetsUpdated::from((
                        &margin_user.assets,
                        margin_user.key()
                    )))
                }
            } else if let Some(mut margin_user) = margin_user {
                margin_user.assets.entitled_tickets += base_size;
                emit!(AssetsUpdated::from((
                    &margin_user.assets,
                    margin_user.key()
                )));
            } else {
                ctx.mint(
                    &ctx.accounts.bond_ticket_mint,
                    maker.as_token_account(),
                    base_size,
                )?;
            }

            emit!(OrderFilled {
                bond_manager: ctx.accounts.bond_manager.key(),
                authority: maker_info.owner,
                order_id: maker_order_id,
                base_filled: base_size,
                quote_filled: quote_size,
                counterparty: None,
                fill_timestamp,
                // Not enough info to be more specific, side matters most
                order_type: crate::events::OrderType::Lend,
                sequence_number: 0,
                maturation_timestamp
            });
        }
        Side::Ask => {
            let mut emit_order_filled = true;
            let mut manager = manager.load_mut()?;
            let maturation_timestamp = fill_timestamp.safe_add(manager.borrow_duration)?;
            if let Some(mut margin_user) = margin_user {
                margin_user.assets.reduce_order(quote_size);
                if maker_info.flags.contains(CallbackFlags::NEW_DEBT) {
                    let disburse = manager.loan_to_disburse(quote_size);
                    manager
                        .collected_fees
                        .try_add_assign(quote_size.safe_sub(disburse)?)?;
                    margin_user
                        .assets
                        .entitled_tokens
                        .try_add_assign(disburse)?;
                    let sequence_number = margin_user
                        .debt
                        .new_obligation_from_fill(base_size, maturation_timestamp)?;

                    let obligation = Obligation {
                        sequence_number,
                        borrower_account: margin_user.key(),
                        bond_manager: ctx.accounts.bond_manager.key(),
                        order_tag: maker_info.order_tag,
                        maturation_timestamp,
                        balance: base_size,
                        flags: ObligationFlags::default(),
                    };

                    let new_debt = loan.as_mut().unwrap().new_debt()?;
                    let obligation_key = new_debt.key();
                    let flags = obligation.flags;
                    **new_debt = obligation;

                    // ObligationCreated includes OrderFill info, thus no OrderFill needed
                    // where OligationCreated is emitted.
                    emit_order_filled = false;
                    emit!(ObligationCreated {
                        obligation: obligation_key,
                        authority: maker_info.owner,
                        order_id: Some(maker_order_id),
                        sequence_number,
                        bond_manager: ctx.accounts.bond_manager.key(),
                        maturation_timestamp,
                        quote_filled: quote_size,
                        base_filled: base_size,
                        flags,
                    });
                    emit!(AssetsUpdated::from((
                        &margin_user.assets,
                        margin_user.key()
                    )));
                    emit!(DebtUpdated::from((&margin_user.debt, margin_user.key())));
                } else {
                    margin_user
                        .assets
                        .entitled_tokens
                        .try_add_assign(quote_size)?;
                    emit!(AssetsUpdated::from((
                        &margin_user.assets,
                        margin_user.key()
                    )));
                }
            } else {
                ctx.withdraw(
                    &ctx.accounts.underlying_token_vault,
                    maker.as_token_account(),
                    quote_size,
                )?;
            }

            if emit_order_filled {
                emit!(OrderFilled {
                    bond_manager: ctx.accounts.bond_manager.key(),
                    authority: maker_info.owner,
                    order_id: maker_order_id,
                    base_filled: base_size,
                    quote_filled: quote_size,
                    counterparty: None,
                    fill_timestamp,
                    order_type: crate::events::OrderType::MarginBorrow,
                    sequence_number: 0,
                    maturation_timestamp
                });
            }
        }
    }

    Ok(())
}

fn handle_out<'info>(
    ctx: &Context<'_, '_, '_, 'info, ConsumeEvents<'info>>,
    accounts: OutAccounts<'info>,
    out: &OutInfo,
) -> Result<()> {
    let OutAccounts {
        user,
        user_adapter_account,
    } = accounts;
    let OutInfo { event, info } = out;
    // push to adapter if flagged
    if let Some(mut adapter) = user_adapter_account {
        if adapter.push_event(*event, Some(info), None).is_err() {
            // don't stop the event processor for a malfunctioning adapter
            // adapter users are responsible for the maintenance of their adapter
            msg!("user adapter failed to push event");
        }
    }
    let OutEvent {
        side,
        order_id,
        base_size,
        ..
    } = event;

    let margin_user = info
        .flags
        .contains(CallbackFlags::MARGIN)
        .then(|| user.margin_user())
        .transpose()?;

    let price = (order_id >> 64) as u64;
    // todo defensive rounding
    let quote_size = fp32_mul(*base_size, price).ok_or(BondsError::ArithmeticOverflow)?;
    match Side::from_u8(*side).unwrap() {
        Side::Bid => {
            if let Some(mut margin_user) = margin_user {
                margin_user.assets.entitled_tokens += quote_size;
                emit!(AssetsUpdated::from((
                    &margin_user.assets,
                    margin_user.key()
                )));
            } else {
                ctx.withdraw(
                    &ctx.accounts.underlying_token_vault,
                    user.as_token_account(),
                    quote_size,
                )?
            }
        }
        Side::Ask => {
            if let Some(mut margin_user) = margin_user {
                if info.flags.contains(CallbackFlags::NEW_DEBT) {
                    margin_user.debt.process_out(*base_size)?;
                    emit!(DebtUpdated::from((&margin_user.debt, margin_user.key())));
                } else {
                    margin_user.assets.entitled_tickets += base_size;
                    emit!(AssetsUpdated::from((
                        &margin_user.assets,
                        margin_user.key()
                    )));
                }
            } else {
                ctx.mint(
                    &ctx.accounts.bond_ticket_mint,
                    user.as_token_account(),
                    *base_size,
                )?
            }
        }
    }

    emit!(OrderRemoved {
        bond_manager: ctx.accounts.bond_manager.key(),
        authority: info.owner,
        order_id: *order_id,
        base_removed: *base_size,
        quote_removed: quote_size,
    });

    Ok(())
}
