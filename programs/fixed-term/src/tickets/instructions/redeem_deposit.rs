use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    control::state::Market,
    market_token_manager::MarketTokenManager,
    tickets::{events::DepositRedeemed, state::TermDeposit},
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct RedeemDeposit<'info> {
    /// The tracking account for the deposit
    #[account(mut,
              close = payer,
              has_one = owner
    )]
    pub deposit: Account<'info, TermDeposit>,

    /// The account that owns the deposit
    #[account(mut)]
    pub owner: AccountInfo<'info>,

    /// The authority that must sign to redeem the deposit
    pub authority: Signer<'info>,

    /// Receiver for the rent used to track the deposit
    pub payer: AccountInfo<'info>,

    /// The token account designated to receive the assets underlying the claim
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,

    /// The Market responsible for the asset
    #[account(
        has_one = underlying_token_vault @ FixedTermErrorCode::WrongVault,
        constraint = !market.load()?.tickets_paused @ FixedTermErrorCode::TicketsPaused,
    )]
    pub market: AccountLoader<'info, Market>,

    /// The vault stores the tokens of the underlying asset managed by the Market
    #[account(mut)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

    /// SPL token program
    pub token_program: Program<'info, Token>,
}

impl<'info> RedeemDeposit<'info> {
    pub fn redeem(&self) -> Result<u64> {
        let current_time = Clock::get()?.unix_timestamp;
        if current_time < self.deposit.matures_at {
            msg!(
                "Matures at time: [{:?}]\nCurrent time: [{:?}]",
                self.deposit.matures_at,
                current_time
            );
            return err!(FixedTermErrorCode::ImmatureTicket);
        }

        // transfer from the vault to the ticket_holder
        self.withdraw(
            &self.underlying_token_vault,
            &self.token_account,
            self.deposit.amount,
        )?;

        emit!(DepositRedeemed {
            market: self.market.key(),
            ticket_holder: self.owner.key(),
            redeemed_value: self.deposit.amount,
            maturation_timestamp: self.deposit.matures_at,
            redeemed_timestamp: current_time,
        });

        Ok(self.deposit.amount)
    }
}

pub fn handler(ctx: Context<RedeemDeposit>) -> Result<()> {
    if ctx.accounts.owner.key != ctx.accounts.authority.key {
        msg!(
            "signer {} is not the deposit owner {}",
            ctx.accounts.authority.key,
            ctx.accounts.owner.key
        );

        return Err(FixedTermErrorCode::DoesNotOwnTicket.into());
    }

    let _ = ctx.accounts.redeem()?;

    Ok(())
}
