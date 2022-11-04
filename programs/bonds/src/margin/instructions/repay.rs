use std::cmp::min;

use anchor_lang::{prelude::*, AccountsClose};
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};
use jet_program_common::traits::TrySubAssign;

use crate::{
    events::{ObligationFulfilled, ObligationRepay},
    margin::state::{MarginUser, Obligation},
    BondsError,
};

#[derive(Accounts)]
pub struct Repay<'info> {
    /// The account tracking information related to this particular user
    pub borrower_account: Account<'info, MarginUser>,

    #[account(
        mut,
        has_one = borrower_account @ BondsError::UserNotInMarket,
        constraint = obligation.sequence_number
            == borrower_account.debt.next_obligation_to_repay().unwrap()
            @ BondsError::ObligationHasWrongSequenceNumber
    )]
    pub obligation: Account<'info, Obligation>,

    /// No payment will be made towards next_obligation: it is needed purely for bookkeeping.
    /// if the user has additional obligations, this must be the one with the following sequence number.
    /// otherwise, put whatever address you want in here
    pub next_obligation: AccountInfo<'info>,

    /// The token account to deposit tokens from
    #[account(mut)]
    pub source: Account<'info, TokenAccount>,

    /// The signing authority for the source_account
    pub payer: Signer<'info>,

    /// The token vault holding the underlying token of the bond
    #[account(mut)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

    /// SPL token program
    pub token_program: Program<'info, Token>,
}

impl<'info> Repay<'info> {
    pub fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.source.to_account_info(),
                to: self.underlying_token_vault.to_account_info(),
                authority: self.payer.to_account_info(),
            },
        )
    }
}

pub fn handler(ctx: Context<Repay>, amount: u64) -> Result<()> {
    let amount = min(amount, ctx.accounts.obligation.balance);
    transfer(ctx.accounts.transfer_context(), amount)?;

    let obligation = &mut ctx.accounts.obligation;
    let user = &mut ctx.accounts.borrower_account;

    obligation.balance.try_sub_assign(amount)?;

    if obligation.balance > 0 {
        user.debt
            .partially_repay_obligation(obligation.sequence_number, amount)?;
    } else {
        emit!(ObligationFulfilled {
            obligation: obligation.key(),
            orderbook_user: user.key(),
            borrower: obligation.borrower_account,
            timestamp: Clock::get()?.unix_timestamp,
        });

        obligation.close(ctx.accounts.payer.to_account_info())?;

        let user_key = user.key();
        let next_obligation = Account::<Obligation>::try_from(&ctx.accounts.next_obligation)
            .and_then(|ob| {
                require_eq!(ob.borrower_account, user_key, BondsError::UserNotInMarket);
                Ok(ob)
            });
        user.debt
            .fully_repay_obligation(obligation.sequence_number, amount, next_obligation)?;
    }

    emit!(ObligationRepay {
        orderbook_user: ctx.accounts.borrower_account.key(),
        obligation: obligation.key(),
        repayment_amount: amount,
        final_balance: obligation.balance,
    });

    Ok(())
}
