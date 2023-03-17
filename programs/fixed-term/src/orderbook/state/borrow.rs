use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, transfer, MintTo, Transfer};
use jet_margin::{AdapterResult, PositionChange};
use jet_program_common::traits::SafeSub;

use crate::{
    events::{OrderPlaced, OrderType},
    margin::state::{return_to_margin, BorrowAutoRollConfig, MarginUser, TermLoanBuilder},
    FixedTermErrorCode,
};

use super::{
    CallbackFlags, MarginCallbackInfo, OrderParams, OrderbookMut, RoundingAction,
    SensibleOrderSummary,
};

pub struct MarginBorrowOrderAccounts<'a, 'info> {
    /// The account tracking borrower debts
    pub margin_user: &'a mut Account<'info, MarginUser>,

    /// TermLoan account minted upon match
    /// CHECK: in instruction logic
    pub term_loan: &'a AccountInfo<'info>,

    /// The margin account for this borrow order
    pub margin_account: &'a AccountInfo<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    /// CHECK: margin_user
    pub claims: &'a AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    /// CHECK: in instruction handler
    pub claims_mint: &'a AccountInfo<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    pub underlying_collateral: &'a AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    pub underlying_collateral_mint: &'a AccountInfo<'info>,

    /// The market token vault
    pub underlying_token_vault: &'a AccountInfo<'info>,

    /// The market fee vault
    pub fee_vault: &'a AccountInfo<'info>,

    /// Where to receive borrowed tokens
    pub underlying_settlement: &'a AccountInfo<'info>,

    pub orderbook_mut: &'a mut OrderbookMut<'info>,

    /// payer for `TermLoan` initialization
    pub payer: &'a AccountInfo<'info>,

    /// Solana system program
    pub system_program: &'a AccountInfo<'info>,

    pub token_program: &'a AccountInfo<'info>,
    // Optional event adapter account
    pub event_adapter: Option<Pubkey>,
}

impl<'a, 'info> MarginBorrowOrderAccounts<'a, 'info> {
    pub fn borrow_order(&mut self, mut params: OrderParams) -> Result<()> {
        self.orderbook_mut
            .market
            .load()?
            .add_origination_fee(&mut params);

        let (callback_info, order_summary) = self.orderbook_mut.place_margin_order(
            Side::Ask,
            params,
            self.margin_account.key(),
            self.margin_user.key(),
            self.event_adapter,
            self.callback_flags(&params)?,
        )?;

        self.handle_posted(&order_summary)?;
        if order_summary.base_filled() > 0 {
            self.handle_filled(&order_summary, &callback_info)?;
        }

        // place a claim for the borrowed tokens
        mint_to(
            CpiContext::new(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.claims_mint.to_account_info(),
                    to: self.claims.to_account_info(),
                    authority: self.orderbook_mut.market.to_account_info(),
                },
            )
            .with_signer(&[&self.orderbook_mut.market.load()?.authority_seeds()]),
            order_summary.base_combined(),
        )?;

        emit!(OrderPlaced {
            market: self.orderbook_mut.market.key(),
            authority: self.margin_account.key(),
            margin_user: Some(self.margin_user.key()),
            order_tag: callback_info.order_tag.as_u128(),
            order_summary: order_summary.summary(),
            limit_price: params.limit_price,
            auto_stake: params.auto_stake,
            post_only: params.post_only,
            post_allowed: params.post_allowed,
            order_type: OrderType::MarginBorrow,
        });
        self.margin_user.emit_debt_balances();

        // this is just used to make sure the position is still registered.
        // it's actually registered by initialize_margin_user
        return_to_margin(
            self.margin_account,
            &AdapterResult {
                position_changes: vec![(
                    self.claims_mint.key(),
                    vec![PositionChange::Register(self.claims.key())],
                )],
            },
        )
    }

    fn handle_posted(&mut self, summary: &SensibleOrderSummary) -> Result<()> {
        let posted_token_value = summary.quote_posted(RoundingAction::PostBorrow)?;
        let posted_ticket_value = summary.base_posted();

        self.margin_user
            .post_borrow_order(posted_token_value, posted_ticket_value)?;

        // collateralize the tokens involved in the order
        mint_to(
            CpiContext::new(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.underlying_collateral_mint.to_account_info(),
                    to: self.underlying_collateral.to_account_info(),
                    authority: self.orderbook_mut.market.to_account_info(),
                },
            )
            .with_signer(&[&self.orderbook_mut.market.load()?.authority_seeds()]),
            posted_token_value,
        )?;

        Ok(())
    }

    fn handle_filled(
        &mut self,
        summary: &SensibleOrderSummary,
        info: &MarginCallbackInfo,
    ) -> Result<u64> {
        let filled_ticket_value = summary.base_filled();
        let filled_token_value = summary.quote_filled(RoundingAction::FillBorrow)?;
        let current_time = Clock::get()?.unix_timestamp;
        let maturation_timestamp =
            self.orderbook_mut.market.load()?.borrow_tenor as i64 + current_time;

        let sequence_number = self
            .margin_user
            .taker_fill_borrow_order(filled_ticket_value, maturation_timestamp)?;

        let disburse = self
            .orderbook_mut
            .market
            .load()?
            .loan_to_disburse(filled_token_value);
        let fees = filled_token_value.safe_sub(disburse)?;

        // write a TermLoan account
        let builder = TermLoanBuilder::new_from_order(
            self,
            summary,
            info,
            current_time,
            maturation_timestamp,
            fees,
            sequence_number,
        )?;
        builder.init_and_write(self.term_loan, self.payer, self.system_program)?;

        // Allot the borrower the tokens from the filled order
        transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.underlying_token_vault.to_account_info(),
                    to: self.underlying_settlement.to_account_info(),
                    authority: self.orderbook_mut.market.to_account_info(),
                },
            )
            .with_signer(&[&self.orderbook_mut.market.load()?.authority_seeds()]),
            disburse,
        )?;

        // Collect fees from the order fill
        transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.underlying_token_vault.to_account_info(),
                    to: self.fee_vault.to_account_info(),
                    authority: self.orderbook_mut.market.to_account_info(),
                },
            )
            .with_signer(&[&self.orderbook_mut.market.load()?.authority_seeds()]),
            fees,
        )?;

        Ok(disburse)
    }

    fn callback_flags(&self, params: &OrderParams) -> Result<CallbackFlags> {
        let auto_roll = if params.auto_roll {
            if self.margin_user.borrow_roll_config == BorrowAutoRollConfig::default() {
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

        let flags = CallbackFlags::NEW_DEBT | CallbackFlags::MARGIN | auto_roll;
        Ok(flags)
    }
}
