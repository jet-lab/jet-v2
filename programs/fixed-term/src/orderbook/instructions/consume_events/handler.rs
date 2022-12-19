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
    events::{skip_err, OrderFilled, OrderRemoved, TermLoanCreated},
    margin::state::{TermLoan, TermLoanFlags},
    market_token_manager::MarketTokenManager,
    orderbook::state::{fp32_mul, CallbackFlags, CallbackInfo, FillInfo, OutInfo},
    tickets::state::TermDeposit,
    FixedTermErrorCode,
};

use super::{queue, ConsumeEvents, FillAccounts, OutAccounts, PreparedEvent};

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, ConsumeEvents<'info>>,
    num_events: u32,
    seed: Vec<u8>,
) -> Result<()> {
    let mut num_iters = 0;
    for event in queue(&ctx, seed)?.take(num_events as usize) {
        match event? {
            PreparedEvent::Fill(mut accounts, info) => handle_fill(&ctx, &mut accounts, &info),
            PreparedEvent::Out(mut accounts, info) => handle_out(&ctx, &mut accounts, &info),
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

#[inline(never)]
fn handle_fill<'info>(
    ctx: &Context<'_, '_, '_, 'info, ConsumeEvents<'info>>,
    accounts: &mut FillAccounts<'info>,
    fill: &FillInfo,
) -> Result<()> {
    let market = &ctx.accounts.market;
    let FillAccounts {
        maker,
        maker_adapter,
        taker_adapter,
        loan,
    } = accounts;
    let FillInfo {
        event,
        maker_info,
        taker_info,
    } = fill;
    if let Some(adapter) = maker_adapter {
        if let Err(e) = adapter.push_event(*event, Some(maker_info), Some(taker_info)) {
            skip_err!(
                "Failed to push event to adapter {}. Error: {:?}",
                adapter.key(),
                e
            );
        }
    }
    if let Some(adapter) = taker_adapter {
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

    match maker_side {
        Side::Bid => {
            let maturation_timestamp = fill_timestamp.safe_add(market.load()?.lend_tenor)?;
            if maker_info.flags.contains(CallbackFlags::AUTO_STAKE) {
                let matures_at = fill_timestamp.safe_add(market.load()?.lend_tenor)?;
                let mut sequence_number = 0;

                if maker_info.flags.contains(CallbackFlags::MARGIN) {
                    let mut margin_user = maker.margin_user()?;
                    margin_user.assets.reduce_order(quote_size);
                    sequence_number = margin_user.assets.new_deposit(base_size)?;
                    margin_user.emit_asset_balances();
                }

                **loan.as_mut().unwrap().auto_stake()? = TermDeposit {
                    matures_at,
                    sequence_number,
                    principal: quote_size,
                    amount: base_size,
                    owner: maker.pubkey(),
                    market: market.key(),
                };
            } else if maker_info.flags.contains(CallbackFlags::MARGIN) {
                let mut margin_user = maker.margin_user()?;
                margin_user.assets.reduce_order(quote_size);
                margin_user.assets.entitled_tickets += base_size;
                margin_user.emit_asset_balances();
            } else {
                ctx.mint(
                    &ctx.accounts.ticket_mint,
                    maker.as_token_account(),
                    base_size,
                )?;
            }

            emit!(OrderFilled {
                market: ctx.accounts.market.key(),
                maker_authority: maker_info.owner,
                taker_authority: taker_info.owner,
                maker_order_tag: maker_info.order_tag.as_u128(),
                taker_order_tag: taker_info.order_tag.as_u128(),
                base_filled: base_size,
                quote_filled: quote_size,
                fill_timestamp,
                // Not enough info to be more specific, side matters most
                order_type: crate::events::OrderType::Lend,
                sequence_number: 0,
                maturation_timestamp
            });
        }
        Side::Ask => {
            let maturation_timestamp = fill_timestamp.safe_add(market.load()?.borrow_tenor)?;
            if maker_info.flags.contains(CallbackFlags::MARGIN) {
                let mut margin_user = maker.margin_user()?;
                margin_user.assets.reduce_order(quote_size);
                if maker_info.flags.contains(CallbackFlags::NEW_DEBT) {
                    let mut manager = market.load_mut()?;
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
                        .new_term_loan_from_fill(base_size, maturation_timestamp)?;

                    let flags = TermLoanFlags::default();

                    let term_loan = loan.as_mut().unwrap().new_debt()?;
                    **term_loan = TermLoan {
                        sequence_number,
                        margin_user: margin_user.key(),
                        market: ctx.accounts.market.key(),
                        order_tag: maker_info.order_tag,
                        maturation_timestamp,
                        balance: base_size,
                        flags,
                    };

                    // TermLoanCreated includes OrderFill info, thus no OrderFill needed
                    // where TermLoanCreated is emitted.
                    emit!(TermLoanCreated {
                        term_loan: term_loan.key(),
                        authority: maker_info.owner,
                        order_tag: maker_info.order_tag.as_u128(),
                        sequence_number,
                        market: ctx.accounts.market.key(),
                        maturation_timestamp,
                        quote_filled: quote_size,
                        base_filled: base_size,
                        flags,
                    });
                    margin_user.emit_all_balances();
                } else {
                    margin_user
                        .assets
                        .entitled_tokens
                        .try_add_assign(quote_size)?;
                    margin_user.emit_asset_balances();
                }
            } else {
                ctx.withdraw(
                    &ctx.accounts.underlying_token_vault,
                    maker.as_token_account(),
                    quote_size,
                )?;
            }

            emit!(OrderFilled {
                market: ctx.accounts.market.key(),
                maker_authority: maker_info.owner,
                taker_authority: taker_info.owner,
                maker_order_tag: maker_info.order_tag.as_u128(),
                taker_order_tag: taker_info.order_tag.as_u128(),
                base_filled: base_size,
                quote_filled: quote_size,
                fill_timestamp,
                // We can be more specific with the type
                order_type: crate::events::OrderType::MarginBorrow,
                sequence_number: 0,
                maturation_timestamp
            });
        }
    }

    Ok(())
}

#[inline(never)]
fn handle_out<'info>(
    ctx: &Context<'_, '_, '_, 'info, ConsumeEvents<'info>>,
    accounts: &mut OutAccounts<'info>,
    out: &OutInfo,
) -> Result<()> {
    let OutAccounts {
        user,
        user_adapter_account,
    } = accounts;
    let OutInfo { event, info } = out;
    // push to adapter if flagged
    if let Some(adapter) = user_adapter_account {
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

    let price = (order_id >> 64) as u64;
    // todo defensive rounding
    let quote_size = fp32_mul(*base_size, price).ok_or(FixedTermErrorCode::ArithmeticOverflow)?;
    match Side::from_u8(*side).unwrap() {
        Side::Bid => {
            if info.flags.contains(CallbackFlags::MARGIN) {
                let mut margin_user = user.margin_user()?;
                margin_user.assets.entitled_tokens += quote_size;
                margin_user.emit_asset_balances();
            } else {
                ctx.withdraw(
                    &ctx.accounts.underlying_token_vault,
                    user.as_token_account(),
                    quote_size,
                )?;
            }
        }
        Side::Ask => {
            if info.flags.contains(CallbackFlags::MARGIN) {
                let mut margin_user = user.margin_user()?;

                if info.flags.contains(CallbackFlags::NEW_DEBT) {
                    margin_user.debt.process_out(*base_size)?;
                    margin_user.emit_debt_balances();
                } else {
                    margin_user.assets.entitled_tickets += base_size;
                    margin_user.emit_asset_balances();
                }
            } else {
                ctx.mint(
                    &ctx.accounts.ticket_mint,
                    user.as_token_account(),
                    *base_size,
                )?;
            }
        }
    }

    emit!(OrderRemoved {
        market: ctx.accounts.market.key(),
        authority: info.owner,
        order_tag: info.order_tag.as_u128(),
        base_removed: *base_size,
        quote_removed: quote_size,
    });

    Ok(())
}
