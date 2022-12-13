use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

use crate::{
    control::state::Market,
    tickets::{events::DepositRedeemed, state::TermDeposit},
    FixedTermErrorCode,
};

#[derive(Accounts)]
pub struct RedeemDeposit<'info> {
    /// The tracking account for the deposit
    #[account(mut,
              close = owner,
              has_one = owner
    )]
    pub deposit: Account<'info, TermDeposit>,

    /// The account that owns the deposit
    #[account(mut)]
    pub owner: Signer<'info>,

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
        transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                Transfer {
                    to: self.token_account.to_account_info(),
                    from: self.underlying_token_vault.to_account_info(),
                    authority: self.market.to_account_info(),
                },
            )
            .with_signer(&[&self.market.load()?.authority_seeds()]),
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
    let _ = ctx.accounts.redeem()?;

    Ok(())
}
