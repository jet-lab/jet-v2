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
    events::skip_err,
    margin::state::{TermLoan, TermLoanFlags},
    market_token_manager::MarketTokenManager,
    orderbook::state::{fp32_mul, CallbackFlags, CallbackInfo, FillInfo, OutInfo},
    tickets::state::SplitTicket,
    utils::map,
    FixedTermErrorCode,
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
        return err!(FixedTermErrorCode::NoEvents);
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
    let manager = &ctx.accounts.market;
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
            if maker_info.flags.contains(CallbackFlags::AUTO_STAKE) {
                map!(margin_user.assets.stake_tickets(base_size)?);
                let principal = quote_size;
                let interest = base_size.safe_sub(principal)?;
                let maturation_timestamp = fill_timestamp.safe_add(manager.load()?.lend_tenor)?;
                **loan.as_mut().unwrap().auto_stake()? = SplitTicket {
                    owner: maker.pubkey(),
                    market: manager.key(),
                    order_tag: maker_info.order_tag,
                    maturation_timestamp,
                    struck_timestamp: fill_timestamp,
                    principal,
                    interest,
                };
            } else if let Some(mut margin_user) = margin_user {
                margin_user.assets.entitled_tickets += base_size;
            } else {
                ctx.mint(
                    &ctx.accounts.ticket_mint,
                    maker.as_token_account(),
                    base_size,
                )?;
            }
        }
        Side::Ask => {
            if let Some(mut margin_user) = margin_user {
                margin_user.assets.reduce_order(quote_size);
                if maker_info.flags.contains(CallbackFlags::NEW_DEBT) {
                    let mut manager = manager.load_mut()?;
                    let disburse = manager.loan_to_disburse(quote_size);
                    manager
                        .collected_fees
                        .try_add_assign(quote_size.safe_sub(disburse)?)?;
                    margin_user
                        .assets
                        .entitled_tokens
                        .try_add_assign(disburse)?;
                    let maturation_timestamp = fill_timestamp.safe_add(manager.borrow_tenor)?;
                    let sequence_number = margin_user
                        .debt
                        .new_term_loan_from_fill(base_size, maturation_timestamp)?;

                    **loan.as_mut().unwrap().new_debt()? = TermLoan {
                        sequence_number,
                        borrower_account: margin_user.key(),
                        market: ctx.accounts.market.key(),
                        order_tag: maker_info.order_tag,
                        maturation_timestamp,
                        balance: base_size,
                        flags: TermLoanFlags::default(),
                    };
                } else {
                    margin_user
                        .assets
                        .entitled_tokens
                        .try_add_assign(quote_size)?;
                }
            } else {
                ctx.withdraw(
                    &ctx.accounts.underlying_token_vault,
                    maker.as_token_account(),
                    quote_size,
                )?;
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
    let quote_size = fp32_mul(*base_size, price).ok_or(FixedTermErrorCode::ArithmeticOverflow)?;
    match Side::from_u8(*side).unwrap() {
        Side::Bid => {
            if let Some(mut margin_user) = margin_user {
                margin_user.assets.entitled_tokens += quote_size;
                Ok(())
            } else {
                ctx.withdraw(
                    &ctx.accounts.underlying_token_vault,
                    user.as_token_account(),
                    quote_size,
                )
            }
        }
        Side::Ask => {
            if let Some(mut margin_user) = margin_user {
                if info.flags.contains(CallbackFlags::NEW_DEBT) {
                    margin_user.debt.process_out(*base_size)
                } else {
                    margin_user.assets.entitled_tickets += base_size;
                    Ok(())
                }
            } else {
                ctx.mint(
                    &ctx.accounts.ticket_mint,
                    user.as_token_account(),
                    *base_size,
                )
            }
        }
    }
}
