use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, MintTo};
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    margin::state::{AutoRollConfig, MarginUser},
    orderbook::{
        instructions::lend_order::*,
        state::{CallbackFlags, OrderParams},
    },
    serialization::RemainingAccounts,
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct MarginLendOrder<'info> {
    /// The account tracking borrower debts
    #[account(
        mut,
        constraint = margin_user.margin_account.key() == inner.authority.key(),
        has_one = ticket_collateral @ FixedTermErrorCode::WrongTicketCollateralAccount,
    )]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub ticket_collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub ticket_collateral_mint: AccountInfo<'info>,

    #[market(orderbook_mut)]
    #[token_program]
    pub inner: LendOrder<'info>,
    // Optional event adapter account
    // pub event_adapter: AccountInfo<'info>,
}

impl<'info> MarginLendOrder<'info> {
    #[inline(never)]
    pub fn lend_order(&mut self, params: OrderParams, adapter: Option<Pubkey>) -> Result<()> {
        let user = &mut self.margin_user;

        let (callback_info, order_summary) = self.inner.orderbook_mut.place_order(
            self.inner.authority.key(),
            Side::Bid,
            params,
            user.key(),
            user.key(),
            adapter,
            order_flags(user, &params)?,
        )?;
        let staked = self.inner.lend(
            user.key(),
            &user.assets.next_new_deposit_seqno().to_le_bytes(),
            user.assets.next_new_deposit_seqno(),
            callback_info,
            &order_summary,
        )?;
        if staked > 0 {
            self.margin_user.assets.new_deposit(staked)?;
        }
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
            staked + order_summary.quote_posted()?,
        )?;
        emit!(crate::events::OrderPlaced {
            market: self.inner.orderbook_mut.market.key(),
            authority: self.inner.authority.key(),
            margin_user: Some(self.margin_user.key()),
            order_tag: callback_info.order_tag.as_u128(),
            order_summary: order_summary.summary(),
            auto_stake: params.auto_stake,
            post_only: params.post_only,
            post_allowed: params.post_allowed,
            limit_price: params.limit_price,
            order_type: crate::events::OrderType::MarginLend,
        });
        self.margin_user.emit_asset_balances();
        Ok(())
    }
}

fn order_flags(user: &Account<MarginUser>, params: &OrderParams) -> Result<CallbackFlags> {
    let auto_roll = if params.auto_roll {
        if user.lend_roll_config == AutoRollConfig::default() {
            msg!(
                "Auto roll settings have not been configured for margin user [{}]",
                user.key()
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

pub fn handler(ctx: Context<MarginLendOrder>, params: OrderParams) -> Result<()> {
    ctx.accounts.lend_order(
        params,
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
    )
}
