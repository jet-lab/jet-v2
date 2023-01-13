use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    margin::state::{AutoRollConfig, MarginUser},
    orderbook::{
        instructions::lend_order::*,
        state::{
            margin_lend, CallbackFlags, InitTermDepositParams, LendAccounts, MarginLendAccounts,
            OrderParams,
        },
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

pub fn order_flags(user: &Account<MarginUser>, params: &OrderParams) -> Result<CallbackFlags> {
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
    let (callback_info, order_summary) = ctx.accounts.inner.orderbook_mut.place_order(
        ctx.accounts.inner.authority.key(),
        Side::Bid,
        params,
        ctx.accounts.margin_user.key(),
        ctx.accounts.margin_user.key(),
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
        order_flags(ctx.accounts.margin_user.as_ref(), &params)?,
    )?;

    let accounts = &mut MarginLendAccounts {
        margin_user: ctx.accounts.margin_user.clone(),
        ticket_collateral: &ctx.accounts.ticket_collateral,
        ticket_collateral_mint: &ctx.accounts.ticket_collateral_mint,
        inner: &LendAccounts {
            authority: &ctx.accounts.inner.authority.to_account_info(),
            market: &ctx.accounts.inner.orderbook_mut.market,
            ticket_mint: &ctx.accounts.inner.ticket_mint,
            ticket_settlement: &ctx.accounts.inner.ticket_settlement,
            lender_tokens: &ctx.accounts.inner.lender_tokens,
            payer: &ctx.accounts.inner.payer,
            underlying_token_vault: &ctx.accounts.inner.underlying_token_vault,
            token_program: &ctx.accounts.inner.token_program,
            system_program: &ctx.accounts.inner.system_program,
        },
    };
    let deposit_params = if callback_info.flags.contains(CallbackFlags::AUTO_STAKE) {
        Some(InitTermDepositParams {
            market: ctx.accounts.inner.orderbook_mut.market.key(),
            owner: ctx.accounts.margin_user.key(),
            tenor: ctx.accounts.inner.orderbook_mut.market.load()?.lend_tenor,
            sequence_number: ctx.accounts.margin_user.assets.next_new_deposit_seqno(),
            auto_roll: callback_info.flags.contains(CallbackFlags::AUTO_ROLL),
            seed: ctx
                .accounts
                .margin_user
                .assets
                .next_new_deposit_seqno()
                .to_le_bytes()
                .to_vec(),
        })
    } else {
        None
    };

    margin_lend(
        accounts,
        deposit_params,
        &callback_info,
        &order_summary,
        true,
    )?;

    emit!(crate::events::OrderPlaced {
        market: ctx.accounts.inner.orderbook_mut.market.key(),
        authority: ctx.accounts.inner.authority.key(),
        margin_user: Some(ctx.accounts.margin_user.key()),
        order_tag: callback_info.order_tag.as_u128(),
        order_summary: order_summary.summary(),
        auto_stake: params.auto_stake,
        post_only: params.post_only,
        post_allowed: params.post_allowed,
        limit_price: params.limit_price,
        order_type: crate::events::OrderType::MarginLend,
    });
    ctx.accounts.margin_user.emit_asset_balances();
    Ok(())
}
