use agnostic_orderbook::{
    instruction::consume_events,
    state::{
        event_queue::{FillEvent, OutEvent},
        Side,
    },
};
use anchor_lang::prelude::*;
use num_traits::FromPrimitive;

use jet_program_common::traits::{SafeAdd, SafeSub};

use crate::{
    control::state::Market,
    events::{OrderFilled, OrderRemoved, OrderType, TermLoanCreated},
    margin::state::{MarginUser, TermLoan, TermLoanFlags},
    market_token_manager::MarketTokenManager,
    orderbook::state::{
        CallbackFlags, CallbackInfo, EventQuote, FillInfo, MarketSide, OutInfo, UserCallbackInfo,
    },
    serialization::{AnchorAccount, Mut},
    tickets::state::TermDepositWriter,
    FixedTermErrorCode,
};

use super::{
    queue, ConsumeEvents, FillAccount, FillAccounts, MarginFillAccounts, OutAccounts, PreparedEvent,
};

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, ConsumeEvents<'info>>,
    num_events: u32,
    seed: Vec<u8>,
) -> Result<()> {
    let mut num_iters = 0;

    for event in queue(&ctx, seed)?.take(num_events as usize) {
        match event? {
            PreparedEvent::Fill(accounts, info) => handle_fill(&ctx, accounts, info)?,
            PreparedEvent::Out(accounts, info) => handle_out(&ctx, accounts, info)?,
        }

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
    accounts: FillAccounts<'info>,
    fill: FillInfo,
) -> Result<()> {
    match accounts {
        FillAccounts::Margin(accs) => handle_margin_fill(
            ctx,
            &ctx.accounts.market,
            accs,
            fill,
            ctx.accounts.payer.key(),
        ),
        FillAccounts::Signer(accs) => handle_signer_fill(ctx, accs, fill),
    }
}

#[inline(never)]
fn handle_margin_fill<'info>(
    ctx: &Context<'_, '_, '_, 'info, ConsumeEvents<'info>>,
    market: &AccountLoader<'info, Market>,
    mut accounts: MarginFillAccounts<'info>,
    info: FillInfo,
    payer: Pubkey,
) -> Result<()> {
    let FillInfo {
        event,
        maker_info,
        taker_info,
    } = info;

    let FillEvent {
        taker_side,
        base_size,
        ..
    } = event;

    let quote_size = event.quote_size()?;
    let maker_side: MarketSide = Side::from_u8(taker_side).unwrap().opposite().into();
    let user = &mut accounts.margin_user;
    let info = maker_info.unwrap_margin();

    let (order_type, sequence_number, tenor) = match maker_side {
        // maker has loaned tokens to the taker
        MarketSide::Lend => {
            let tenor = market.load()?.lend_tenor;
            let sequence_number = if let Some(term_account) = &mut accounts.term_account {
                let sequence_number = user.maker_fill_lend_order(true, base_size)?;
                TermDepositWriter {
                    market: user.market,
                    owner: user.margin_account,
                    payer,
                    order_tag: info.order_tag.as_u128(),
                    tenor,
                    sequence_number,
                    amount: base_size,
                    principal: quote_size,
                    flags: info.flags.into(),
                    seed: vec![], // account already initialized by the queue iterator,
                }
                .write(term_account.term_deposit()?)?;

                sequence_number
            } else {
                user.maker_fill_lend_order(false, base_size)?;
                0
            };
            user.emit_asset_balances()?;

            (OrderType::MarginLend, sequence_number, tenor)
        }

        // maker has borrowed tokens from the taker
        MarketSide::Borrow => {
            let (tenor, disburse) = {
                let market = market.load()?;
                let tenor = market.borrow_tenor;
                let disburse = market.loan_to_disburse(quote_size);

                (tenor, disburse)
            };
            let strike_timestamp = Clock::get()?.unix_timestamp;
            let maturation_timestamp = strike_timestamp.safe_add(tenor as i64)?;

            let sequence_number = if let Some(term_account) = accounts.term_account {
                let fees = quote_size.safe_sub(disburse)?;

                ctx.withdraw(
                    &ctx.accounts.underlying_token_vault,
                    &ctx.accounts.fee_vault,
                    fees,
                )?;

                let sequence_number = user.maker_fill_borrow_order(
                    true,
                    disburse,
                    quote_size,
                    base_size,
                    maturation_timestamp,
                )?;
                let mut flags = TermLoanFlags::empty();

                if info.flags.contains(CallbackFlags::AUTO_ROLL) {
                    flags |= TermLoanFlags::AUTO_ROLL;
                }

                let mut loan = term_account.term_loan()?;
                *loan = TermLoan {
                    sequence_number,
                    margin_user: user.key(),
                    market: user.market,
                    payer,
                    order_tag: info.order_tag,
                    maturation_timestamp,
                    strike_timestamp,
                    principal: quote_size,
                    interest: base_size.safe_sub(quote_size)?,
                    balance: base_size,
                    flags,
                };

                // TermLoanCreated includes OrderFill info, thus no OrderFill needed
                // where TermLoanCreated is emitted.
                emit!(TermLoanCreated {
                    term_loan: loan.key(),
                    authority: info.margin_account,
                    payer,
                    order_tag: info.order_tag.as_u128(),
                    sequence_number,
                    market: user.market,
                    maturation_timestamp,
                    quote_filled: quote_size,
                    base_filled: base_size,
                    fees,
                    flags,
                });
                user.emit_all_balances()?;

                sequence_number
            } else {
                user.maker_fill_borrow_order(
                    false,
                    disburse,
                    quote_size,
                    base_size,
                    maturation_timestamp,
                )?;
                user.emit_asset_balances()?;
                0
            };

            (OrderType::MarginBorrow, sequence_number, tenor)
        }
    };

    let (taker_authority, taker_order_tag, fill_timestamp) = match taker_info {
        UserCallbackInfo::Margin(info) => (
            info.margin_account,
            info.order_tag.as_u128(),
            info.order_submitted,
        ),
        UserCallbackInfo::Signer(info) => {
            (info.signer, info.order_tag.as_u128(), info.order_submitted)
        }
    };
    emit!(OrderFilled {
        market: user.market,
        maker_authority: info.margin_account,
        taker_authority,
        maker_order_tag: info.order_tag.as_u128(),
        taker_order_tag,
        order_type,
        sequence_number,
        base_filled: base_size,
        quote_filled: quote_size,
        fill_timestamp,
        maturation_timestamp: fill_timestamp.safe_add(tenor as i64)?,
    });
    Ok(())
}

#[inline(never)]
fn handle_signer_fill<'info>(
    ctx: &Context<'_, '_, '_, 'info, ConsumeEvents<'info>>,
    account: FillAccount<'info>,
    info: FillInfo,
) -> Result<()> {
    let FillInfo {
        event,
        maker_info,
        taker_info,
    } = info;

    let FillEvent {
        taker_side,
        base_size,
        ..
    } = event;

    let quote_size = event.quote_size()?;
    let maker_side: MarketSide = Side::from_u8(taker_side).unwrap().opposite().into();
    let info = maker_info.unwrap_signer();

    let (order_type, tenor) = match maker_side {
        MarketSide::Lend => {
            let tenor = ctx.accounts.market.load()?.lend_tenor;
            match account {
                FillAccount::TermDeposit(mut deposit) => {
                    TermDepositWriter {
                        market: ctx.accounts.market.key(),
                        owner: info.signer,
                        payer: ctx.accounts.payer.key(),
                        order_tag: info.order_tag.as_u128(),
                        tenor,
                        sequence_number: 0,
                        amount: base_size,
                        principal: quote_size,
                        flags: info.flags.into(),
                        seed: vec![], // account initialized by queue iterator
                    }
                    .write(&mut deposit)?;
                }
                FillAccount::Token(token_account) => {
                    ctx.mint(&ctx.accounts.ticket_mint, token_account, base_size)?;
                }
            }

            (OrderType::Lend, tenor)
        }
        MarketSide::Borrow => {
            ctx.withdraw(
                &ctx.accounts.underlying_token_vault,
                account.as_token_account(),
                quote_size,
            )?;

            (
                OrderType::SellTickets,
                ctx.accounts.market.load()?.borrow_tenor,
            )
        }
    };

    let (taker_authority, taker_order_tag, fill_timestamp) = match taker_info {
        UserCallbackInfo::Margin(info) => (
            info.margin_account,
            info.order_tag.as_u128(),
            info.order_submitted,
        ),
        UserCallbackInfo::Signer(info) => {
            (info.signer, info.order_tag.as_u128(), info.order_submitted)
        }
    };

    emit!(OrderFilled {
        market: ctx.accounts.market.key(),
        maker_authority: info.signer,
        taker_authority,
        maker_order_tag: info.order_tag.as_u128(),
        taker_order_tag,
        order_type,
        sequence_number: 0,
        base_filled: base_size,
        quote_filled: quote_size,
        fill_timestamp,
        maturation_timestamp: fill_timestamp.safe_add(tenor as i64)?
    });

    Ok(())
}

#[inline(never)]
fn handle_out<'info>(
    ctx: &Context<'_, '_, '_, 'info, ConsumeEvents<'info>>,
    accounts: OutAccounts<'info>,
    out: OutInfo,
) -> Result<()> {
    match accounts {
        OutAccounts::Margin(user) => handle_margin_out(ctx, user, out),
        OutAccounts::Signer(user) => handle_signer_out(ctx, user, out),
    }
}

#[inline(never)]
fn handle_margin_out<'info>(
    ctx: &Context<'_, '_, '_, 'info, ConsumeEvents<'info>>,
    mut user: AnchorAccount<'info, MarginUser, Mut>,
    out: OutInfo,
) -> Result<()> {
    let OutInfo { event, info } = out;
    let OutEvent {
        side, base_size, ..
    } = event;

    let info = info.unwrap_margin();
    let quote_size = event.quote_size()?;
    let side: MarketSide = Side::from_u8(side).unwrap().into();

    match side {
        MarketSide::Borrow => user.cancel_borrow_order(
            quote_size,
            base_size,
            info.flags.contains(CallbackFlags::NEW_DEBT),
        )?,
        MarketSide::Lend => user.cancel_lend_order(quote_size, base_size)?,
    }

    emit!(OrderRemoved {
        market: ctx.accounts.market.key(),
        authority: info.margin_account,
        order_tag: info.order_tag.as_u128(),
        base_removed: base_size,
        quote_removed: quote_size
    });
    Ok(())
}

#[inline(never)]
fn handle_signer_out<'info>(
    ctx: &Context<'_, '_, '_, 'info, ConsumeEvents<'info>>,
    user: AccountInfo<'info>,
    out: OutInfo,
) -> Result<()> {
    let OutInfo { event, info } = out;
    let OutEvent {
        side, base_size, ..
    } = event;

    let info = info.unwrap_signer();
    let quote_size = event.quote_size()?;

    match Side::from_u8(side).unwrap() {
        Side::Bid => ctx.withdraw(&ctx.accounts.underlying_token_vault, user, quote_size)?,
        Side::Ask => ctx.mint(&ctx.accounts.ticket_mint, user, base_size)?,
    }

    emit!(OrderRemoved {
        market: ctx.accounts.market.key(),
        authority: info.signer,
        order_tag: info.order_tag.as_u128(),
        base_removed: base_size,
        quote_removed: quote_size
    });
    Ok(())
}
