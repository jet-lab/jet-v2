use anchor_lang::{prelude::*, AccountsClose};
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};
use jet_proto_math::traits::TrySubAssign;

use crate::{
    events::{ObligationFulfilled, ObligationRepay},
    orderbook::state::{
        debt::{Obligation, ObligationFlags},
        user::OrderbookUser,
    },
    BondsError,
};

#[derive(Accounts)]
pub struct Repay<'info> {
    /// The account tracking information related to this particular user
    pub orderbook_user_account: Account<'info, OrderbookUser>,

    #[account(
        mut,
        has_one = orderbook_user_account @ BondsError::UserNotInMarket,
    )]
    pub obligation: Account<'info, Obligation>,

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
    transfer(ctx.accounts.transfer_context(), amount)?;

    let obligation = &mut ctx.accounts.obligation;
    let user = &mut ctx.accounts.orderbook_user_account;

    obligation.balance.try_sub_assign(amount)?;

    if obligation.flags.contains(ObligationFlags::MARKED_DUE) {
        user.debt.repay_past_due(amount)?;
    } else {
        user.debt.repay_committed(amount)?;
    }

    emit!(ObligationRepay {
        orderbook_user: ctx.accounts.orderbook_user_account.key(),
        obligation: obligation.key(),
        repayment_amount: amount,
        final_balance: obligation.balance,
    });

    if obligation.balance == 0 {
        emit!(ObligationFulfilled {
            obligation: obligation.clone().into_inner(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        obligation.close(ctx.accounts.payer.to_account_info())?;
        ctx.accounts
            .orderbook_user_account
            .outstanding_obligations
            .try_sub_assign(1)?;
    }

    Ok(())
}
