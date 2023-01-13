use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::{accessor::mint, Mint, Token, TokenAccount};
use jet_program_proc_macros::MarketTokenManager;

use crate::{orderbook::state::*, serialization::RemainingAccounts, FixedTermErrorCode};

#[derive(Accounts, MarketTokenManager)]
pub struct LendOrder<'info> {
    /// Authority accounted for as the owner of resulting orderbook bids and `TermDeposit` accounts
    pub authority: Signer<'info>,

    #[market]
    pub orderbook_mut: OrderbookMut<'info>,

    /// where to settle tickets on match:
    /// - TermDeposit that will be created if the order is filled as a taker and `auto_stake` is enabled
    /// - ticket token account to receive tickets
    /// be careful to check this properly. one way is by using lender_tickets_token_account
    #[account(mut)]
    pub(crate) ticket_settlement: AccountInfo<'info>,

    /// where to loan tokens from
    #[account(mut, constraint = mint(&lender_tokens.to_account_info())? == orderbook_mut.underlying_mint() @ FixedTermErrorCode::WrongUnderlyingTokenMint)]
    pub lender_tokens: Account<'info, TokenAccount>,

    /// The market token vault
    #[account(mut, address = orderbook_mut.vault() @ FixedTermErrorCode::WrongVault)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

    /// The market token vault
    #[account(mut, address = orderbook_mut.ticket_mint() @ FixedTermErrorCode::WrongTicketMint)]
    pub ticket_mint: Account<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> LendOrder<'info> {
    pub fn lender_tickets_token_account(&self) -> Result<Pubkey> {
        Account::<'info, TokenAccount>::try_from(&self.ticket_settlement)?;
        require!(
            mint(&self.ticket_settlement.to_account_info())? == self.orderbook_mut.ticket_mint(),
            FixedTermErrorCode::WrongTicketMint
        );

        Ok(self.ticket_settlement.key())
    }
}

pub fn handler(ctx: Context<LendOrder>, params: OrderParams, seed: Vec<u8>) -> Result<()> {
    let (callback_info, order_summary) = ctx.accounts.orderbook_mut.place_order(
        ctx.accounts.authority.key(),
        Side::Bid,
        params,
        if params.auto_stake {
            ctx.accounts.authority.key()
        } else {
            ctx.accounts.lender_tickets_token_account()?
        },
        ctx.accounts.lender_tokens.key(),
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
        if params.auto_stake {
            CallbackFlags::AUTO_STAKE
        } else {
            CallbackFlags::empty()
        },
    )?;
    let lend_accounts = LendAccounts {
        authority: &ctx.accounts.authority.to_account_info(),
        market: &ctx.accounts.orderbook_mut.market,
        ticket_mint: &ctx.accounts.ticket_mint,
        ticket_settlement: &ctx.accounts.ticket_settlement,
        lender_tokens: &ctx.accounts.lender_tokens,
        underlying_token_vault: &ctx.accounts.underlying_token_vault,
        payer: &ctx.accounts.payer,
        token_program: &ctx.accounts.token_program,
        system_program: &ctx.accounts.system_program,
    };
    let deposit_params = if callback_info.flags.contains(CallbackFlags::AUTO_STAKE) {
        Some(InitTermDepositParams {
            market: ctx.accounts.orderbook_mut.market.key(),
            owner: ctx.accounts.authority.key(),
            tenor: ctx.accounts.orderbook_mut.market.load()?.lend_tenor,
            sequence_number: 0,
            auto_roll: callback_info.flags.contains(CallbackFlags::AUTO_ROLL),
            seed,
        })
    } else {
        None
    };

    lend(
        &lend_accounts,
        deposit_params,
        &callback_info,
        &order_summary,
        true,
    )?;

    emit!(crate::events::OrderPlaced {
        market: ctx.accounts.orderbook_mut.market.key(),
        authority: ctx.accounts.authority.key(),
        margin_user: None,
        order_tag: callback_info.order_tag.as_u128(),
        order_summary: order_summary.summary(),
        order_type: crate::events::OrderType::Lend,
        limit_price: params.limit_price,
        auto_stake: params.auto_stake,
        post_only: params.post_only,
        post_allowed: params.post_allowed,
    });

    Ok(())
}
