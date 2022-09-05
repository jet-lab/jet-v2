use std::slice::Iter;

use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use num_traits::FromPrimitive;

use crate::{
    orderbook::state::{
        debt::Obligation,
        event_queue::{EventQueue, OrderbookEvent, QueueIterator},
        user::OrderbookUser,
        CallbackFlags, CallbackInfo,
    },
    serialization::{Mut, RemainingAccounts},
    tickets::state::SplitTicket,
    BondsError,
};

use super::{
    auto_stake, new_debt, ConsumeEvents, EventAccounts, FillAccounts, OutAccounts, UserData,
};

pub trait Queue<'c, 'info> {
    fn queue(&self, seeds: Vec<Vec<u8>>) -> Result<EventQueueOwner<'c, 'info>>;
}

impl<'c, 'info> Queue<'c, 'info> for Context<'_, '_, 'c, 'info, ConsumeEvents<'info>> {
    fn queue(&self, seeds: Vec<Vec<u8>>) -> Result<EventQueueOwner<'c, 'info>> {
        EventQueueOwner::new(
            self.accounts.event_queue.to_account_info(),
            self.remaining_accounts,
            self.accounts.system_program.to_account_info(),
            self.accounts.payer.to_account_info(),
            seeds,
        )
    }
}

pub struct EventQueueOwner<'a, 'info> {
    queue: EventQueue<'info>,
    accounts: &'a [AccountInfo<'info>],
    /// CHECK: anchor linter bug requires this
    system_program: AccountInfo<'info>,
    /// CHECK: anchor linter bug requires this
    payer: AccountInfo<'info>,
    seeds: Vec<Vec<u8>>,
}

impl<'a, 'info> EventQueueOwner<'a, 'info> {
    pub fn new(
        queue_info: AccountInfo<'info>,
        remaining_accounts: &'a [AccountInfo<'info>],
        system_program: AccountInfo<'info>,
        payer: AccountInfo<'info>,
        seeds: Vec<Vec<u8>>,
    ) -> Result<Self> {
        let queue = EventQueue::from_data(queue_info.data.clone())?;
        Ok(Self {
            queue,
            accounts: remaining_accounts,
            system_program,
            payer,
            seeds,
        })
    }

    pub fn iter<'b>(&'b self) -> EventIterator<'a, 'b, 'info> {
        EventIterator {
            queue: self.queue.iter(),
            accounts: self.accounts.iter(),
            system_program: self.system_program.clone(),
            payer: self.payer.clone(),
            seeds: self.seeds.iter(),
        }
    }
}

pub struct EventIterator<'a, 'b, 'info: 'a + 'b> {
    queue: QueueIterator<'info>,
    accounts: Iter<'a, AccountInfo<'info>>,
    /// CHECK: anchor linter bug requires this
    system_program: AccountInfo<'info>,
    /// CHECK: anchor linter bug requires this
    payer: AccountInfo<'info>,
    seeds: Iter<'b, Vec<u8>>,
}

impl<'a, 'b, 'info: 'a + 'b> Iterator for EventIterator<'a, 'b, 'info> {
    type Item = Result<(EventAccounts<'info>, OrderbookEvent)>;

    fn next(&mut self) -> Option<Result<(EventAccounts<'info>, OrderbookEvent)>> {
        let event = self.queue.next()?;
        Some(self.extract_accounts(&event).map(|accts| (accts, event)))
    }
}

impl<'a, 'b, 'info: 'a + 'b> EventIterator<'a, 'b, 'info> {
    fn extract_accounts(&mut self, event: &OrderbookEvent) -> Result<EventAccounts<'info>> {
        match event {
            OrderbookEvent::Fill {
                event,
                maker_info,
                taker_info,
            } => {
                let mut maker = self.accounts.next_user(maker_info)?;
                let mut taker = self.accounts.next_user(taker_info)?;
                let (lender, borrower) = lender_borrower(event.taker_side, &mut maker, &mut taker);

                let mut auto_stake = None;
                if lender.callback.flags.contains(CallbackFlags::AUTO_STAKE) {
                    auto_stake = Some(
                        self.accounts.init_next::<SplitTicket>(
                            self.payer.to_account_info(),
                            self.system_program.to_account_info(),
                            auto_stake::seeds(auto_stake::Seeds {
                                lender: lender.account.key().as_ref(),
                                seeds_parameter: self
                                    .seeds
                                    .next()
                                    .ok_or(BondsError::InsufficientSeeds)?,
                            })
                            .as_slice(),
                        )?,
                    )
                }

                let mut lender_adapter_account = None;
                if lender.callback.adapter_account_key != Pubkey::default().to_bytes() {
                    lender_adapter_account = Some(self.accounts.next_adapter()?);
                }

                let mut new_debt = None;
                if borrower.callback.flags.contains(CallbackFlags::NEW_DEBT) {
                    new_debt = Some(
                        self.accounts.init_next::<Obligation>(
                            self.payer.to_account_info(),
                            self.system_program.to_account_info(),
                            new_debt::seeds(new_debt::Seeds {
                                borrower: borrower.account.key().as_ref(),
                                seeds_parameter: self
                                    .seeds
                                    .next()
                                    .ok_or(BondsError::InsufficientSeeds)?,
                            })
                            .as_slice(),
                        )?,
                    );
                }

                let mut borrower_adapter_account = None;
                if borrower.callback.adapter_account_key != Pubkey::default().to_bytes() {
                    borrower_adapter_account = Some(self.accounts.next_adapter()?);
                }

                Ok(EventAccounts::Fill(Box::new(FillAccounts {
                    maker,
                    taker,
                    auto_stake,
                    new_debt,
                    borrower_adapter_account,
                    lender_adapter_account,
                })))
            }
            OrderbookEvent::Out { info, .. } => {
                let user = self.accounts.next_user(info)?;
                let user_adapter_account =
                    match user.callback.adapter_account_key != Pubkey::default().to_bytes() {
                        true => Some(self.accounts.next_adapter()?),
                        false => None,
                    };
                Ok(EventAccounts::Out(Box::new(OutAccounts {
                    user,
                    user_adapter_account,
                })))
            }
        }
    }
}

pub fn lender_borrower<T>(taker_side: u8, maker: T, taker: T) -> (T, T) {
    match Side::from_u8(taker_side).unwrap() {
        Side::Bid => (taker, maker),
        Side::Ask => (maker, taker),
    }
}

pub trait UserAccounts<'a, 'info: 'a>: RemainingAccounts<'a, 'info> {
    fn next_user(&mut self, callback_info: &CallbackInfo) -> Result<UserData<'info>> {
        let account = self.next_anchor::<OrderbookUser, Mut>()?;

        if account.key() != callback_info.orderbook_account() {
            msg!(
                "Provided user account {} does not match event's callback info {}",
                account.key(),
                callback_info.orderbook_account()
            );
            return err!(BondsError::WrongOrderbookUser);
        }
        Ok(UserData {
            account,
            callback: *callback_info,
        })
    }
}
impl<'a, 'info: 'a, T: RemainingAccounts<'a, 'info>> UserAccounts<'a, 'info> for T {}
