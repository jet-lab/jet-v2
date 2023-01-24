use std::slice::Iter;

use anchor_lang::prelude::*;

use crate::{
    events::skip_err,
    margin::state::{MarginUser, TermLoan},
    orderbook::state::{
        CallbackFlags, EventQueue, FillInfo, MaybePushAdapterEvent, OrderbookEvent, OutInfo,
        QueueIterator, UserCallbackInfo,
    },
    serialization::{AnchorAccount, Mut, RemainingAccounts},
    tickets::state::TermDeposit,
    FixedTermErrorCode,
};

use super::{
    ConsumeEvents, FillAccount, FillAccounts, MarginFillAccounts, OutAccounts, TermAccount,
    UserAccount,
};

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
            OrderbookEvent::Fill(fill) => {
                PreparedEvent::Fill(self.extract_fill_accounts(&fill)?, fill)
            }
            OrderbookEvent::Out(out) => PreparedEvent::Out(self.extract_out_accounts(&out)?, out),
        })
    }

    pub fn extract_fill_accounts(&mut self, fill: &FillInfo) -> Result<FillAccounts<'info>> {
        self.try_update_fill_adapters(fill)?;

        let accounts = match &fill.maker_info {
            UserCallbackInfo::Margin(info) => FillAccounts::Margin({
                let margin_user = self.accounts.next_margin_user(&info.margin_user)?;
                let term_account = if info.flags.contains(CallbackFlags::AUTO_STAKE) {
                    let seed = margin_user.assets.next_new_deposit_seqno().to_le_bytes();
                    Some(TermAccount::Deposit(
                        self.accounts.init_next::<TermDeposit>(
                            self.payer.to_account_info(),
                            self.system_program.to_account_info(),
                            &TermDeposit::seeds(
                                self.market.as_ref(),
                                info.margin_account.as_ref(),
                                &seed,
                            ),
                        )?,
                    ))
                } else if info.flags.contains(CallbackFlags::NEW_DEBT) {
                    let seed = margin_user.debt.next_new_loan_seqno().to_le_bytes();
                    Some(TermAccount::Loan(self.accounts.init_next::<TermLoan>(
                        self.payer.to_account_info(),
                        self.system_program.to_account_info(),
                        &[
                            crate::seeds::TERM_LOAN,
                            self.market.as_ref(),
                            &info.margin_account.as_ref(),
                            &seed,
                        ],
                    )?))
                } else {
                    None
                };
                MarginFillAccounts {
                    margin_user,
                    term_account,
                }
            }),
            UserCallbackInfo::Signer(info) => {
                FillAccounts::Signer(if info.flags.contains(CallbackFlags::AUTO_STAKE) {
                    let mut seed = [0u8; 8];
                    self.next_seed(&mut seed);
                    FillAccount::TermDeposit(self.accounts.init_next::<TermDeposit>(
                        self.payer.to_account_info(),
                        self.system_program.to_account_info(),
                        &TermDeposit::seeds(self.market.as_ref(), info.signer.as_ref(), &seed),
                    )?)
                } else {
                    FillAccount::Token(
                        self.accounts
                            .next_token_account(&info.ticket_or_deposit_account)?,
                    )
                })
            }
        };
        Ok(accounts)
    }

    pub fn extract_out_accounts(&mut self, out: &OutInfo) -> Result<OutAccounts<'info>> {
        self.try_update_out_adapter(out)?;
        let accounts = match &out.info {
            UserCallbackInfo::Margin(info) => {
                OutAccounts::Margin(self.accounts.next_margin_user(&info.margin_user)?)
            }
            UserCallbackInfo::Signer(info) => {
                OutAccounts::Signer(self.accounts.next_token_account(&info.signer)?)
            }
        };

        Ok(accounts)
    }
    fn next_seed(&mut self, seed: &mut [u8; 8]) {
        seed[..self.seed.len()].copy_from_slice(&self.seed);
        self.seed[0] = self.seed[0].wrapping_add(1);
    }

    pub fn try_update_fill_adapters(&mut self, fill: &FillInfo) -> Result<()> {
        self.accounts
            .maybe_adapter_if_needed(fill.maker_info.adapter())?
            .maybe_push_event(
                fill.event,
                Some(&(&fill.maker_info).into()),
                Some(&(&fill.taker_info).into()),
            );

        self.accounts
            .maybe_adapter_if_needed(fill.taker_info.adapter())?
            .maybe_push_event(
                fill.event,
                Some(&(&fill.maker_info).into()),
                Some(&(&fill.taker_info).into()),
            );

        Ok(())
    }

    pub fn try_update_out_adapter(&mut self, out: &OutInfo) -> Result<()> {
        self.accounts
            .maybe_adapter_if_needed(out.info.adapter())?
            .maybe_push_event(out.event, Some(&(&out.info).into()), None);

        Ok(())
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

    fn next_token_account(&mut self, key: &Pubkey) -> Result<AccountInfo<'info>> {
        self.next_user_account(*key).map(|a| a.as_token_account())
    }

    fn next_margin_user(&mut self, key: &Pubkey) -> Result<AnchorAccount<'info, MarginUser, Mut>> {
        self.next_user_account(*key).map(|a| a.margin_user())?
    }

    fn maybe_adapter_if_needed(
        &mut self,
        adapter_key: &Pubkey,
    ) -> Result<Option<EventQueue<'info>>> {
        if adapter_key != &Pubkey::default() {
            match self.next_adapter() {
                Ok(adapter) => {
                    // this needs to fail the ix because it means the crank passed the wrong account
                    require_eq!(
                        adapter_key,
                        &adapter.key(),
                        FixedTermErrorCode::WrongAdapter
                    );
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
