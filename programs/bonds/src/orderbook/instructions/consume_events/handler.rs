use agnostic_orderbook::{
    instruction::consume_events,
    state::{
        event_queue::{FillEvent, OutEvent},
        Side,
    },
};
use anchor_lang::prelude::*;
use jet_proto_math::traits::{SafeAdd, SafeSub, TryAddAssign};
use num_traits::FromPrimitive;

use crate::{
    orderbook::state::{
        debt::{Obligation, ObligationFlags},
        event_queue::OrderbookEvent,
        fp32_mul, CallbackFlags, CallbackInfo,
    },
    tickets::state::SplitTicket,
    BondsError,
};

use super::{lender_borrower, ConsumeEvents, EventAccounts, FillAccounts, OutAccounts, Queue};

pub fn handler<'a, 'b, 'info>(
    ctx: Context<'a, 'b, 'b, 'info, ConsumeEvents<'info>>,
    num_events: u32,
    seeds: Vec<Vec<u8>>,
) -> Result<()> {
    let bond_manager = ctx.accounts.bond_manager.key();
    let duration = ctx.accounts.bond_manager.load()?.duration;

    let mut num_iters = 0;
    for event in ctx.queue(seeds)?.iter().take(num_events as usize) {
        let (accounts, event) = event?;

        // Delegate event processing to the appropriate handler
        match accounts {
            EventAccounts::Fill(accounts) => {
                let event = match event {
                    OrderbookEvent::Fill { event, .. } => Ok(event),
                    _ => err!(BondsError::InvalidEvent),
                }?;
                handle_fill(bond_manager, duration, accounts, &event)
            }
            EventAccounts::Out(accounts) => {
                let event = match event {
                    OrderbookEvent::Out { event, .. } => Ok(event),
                    _ => err!(BondsError::InvalidEvent),
                }?;
                handle_out(accounts, &event)
            }
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
            reward_target: &ctx.accounts.bond_manager.to_account_info(),
        },
        consume_events::Params {
            number_of_entries_to_consume: num_iters,
        },
    )?;

    Ok(())
}

fn handle_fill(
    bond_manager: Pubkey,
    duration: i64,
    mut accounts: Box<FillAccounts>,
    event: &FillEvent,
) -> Result<()> {
    let FillAccounts {
        maker,
        taker,
        auto_stake,
        new_debt,
        borrower_adapter_account,
        lender_adapter_account,
    } = accounts.as_mut();

    let borrower_adapter_res = match borrower_adapter_account {
        Some(adapter) => adapter.push_event(*event, Some(&maker.callback), Some(&taker.callback)),
        None => Ok(()),
    };
    let lender_adapter_res = match lender_adapter_account {
        Some(adapter) => adapter.push_event(*event, Some(&maker.callback), Some(&taker.callback)),
        None => Ok(()),
    };

    let FillEvent {
        taker_side,
        quote_size,
        base_size,
        ..
    } = event;

    let current_time = Clock::get()?.unix_timestamp;
    let maturation_timestamp = duration.safe_add(current_time)?;
    let (lender, borrower) = lender_borrower(*taker_side, maker, taker);

    if let Err(e) = borrower_adapter_res {
        // don't stop event processing if adapter errors occur
        msg!(
            "Adapter of account {} failed to push events. Error: {:?}",
            borrower.account.key(),
            e
        )
    }
    if let Err(e) = lender_adapter_res {
        // don't stop event processing if adapter errors occur
        msg!(
            "Adapter of account {} failed to push events. Error: {:?}",
            lender.account.key(),
            e
        )
    }

    if let Some(ticket) = auto_stake {
        let principal = *quote_size;
        let interest = base_size.safe_sub(principal)?;

        **ticket = SplitTicket {
            owner: lender.account.key(),
            bond_manager,
            order_tag: lender.callback.order_tag,
            maturation_timestamp,
            struck_timestamp: current_time,
            principal,
            interest,
        };
    } else {
        // credit the lender
        lender
            .account
            .bond_tickets_stored
            .try_add_assign(*base_size)?;
    }

    if let Some(obligation) = new_debt {
        borrower.account.outstanding_obligations.try_add_assign(1)?;
        **obligation = Obligation {
            orderbook_user_account: borrower.account.key(),
            bond_manager,
            order_tag: borrower.callback.order_tag,
            maturation_timestamp,
            balance: *base_size,
            flags: ObligationFlags::default(),
        };
        borrower.account.debt.commit(*quote_size)?;
    }

    borrower
        .account
        .underlying_token_stored
        .try_add_assign(*quote_size)?;

    Ok(())
}

fn handle_out(mut accounts: Box<OutAccounts>, event: &OutEvent) -> Result<()> {
    let OutAccounts {
        user,
        user_adapter_account,
    } = accounts.as_mut();

    // push to adapter if flagged
    if let Some(adapter) = user_adapter_account {
        if adapter
            .push_event(*event, Some(&user.callback), None)
            .is_err()
        {
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
    let quote_size = fp32_mul(*base_size, price).ok_or(BondsError::ArithmeticOverflow)?;
    match Side::from_u8(*side).unwrap() {
        Side::Bid => {
            user.account
                .underlying_token_stored
                .try_add_assign(quote_size)?;
        }
        Side::Ask => {
            if user.callback.flags.contains(CallbackFlags::NEW_DEBT) {
                user.account.debt.cancel_pending(quote_size)?;
            } else {
                user.account
                    .underlying_token_stored
                    .try_add_assign(*base_size)?;
            }
        }
    };

    Ok(())
}
