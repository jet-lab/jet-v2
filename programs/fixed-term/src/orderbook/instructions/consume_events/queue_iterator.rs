use std::slice::Iter;

use anchor_lang::prelude::*;

use crate::{
    events::skip_err,
    margin::state::TermLoan,
    orderbook::state::{
        CallbackFlags, CallbackInfo, EventQueue, FillInfo, OrderbookEvent, OutInfo, QueueIterator,
    },
    serialization::RemainingAccounts,
    tickets::state::TermDeposit,
    FixedTermErrorCode,
};

use super::{ConsumeEvents, FillAccounts, LoanAccount, OutAccounts, UserAccount};

pub fn queue<'c, 'info>(
    ctx: &Context<'_, '_, 'c, 'info, ConsumeEvents<'info>>,
    seed: Vec<u8>,
) -> Result<EventIterator<'c, 'info>> {
    Ok(EventIterator {
        queue: EventQueue::deserialize_market(ctx.accounts.event_queue.to_account_info())?.iter(),
        accounts: ctx.remaining_accounts.iter(),
        system_program: ctx.accounts.system_program.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        market: ctx.accounts.market.key(),
        seed,
    })
}

pub struct EventIterator<'a, 'info> {
    queue: QueueIterator<'info>,
    accounts: Iter<'a, AccountInfo<'info>>,
    system_program: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    market: Pubkey,
    seed: Vec<u8>,
}

impl<'a, 'info> Iterator for EventIterator<'a, 'info> {
    type Item = Result<PreparedEvent<'info>>;

    fn next(&mut self) -> Option<Result<PreparedEvent<'info>>> {
        let event = self.queue.next()?;
        Some(self.join_with_accounts(event))
    }
}

#[allow(clippy::large_enum_variant)]
pub enum PreparedEvent<'info> {
    Fill(FillAccounts<'info>, FillInfo),
    Out(OutAccounts<'info>, OutInfo),
}

impl<'a, 'info> EventIterator<'a, 'info> {
    fn join_with_accounts(&mut self, event: OrderbookEvent) -> Result<PreparedEvent<'info>> {
        Ok(match event {
            OrderbookEvent::Fill(fill) => PreparedEvent::Fill(
                self.extract_fill_accounts(&fill.maker_info, &fill.taker_info)?,
                fill,
            ),
            OrderbookEvent::Out(out) => PreparedEvent::Out(
                OutAccounts {
                    user: self.accounts.next_user_account(out.info.out_account)?,
                    user_adapter_account: self.accounts.next_adapter_if_needed(&out.info)?,
                },
                out,
            ),
        })
    }

    fn extract_fill_accounts(
        &mut self,
        maker_info: &CallbackInfo,
        taker_info: &CallbackInfo,
    ) -> Result<FillAccounts<'info>> {
        let maker = self.accounts.next_user_account(maker_info.fill_account)?;
        let maker_adapter = self.accounts.next_adapter_if_needed(maker_info)?;
        let taker_adapter = self.accounts.next_adapter_if_needed(taker_info)?;

        let mut seed = [0u8; 8];

        let loan = if maker_info.flags.contains(CallbackFlags::AUTO_STAKE) {
            match maker.margin_user() {
                Ok(user) => {
                    seed[..8].copy_from_slice(&user.assets.next_deposit_seqno.to_le_bytes())
                }

                Err(_) => seed[..self.seed.len()].copy_from_slice(&self.seed),
            };

            Some(LoanAccount::AutoStake(
                self.accounts.init_next::<TermDeposit>(
                    self.payer.to_account_info(),
                    self.system_program.to_account_info(),
                    &[
                        crate::seeds::TERM_DEPOSIT,
                        self.market.as_ref(),
                        &maker_info.fill_account.to_bytes(),
                        &seed,
                    ],
                )?,
            ))
        } else if maker_info.flags.contains(CallbackFlags::NEW_DEBT) {
            match maker.margin_user() {
                Ok(user) => {
                    seed[..8].copy_from_slice(&user.debt.next_new_term_loan_seqno.to_le_bytes());
                }

                Err(_) => seed[..self.seed.len()].copy_from_slice(&self.seed),
            };

            Some(LoanAccount::NewDebt(self.accounts.init_next::<TermLoan>(
                self.payer.to_account_info(),
                self.system_program.to_account_info(),
                &[
                    crate::seeds::TERM_LOAN,
                    self.market.as_ref(),
                    &maker_info.fill_account.to_bytes(),
                    &seed,
                ],
            )?))
        } else {
            None
        };
        Ok(FillAccounts {
            maker,
            loan,
            maker_adapter,
            taker_adapter,
        })
    }
}

pub trait UserAccounts<'a, 'info: 'a>: RemainingAccounts<'a, 'info> {
    fn next_user_account(&mut self, expected: Pubkey) -> Result<UserAccount<'info>> {
        let account = self.next_account()?;
        if account.key() != expected {
            msg!(
                "Provided user account {} does not match the callback info {}",
                account.key(),
                expected
            );
            return err!(FixedTermErrorCode::WrongUserAccount);
        }
        Ok(UserAccount::new(account.clone()))
    }

    fn next_adapter_if_needed(
        &mut self,
        callback_info: &CallbackInfo,
    ) -> Result<Option<EventQueue<'info>>> {
        if let Some(key) = callback_info.adapter() {
            match self.next_adapter() {
                Ok(adapter) => {
                    // this needs to fail the ix because it means the crank passed the wrong account
                    require_eq!(key, adapter.key(), FixedTermErrorCode::WrongAdapter);
                    Ok(Some(adapter))
                }
                Err(e) => {
                    // this should not fail the ix because it means the crank did everything right
                    // but the user's adapter is just not usable
                    skip_err!(
                        "expected adapter account could not be deserialized as an adapter: {}",
                        e
                    );
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }
}
impl<'a, 'info: 'a, T: RemainingAccounts<'a, 'info>> UserAccounts<'a, 'info> for T {}
