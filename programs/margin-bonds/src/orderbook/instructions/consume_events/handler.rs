use agnostic_orderbook::{
    instruction::consume_events,
    state::{
        event_queue::{FillEvent, OutEvent},
        Side,
    },
};
use anchor_lang::prelude::*;
use anchor_spl::token::{MintTo, Token, Transfer};
use jet_proto_math::traits::{SafeAdd, SafeSub, TryAddAssign};
use num_traits::FromPrimitive;

use crate::{
    control::state::BondManager,
    margin::state::{Obligation, ObligationFlags},
    orderbook::state::{fp32_mul, CallbackFlags, CallbackInfo, OrderbookEvent},
    tickets::state::SplitTicket,
    BondsError,
};

use super::{ConsumeEvents, EventAccounts, FillAccounts, OutAccounts, Queue};

pub fn handler<'a, 'b, 'info>(
    ctx: Context<'a, 'b, 'b, 'info, ConsumeEvents<'info>>,
    num_events: u32,
    seeds: Vec<Vec<u8>>,
) -> Result<()> {
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
                handle_fill(
                    &ctx.accounts.bond_manager,
                    &ctx.accounts.underlying_token_vault,
                    &ctx.accounts.bond_ticket_mint,
                    &ctx.accounts.token_program,
                    duration,
                    accounts,
                    &event,
                )
            }
            EventAccounts::Out(accounts) => {
                let event = match event {
                    OrderbookEvent::Out { event, .. } => Ok(event),
                    _ => err!(BondsError::InvalidEvent),
                }?;
                handle_out(
                    &ctx.accounts.bond_manager,
                    &ctx.accounts.underlying_token_vault,
                    &ctx.accounts.bond_ticket_mint,
                    &ctx.accounts.token_program,
                    accounts,
                    &event,
                )
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
        },
        consume_events::Params {
            number_of_entries_to_consume: num_iters,
        },
    )?;

    Ok(())
}

fn handle_fill<'a, 'info>(
    bond_manager: &AccountLoader<'info, BondManager>,
    underlying_vault: &AccountInfo<'info>,
    ticket_mint: &AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    duration: i64,
    mut accounts: Box<FillAccounts<'a, 'info>>,
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
        Some(adapter) => adapter.push_event(*event, Some(&maker.info), Some(&taker.info)),
        None => Ok(()),
    };
    let lender_adapter_res = match lender_adapter_account {
        Some(adapter) => adapter.push_event(*event, Some(&maker.info), Some(&taker.info)),
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
    let (lender, borrower) = match Side::from_u8(*taker_side).unwrap() {
        Side::Bid => (taker, maker),
        Side::Ask => (maker, taker),
    };

    if let Err(e) = borrower_adapter_res {
        // don't stop event processing if adapter errors occur
        msg!(
            "Adapter of borrower account {} failed to push events. Error: {:?}",
            borrower.key,
            e
        )
    }
    if let Err(e) = lender_adapter_res {
        // don't stop event processing if adapter errors occur
        msg!(
            "Adapter of lender account {} failed to push events. Error: {:?}",
            lender.key,
            e
        )
    }

    if let Some(ticket) = auto_stake {
        let principal = *quote_size;
        let interest = base_size.safe_sub(principal)?;

        **ticket = SplitTicket {
            owner: lender.key,
            bond_manager: bond_manager.key(),
            order_tag: lender.info.order_tag,
            maturation_timestamp,
            struck_timestamp: current_time,
            principal,
            interest,
        };
    } else {
        // credit the lender
        anchor_spl::token::mint_to(
            CpiContext::new(
                token_program.to_account_info(),
                MintTo {
                    mint: ticket_mint.to_account_info(),
                    to: lender.vault.to_account_info(),
                    authority: bond_manager.to_account_info(),
                },
            )
            .with_signer(&[&bond_manager.load()?.authority_seeds()]),
            *base_size,
        )?;
    }

    if let Some(obligation) = new_debt {
        borrower
            .borrower_account
            .as_mut()
            .unwrap()
            .outstanding_obligations
            .try_add_assign(1)?;
        **obligation = Obligation {
            borrower_account: borrower.key,
            bond_manager: bond_manager.key(),
            order_tag: borrower.info.order_tag,
            maturation_timestamp,
            balance: *base_size,
            flags: ObligationFlags::default(),
        };
        borrower
            .borrower_account
            .as_mut()
            .unwrap()
            .debt
            .process_fill(*base_size)?;
    }

    anchor_spl::token::transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: underlying_vault.to_account_info(),
                to: borrower.vault.to_account_info(),
                authority: bond_manager.to_account_info(),
            },
        )
        .with_signer(&[&bond_manager.load()?.authority_seeds()]),
        *quote_size,
    )?;

    Ok(())
}

fn handle_out<'a, 'info>(
    bond_manager: &AccountLoader<'info, BondManager>,
    underlying_vault: &AccountInfo<'info>,
    ticket_mint: &AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    mut accounts: Box<OutAccounts<'a, 'info>>,
    event: &OutEvent,
) -> Result<()> {
    let OutAccounts {
        user,
        user_adapter_account,
    } = accounts.as_mut();

    // push to adapter if flagged
    if let Some(adapter) = user_adapter_account {
        if adapter.push_event(*event, Some(&user.info), None).is_err() {
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
            anchor_spl::token::transfer(
                CpiContext::new(
                    token_program.to_account_info(),
                    Transfer {
                        from: underlying_vault.to_account_info(),
                        to: user.vault.to_account_info(),
                        authority: bond_manager.to_account_info(),
                    },
                )
                .with_signer(&[&bond_manager.load()?.authority_seeds()]),
                quote_size,
            )?;
        }
        Side::Ask => {
            if user.info.flags.contains(CallbackFlags::NEW_DEBT) {
                user.borrower_account
                    .as_mut()
                    .unwrap()
                    .debt
                    .process_out(*base_size)?;
            } else {
                anchor_spl::token::mint_to(
                    CpiContext::new(
                        token_program.to_account_info(),
                        MintTo {
                            mint: ticket_mint.to_account_info(),
                            to: user.vault.to_account_info(),
                            authority: bond_manager.to_account_info(),
                        },
                    )
                    .with_signer(&[&bond_manager.load()?.authority_seeds()]),
                    *base_size,
                )?;
            }
        }
    };

    Ok(())
}
