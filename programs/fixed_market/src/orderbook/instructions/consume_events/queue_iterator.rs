use std::slice::Iter;

use anchor_lang::prelude::*;

use crate::{
    events::skip_err,
    margin::state::Obligation,
    orderbook::state::{
        CallbackFlags, CallbackInfo, EventQueue, FillInfo, OrderbookEvent, OutInfo, QueueIterator,
    },
    serialization::RemainingAccounts,
    tickets::state::SplitTicket,
    ErrorCode,
};

use super::{ConsumeEvents, FillAccounts, LoanAccount, OutAccounts, UserAccount};

pub fn queue<'c, 'info>(
    ctx: &Context<'_, '_, 'c, 'info, ConsumeEvents<'info>>,
    seeds: Vec<Vec<u8>>,
) -> Result<EventIterator<'c, 'info>> {
    Ok(EventIterator {
        queue: EventQueue::deserialize_market(ctx.accounts.event_queue.to_account_info())?.iter(),
        accounts: ctx.remaining_accounts.iter(),
        system_program: ctx.accounts.system_program.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        seeds: seeds.into_iter(),
    })
}

pub struct EventIterator<'a, 'info> {
    queue: QueueIterator<'info>,
    accounts: Iter<'a, AccountInfo<'info>>,
    /// CHECK: anchor linter bug requires this
    system_program: AccountInfo<'info>,
    /// CHECK: anchor linter bug requires this
    payer: AccountInfo<'info>,
    seeds: std::vec::IntoIter<Vec<u8>>,
}

impl<'a, 'info> Iterator for EventIterator<'a, 'info> {
    type Item = Result<PreparedEvent<'info>>;

    fn next(&mut self) -> Option<Result<PreparedEvent<'info>>> {
        let event = self.queue.next()?;
        Some(self.join_with_accounts(event))
    }
}

pub enum PreparedEvent<'info> {
    Fill(Box<FillAccounts<'info>>, Box<FillInfo>),
    Out(Box<OutAccounts<'info>>, Box<OutInfo>),
}

impl<'a, 'info> EventIterator<'a, 'info> {
    fn join_with_accounts(&mut self, event: OrderbookEvent) -> Result<PreparedEvent<'info>> {
        Ok(match event {
            OrderbookEvent::Fill(fill) => PreparedEvent::Fill(
                self.extract_fill_accounts(&fill.maker_info, &fill.taker_info)?,
                Box::new(fill),
            ),
            OrderbookEvent::Out(out) => PreparedEvent::Out(
                Box::new(OutAccounts {
                    user: self
                        .accounts
                        .next_user_account(out.info.out_account.to_bytes())?,
                    user_adapter_account: self.accounts.next_adapter_if_needed(&out.info)?,
                }),
                Box::new(out),
            ),
        })
    }

    fn extract_fill_accounts(
        &mut self,
        maker_info: &CallbackInfo,
        taker_info: &CallbackInfo,
    ) -> Result<Box<FillAccounts<'info>>> {
        let maker = self.accounts.next_account()?;
        let maker_adapter = self.accounts.next_adapter_if_needed(maker_info)?;
        let taker_adapter = self.accounts.next_adapter_if_needed(taker_info)?;

        let loan = if maker_info.flags.contains(CallbackFlags::AUTO_STAKE) {
            Some(LoanAccount::AutoStake(
                self.accounts.init_next::<SplitTicket>(
                    self.payer.to_account_info(),
                    self.system_program.to_account_info(),
                    &[
                        crate::seeds::SPLIT_TICKET,
                        &maker_info.fill_account.to_bytes(),
                        &self.seeds.next().ok_or(ErrorCode::InsufficientSeeds)?,
                    ],
                )?,
            ))
        } else if maker_info.flags.contains(CallbackFlags::NEW_DEBT) {
            Some(LoanAccount::NewDebt(
                self.accounts.init_next::<Obligation>(
                    self.payer.to_account_info(),
                    self.system_program.to_account_info(),
                    &[
                        crate::seeds::OBLIGATION,
                        &maker_info.fill_account.to_bytes(),
                        &self.seeds.next().ok_or(ErrorCode::InsufficientSeeds)?,
                    ],
                )?,
            ))
        } else {
            None
        };
        Ok(Box::new(FillAccounts {
            maker: UserAccount::new(maker.clone()),
            loan,
            maker_adapter,
            taker_adapter,
        }))
    }
}

pub trait UserAccounts<'a, 'info: 'a>: RemainingAccounts<'a, 'info> {
    fn next_user_account(&mut self, expected: [u8; 32]) -> Result<UserAccount<'info>> {
        let account = self.next_account()?;
        if account.key().to_bytes() != expected {
            msg!(
                "Provided user account {} does not match the callback info {}",
                account.key(),
                Pubkey::new_from_array(expected)
            );
            return err!(ErrorCode::WrongUserAccount);
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
                    require_eq!(key, adapter.key(), ErrorCode::WrongAdapter);
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
