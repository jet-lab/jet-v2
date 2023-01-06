use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::{accessor::mint, Mint, Token, TokenAccount};
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    events::TermDepositCreated,
    market_token_manager::MarketTokenManager,
    orderbook::state::*,
    serialization::{self, RemainingAccounts},
    tickets::state::TermDeposit,
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct LendOrder<'info> {
    /// Signing authority over the token vault transferring for a lend order
    /// Check for signature occurs in handler logic
    pub authority: AccountInfo<'info>,

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

    /// returns the amount of tickets staked
    pub fn lend(
        &self,
        user: Pubkey,
        seed: &[u8],
        sequence_number: u64,
        callback_info: CallbackInfo,
        order_summary: &SensibleOrderSummary,
    ) -> Result<u64> {
        let market = self.orderbook_mut.market.key();
        let tenor = self.orderbook_mut.market.load()?.lend_tenor;

        let staked = if order_summary.base_filled() > 0 {
            if callback_info.flags.contains(CallbackFlags::AUTO_STAKE) {
                // auto_stake: issue split tickets to the user for immediate fill
                let mut deposit = serialization::init::<TermDeposit>(
                    self.ticket_settlement.to_account_info(),
                    self.payer.to_account_info(),
                    self.system_program.to_account_info(),
                    &[
                        crate::seeds::TERM_DEPOSIT,
                        market.as_ref(),
                        user.as_ref(),
                        seed,
                    ],
                )?;
                let timestamp = Clock::get()?.unix_timestamp;
                let maturation_timestamp = timestamp + tenor as i64;

                *deposit = TermDeposit {
                    market,
                    sequence_number,
                    owner: user,
                    payer: self.payer.key(),
                    matures_at: maturation_timestamp,
                    principal: order_summary.quote_filled()?,
                    amount: order_summary.base_filled(),
                };
                emit!(TermDepositCreated {
                    term_deposit: deposit.key(),
                    authority: user,
                    payer: self.payer.key(),
                    order_tag: Some(callback_info.order_tag.as_u128()),
                    sequence_number,
                    market,
                    maturation_timestamp,
                    principal: deposit.principal,
                    amount: deposit.amount,
                });
                order_summary.base_filled()
            } else {
                // no auto_stake: issue free tickets to the user for immediate fill
                self.mint(
                    &self.ticket_mint,
                    &self.ticket_settlement,
                    order_summary.base_filled(),
                )?;
                0
            }
        } else {
            0
        };
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
            order_summary.quote_combined()?,
        )?;

        Ok(staked)
    }
}

pub fn handler(ctx: Context<LendOrder>, params: OrderParams, seed: Vec<u8>) -> Result<()> {
    if !ctx.accounts.authority.is_signer {
        return err!(FixedTermErrorCode::MissingAuthoritySignature);
    }
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
    ctx.accounts.lend(
        ctx.accounts.authority.key(),
        &seed,
        0,
        callback_info,
        &order_summary,
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
