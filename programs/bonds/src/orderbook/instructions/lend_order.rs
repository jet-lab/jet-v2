use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::{accessor::mint, Mint, Token, TokenAccount};
use proc_macros::BondTokenManager;

use crate::{
    control::state::BondManager,
    orderbook::state::*,
    serialization::{self, RemainingAccounts},
    tickets::state::SplitTicket,
    utils::{ctx, mint_to},
    BondsError,
};

#[derive(Accounts, BondTokenManager)]
pub struct LendOrder<'info> {
    /// Signing authority over the token vault transferring for a lend order
    pub authority: Signer<'info>,

    #[bond_manager]
    pub orderbook_mut: OrderbookMut<'info>,

    /// where to settle tickets on match:
    /// - SplitTicket that will be created if the order is filled as a taker and `auto_stake` is enabled
    /// - ticket token account to receive bond tickets
    /// be careful to check this properly. one way is by using lender_tickets_token_account
    #[account(mut)]
    ticket_settlement: AccountInfo<'info>,

    /// where to loan tokens from
    #[account(mut, constraint = mint(&lender_tokens.to_account_info())? == orderbook_mut.underlying_mint() @ BondsError::WrongUnderlyingTokenMint)]
    pub lender_tokens: Account<'info, TokenAccount>,

    /// The market token vault
    #[account(mut, address = orderbook_mut.vault() @ BondsError::WrongVault)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

    /// The market token vault
    #[account(mut, address = orderbook_mut.ticket_mint() @ BondsError::WrongTicketMint)]
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
            BondsError::WrongTicketMint
        );

        Ok(self.ticket_settlement.key())
    }

    /// returns the amount of tickets staked
    pub fn lend(
        &self,
        user: Pubkey,
        seed: &[u8],
        callback_info: CallbackInfo,
        order_summary: &SensibleOrderSummary,
        bond_manager: &AccountLoader<'info, BondManager>,
    ) -> Result<u64> {
        let staked = if order_summary.base_filled() > 0 {
            if callback_info.flags.contains(CallbackFlags::AUTO_STAKE) {
                // auto_stake: issue split tickets to the user for immediate fill
                let mut split_ticket = serialization::init::<SplitTicket>(
                    self.ticket_settlement.to_account_info(),
                    self.payer.to_account_info(),
                    self.system_program.to_account_info(),
                    &SplitTicket::make_seeds(user.as_ref(), seed),
                )?;
                let timestamp = Clock::get()?.unix_timestamp;
                *split_ticket = SplitTicket {
                    owner: user,
                    bond_manager: bond_manager.key(),
                    order_tag: callback_info.order_tag,
                    struck_timestamp: timestamp,
                    maturation_timestamp: timestamp + bond_manager.load()?.duration,
                    principal: order_summary.quote_filled()?,
                    interest: order_summary.base_filled() - order_summary.quote_filled()?,
                };
                order_summary.base_filled()
            } else {
                // no auto_stake: issue free tickets to the user for immediate fill
                mint_to!(
                    ctx(self),
                    ticket_mint,
                    ticket_settlement,
                    order_summary.base_filled()
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
        callback_info,
        &order_summary,
        &ctx.accounts.orderbook_mut.bond_manager,
    )?;
    emit!(crate::events::LendOrder {
        bond_market: ctx.accounts.orderbook_mut.bond_manager.key(),
        lender: ctx.accounts.authority.key(),
        order_summary: order_summary.summary(),
    });

    Ok(())
}
