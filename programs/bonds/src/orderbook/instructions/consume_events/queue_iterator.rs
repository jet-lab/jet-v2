use std::slice::Iter;

use agnostic_orderbook::state::{event_queue::FillEvent, Side};
use anchor_lang::prelude::*;
use num_traits::FromPrimitive;

use crate::{
    margin::state::{MarginUser, Obligation},
    orderbook::state::{CallbackFlags, CallbackInfo, EventQueue, OrderbookEvent, QueueIterator},
    serialization::RemainingAccounts,
    tickets::state::SplitTicket,
    BondsError,
};

use super::{ConsumeEvents, EventAccounts, FillAccounts, OutAccounts, UserData};

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
    type Item = Result<(EventAccounts<'a, 'info>, OrderbookEvent)>;

    fn next(&mut self) -> Option<Result<(EventAccounts<'a, 'info>, OrderbookEvent)>> {
        let event = self.queue.next()?;
        Some(self.extract_accounts(&event).map(|accts| (accts, event)))
    }
}

impl<'a, 'b, 'info: 'a + 'b> EventIterator<'a, 'b, 'info> {
    fn extract_accounts(&mut self, event: &OrderbookEvent) -> Result<EventAccounts<'a, 'info>> {
        match event {
            OrderbookEvent::Fill {
                event,
                maker_info,
                taker_info,
            } => self.extract_fill_accounts(event, maker_info, taker_info),
            OrderbookEvent::Out { info, .. } => {
                let user = self.accounts.next_user(info)?;
                let user_adapter_account =
                    if info.adapter_account_key != Pubkey::default().to_bytes() {
                        Some(self.accounts.next_adapter()?)
                    } else {
                        None
                    };
                Ok(EventAccounts::Out(Box::new(OutAccounts {
                    user,
                    user_adapter_account,
                })))
            }
        }
    }

    fn extract_fill_accounts(
        &mut self,
        event: &FillEvent,
        maker_info: &CallbackInfo,
        taker_info: &CallbackInfo,
    ) -> Result<EventAccounts<'a, 'info>> {
        let (lender_info, borrower_info) =
            lender_borrower(event.taker_side, maker_info, taker_info);
        let lender = self.accounts.next_user(lender_info)?;
        let borrower = self.accounts.next_user(borrower_info)?;

        let auto_stake = if lender_info.flags.contains(CallbackFlags::AUTO_STAKE) {
            Some(self.accounts.init_next::<SplitTicket>(
                self.payer.to_account_info(),
                self.system_program.to_account_info(),
                &[
                    crate::seeds::SPLIT_TICKET,
                    &lender_info.account_key,
                    self.seeds.next().ok_or(BondsError::InsufficientSeeds)?,
                ],
            )?)
        } else {
            None
        };
        let lender_adapter_account =
            if lender_info.adapter_account_key != Pubkey::default().to_bytes() {
                Some(self.accounts.next_adapter()?)
            } else {
                None
            };

        let new_debt = if borrower_info.flags.contains(CallbackFlags::NEW_DEBT) {
            Some(self.accounts.init_next::<Obligation>(
                self.payer.to_account_info(),
                self.system_program.to_account_info(),
                &[
                    crate::seeds::OBLIGATION,
                    &borrower_info.account_key,
                    self.seeds.next().ok_or(BondsError::InsufficientSeeds)?,
                ],
            )?)
        } else {
            None
        };
        let borrower_adapter_account =
            if borrower_info.adapter_account_key != Pubkey::default().to_bytes() {
                Some(self.accounts.next_adapter()?)
            } else {
                None
            };
        let (maker, taker) = match Side::from_u8(event.taker_side).unwrap() {
            Side::Ask => (lender, borrower),
            Side::Bid => (borrower, lender),
        };
        Ok(EventAccounts::Fill(Box::new(FillAccounts {
            maker,
            taker,
            auto_stake,
            new_debt,
            borrower_adapter_account,
            lender_adapter_account,
        })))
    }
}

pub fn lender_borrower<T>(taker_side: u8, maker: T, taker: T) -> (T, T) {
    match Side::from_u8(taker_side).unwrap() {
        Side::Bid => (taker, maker),
        Side::Ask => (maker, taker),
    }
}

pub trait UserAccounts<'a, 'info: 'a>: RemainingAccounts<'a, 'info> {
    fn next_user(&mut self, callback_info: &CallbackInfo) -> Result<UserData<'a, 'info>> {
        let vault = self.next_account()?;
        if callback_info.user_vault != vault.key().to_bytes() {
            msg!(
                "Provided vault {} does not match the callback info {}",
                vault.key(),
                Pubkey::new_from_array(callback_info.user_vault)
            );
            return err!(BondsError::WrongVault);
        }
        let borrower_account = if callback_info.flags.contains(CallbackFlags::NEW_DEBT) {
            let account = Account::<MarginUser>::try_from(self.next_account()?)?;
            if callback_info.account_key != account.key().to_bytes() {
                msg!(
                    "Provided borrower {} does not match the callback info {}",
                    account.key(),
                    Pubkey::new_from_array(callback_info.account_key)
                );
                return err!(BondsError::WrongMarginUser);
            }
            Some(account)
        } else {
            None
        };
        let key = Pubkey::new_from_array(callback_info.account_key);
        Ok(UserData {
            vault,
            key,
            borrower_account,
            info: *callback_info,
        })
    }
}
impl<'a, 'info: 'a, T: RemainingAccounts<'a, 'info>> UserAccounts<'a, 'info> for T {}
