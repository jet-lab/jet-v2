use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    control::state::Market,
    market_token_manager::MarketTokenManager,
    tickets::{
        events::DepositRedeemed,
        state::{redeem, RedeemDepositAccounts, TermDeposit},
    },
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct RedeemDeposit<'info> {
    /// The tracking account for the deposit
    #[account(mut,
              close = payer,
              has_one = owner,
              has_one = payer
    )]
    pub deposit: Account<'info, TermDeposit>,

    /// The account that owns the deposit
    #[account(mut)]
    pub owner: AccountInfo<'info>,

    /// The authority that must sign to redeem the deposit
    ///
    /// Signature check is handled in instruction logic
    pub authority: AccountInfo<'info>,

    /// Receiver for the rent used to track the deposit
    #[account(mut)]
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
    /// Account for the redemption of the `TermDeposit`
    ///
    /// in the case that this function is downstream from an auto rolled lend order, there is
    /// no need to withdraw funds from the vault, and `is_withdrawing` should be false
    #[inline(never)]
    pub fn redeem(&self, is_withdrawing: bool) -> Result<u64> {
        let current_time = Clock::get()?.unix_timestamp;
        if current_time < self.deposit.matures_at {
            msg!(
                "Matures at time: [{:?}]\nCurrent time: [{:?}]",
                self.deposit.matures_at,
                current_time
            );
            return err!(FixedTermErrorCode::ImmatureTicket);
        }

        // transfer from the vault to the deposit_holder
        if is_withdrawing {
            self.withdraw(
                &self.underlying_token_vault,
                &self.token_account,
                self.deposit.amount,
            )?;
        }

        emit!(DepositRedeemed {
            deposit: self.deposit.key(),
            deposit_holder: self.owner.key(),
            redeemed_value: self.deposit.amount,
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

    let accs = ctx.accounts;
    let accounts = RedeemDepositAccounts {
        deposit: &accs.deposit,
        owner: &accs.owner,
        authority: &accs.authority,
        payer: &accs.payer,
        token_account: &accs.token_account,
        market: &accs.market,
        underlying_token_vault: &accs.underlying_token_vault,
        token_program: &accs.token_program,
    };
    redeem(&accounts, true)?;

    Ok(())
}
