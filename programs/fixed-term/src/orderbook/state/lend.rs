use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::{accessor, mint_to, Mint, MintTo, Token, TokenAccount};
use jet_program_common::traits::SafeAdd;

use crate::{
    margin::state::{AutoRollConfig, MarginUser},
    tickets::state::{InitTermDepositAccounts, TermDepositWriter},
    FixedTermErrorCode,
};

use super::{
    CallbackFlags, CallbackInfo, MarginCallbackInfo, OrderParams, OrderbookMut, RoundingAction,
    SensibleOrderSummary,
};

pub struct LendOrderAccounts<'a, 'info> {
    /// Authority accounted for as the owner of resulting orderbook bids and `TermDeposit` accounts
    pub authority: &'a AccountInfo<'info>,

    pub orderbook_mut: &'a mut OrderbookMut<'info>,

    /// where to settle tickets on match:
    /// - TermDeposit that will be created if the order is filled as a taker and `auto_stake` is enabled
    /// - ticket token account to receive tickets
    /// be careful to check this properly. one way is by using lender_tickets_token_account
    pub(crate) ticket_settlement: &'a AccountInfo<'info>,

    /// where to loan tokens from
    pub lender_tokens: &'a AccountInfo<'info>,

    /// The market token vault
    pub underlying_token_vault: &'a Account<'info, TokenAccount>,

    /// The market token vault
    pub ticket_mint: &'a Account<'info, Mint>,

    pub payer: &'a Signer<'info>,
    pub system_program: &'a Program<'info, System>,
    pub token_program: &'a Program<'info, Token>,
}

impl<'a, 'info> LendOrderAccounts<'a, 'info> {
    pub fn lend_order(
        &mut self,
        params: OrderParams,
        adapter: Option<Pubkey>,
        seed: Vec<u8>,
    ) -> Result<()> {
        let (info, summary) = self.orderbook_mut.place_signer_order(
            Side::Bid,
            params,
            self.authority.key(),
            if params.auto_stake {
                self.authority.key()
            } else {
                self.lender_tickets_token_account()?
            },
            self.lender_tokens.key(),
            adapter,
            if params.auto_stake {
                CallbackFlags::AUTO_STAKE
            } else {
                CallbackFlags::empty()
            },
        )?;
        self.lend(
            &summary,
            self.term_deposit(
                &CallbackInfo::from_signer_info(info.clone()),
                &summary,
                seed,
            )?,
            true,
        )?;

        emit!(crate::events::OrderPlaced {
            market: self.orderbook_mut.market.key(),
            authority: self.authority.key(),
            margin_user: None,
            order_tag: info.order_tag.as_u128(),
            order_summary: summary.summary(),
            order_type: crate::events::OrderType::Lend,
            limit_price: params.limit_price,
            auto_stake: params.auto_stake,
            post_only: params.post_only,
            post_allowed: params.post_allowed,
        });

        Ok(())
    }

    pub fn lend(
        &self,
        summary: &SensibleOrderSummary,
        deposit: Option<TermDepositWriter>,
        requires_payment: bool,
    ) -> Result<u64> {
        let staked = self.issue(summary, deposit)?;

        if requires_payment {
            // take all underlying that has been lent plus what may be lent later
            anchor_spl::token::transfer(
                anchor_lang::prelude::CpiContext::new(
                    self.token_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: self.lender_tokens.to_account_info(),
                        to: self.underlying_token_vault.to_account_info(),
                        authority: self.authority.to_account_info(),
                    },
                ),
                summary.quote_combined()?,
            )?;
        }

        Ok(staked)
    }

    fn issue(
        &self,
        summary: &SensibleOrderSummary,
        deposit: Option<TermDepositWriter>,
    ) -> Result<u64> {
        let staked = if let Some(writer) = deposit {
            if summary.base_filled() > 0 {
                writer.init_and_write(InitTermDepositAccounts {
                    deposit: self.ticket_settlement,
                    payer: self.payer,
                    system_program: self.system_program,
                })?;
            }
            summary.base_filled()
        } else {
            self.issue_tickets(summary.base_filled())?;
            0
        };

        Ok(staked)
    }

    fn issue_tickets(&self, amount: u64) -> Result<()> {
        mint_to(
            CpiContext::new(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.ticket_mint.to_account_info(),
                    to: self.ticket_settlement.clone(),
                    authority: self.orderbook_mut.market.to_account_info(),
                },
            )
            .with_signer(&[&self.orderbook_mut.market.load()?.authority_seeds()]),
            amount,
        )?;

        Ok(())
    }

    fn lender_tickets_token_account(&self) -> Result<Pubkey> {
        Account::<'info, TokenAccount>::try_from(self.ticket_settlement)?;
        require!(
            accessor::mint(&self.ticket_settlement.to_account_info())?
                == self.orderbook_mut.ticket_mint(),
            FixedTermErrorCode::WrongTicketMint
        );

        Ok(self.ticket_settlement.key())
    }

    fn term_deposit(
        &self,
        info: &CallbackInfo,
        summary: &SensibleOrderSummary,
        seed: Vec<u8>,
    ) -> Result<Option<TermDepositWriter>> {
        if info.flags.contains(CallbackFlags::AUTO_STAKE) {
            return Ok(Some(TermDepositWriter {
                market: self.orderbook_mut.market.key(),
                owner: self.authority.key(),
                payer: self.payer.key(),
                order_tag: info.order_tag.as_u128(),
                amount: summary.base_filled(),
                principal: summary.quote_filled(RoundingAction::FillLend)?,
                tenor: self.orderbook_mut.market.load()?.lend_tenor,
                sequence_number: 0,
                flags: info.flags.into(),
                seed,
            }));
        }
        Ok(None)
    }
}

pub struct MarginLendAccounts<'a, 'info> {
    pub margin_user: &'a mut Account<'info, MarginUser>,
    pub ticket_collateral: &'a AccountInfo<'info>,
    pub ticket_collateral_mint: &'a AccountInfo<'info>,
    pub inner: &'a mut LendOrderAccounts<'a, 'info>,
}

impl<'a, 'info> MarginLendAccounts<'a, 'info> {
    pub fn margin_lend_order(
        &mut self,
        params: &OrderParams,
        adapter: Option<Pubkey>,
        requires_payment: bool,
    ) -> Result<()> {
        let (info, summary) = self.inner.orderbook_mut.place_margin_order(
            Side::Bid,
            *params,
            self.inner.authority.key(),
            self.margin_user.key(),
            adapter,
            self.order_flags(params)?,
        )?;

        let deposit = self.maybe_term_deposit(&info, &summary)?;
        self.margin_lend(&summary, deposit, requires_payment)?;
        self.emit_margin_lend_order(params, &info, &summary)
    }

    fn margin_lend(
        &mut self,
        summary: &SensibleOrderSummary,
        deposit: Option<TermDepositWriter>,
        requires_payment: bool,
    ) -> Result<()> {
        let tickets_staked = self.inner.lend(summary, deposit, requires_payment)?;
        let tickets_posted = summary.base_posted();
        self.margin_user
            .lend_order(tickets_staked, tickets_posted)?;

        // collateralize the ticket value of the order
        mint_to(
            CpiContext::new(
                self.inner.token_program.to_account_info(),
                MintTo {
                    mint: self.ticket_collateral_mint.to_account_info(),
                    to: self.ticket_collateral.to_account_info(),
                    authority: self.inner.orderbook_mut.market.to_account_info(),
                },
            )
            .with_signer(&[&self.inner.orderbook_mut.market.load()?.authority_seeds()]),
            tickets_posted.safe_add(tickets_staked)?,
        )
    }

    fn maybe_term_deposit(
        &self,
        info: &MarginCallbackInfo,
        summary: &SensibleOrderSummary,
    ) -> Result<Option<TermDepositWriter>> {
        if info.flags.contains(CallbackFlags::AUTO_STAKE) {
            return Ok(Some(TermDepositWriter {
                market: self.inner.orderbook_mut.market.key(),
                owner: self.inner.authority.key(),
                payer: self.inner.payer.key(),
                order_tag: info.order_tag.as_u128(),
                tenor: self.inner.orderbook_mut.market.load()?.lend_tenor,
                sequence_number: self.margin_user.next_term_deposit(),
                amount: summary.base_filled(),
                principal: summary.quote_filled(RoundingAction::FillLend)?,
                flags: info.flags.into(),
                seed: self.margin_user.next_term_deposit().to_le_bytes().to_vec(),
            }));
        }
        Ok(None)
    }

    fn order_flags(&self, params: &OrderParams) -> Result<CallbackFlags> {
        let auto_roll = if params.auto_roll {
            if self.margin_user.lend_roll_config == AutoRollConfig::default() {
                msg!(
                    "Auto roll settings have not been configured for margin user [{}]",
                    self.margin_user.key()
                );
                return err!(FixedTermErrorCode::InvalidAutoRollConfig);
            }
            CallbackFlags::AUTO_ROLL
        } else {
            CallbackFlags::default()
        };
        let auto_stake = if params.auto_stake {
            CallbackFlags::AUTO_STAKE
        } else {
            CallbackFlags::empty()
        };

        Ok(CallbackFlags::MARGIN | auto_roll | auto_stake)
    }

    fn emit_margin_lend_order(
        &self,
        params: &OrderParams,
        info: &MarginCallbackInfo,
        summary: &SensibleOrderSummary,
    ) -> Result<()> {
        emit!(crate::events::OrderPlaced {
            market: self.inner.orderbook_mut.market.key(),
            authority: self.inner.authority.key(),
            margin_user: Some(self.margin_user.key()),
            order_tag: info.order_tag.as_u128(),
            order_summary: summary.summary(),
            auto_stake: params.auto_stake,
            post_only: params.post_only,
            post_allowed: params.post_allowed,
            limit_price: params.limit_price,
            order_type: crate::events::OrderType::MarginLend,
        });
        self.margin_user.emit_asset_balances()
    }
}
