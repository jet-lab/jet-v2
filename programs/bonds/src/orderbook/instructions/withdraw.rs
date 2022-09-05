use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};
use jet_proto_math::traits::TrySubAssign;

use crate::{
    control::state::BondManager,
    events::OrderbookWithdraw,
    orderbook::state::{user::OrderbookUser, AssetKind},
    utils::mint_to,
    BondsError,
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    /// The `BondManager` account tracks global information related to this particular bond market
    #[account(
        has_one = underlying_token_vault @ BondsError::WrongVault,
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,

    /// The account tracking information related to this particular user
    #[account(
        mut,
        has_one = bond_manager @ BondsError::UserNotInMarket,
        has_one = user @ BondsError::UserDoesNotOwnAccount
    )]
    pub orderbook_user_account: Account<'info, OrderbookUser>,

    /// The signing authority for this user account
    pub user: Signer<'info>,

    /// The token vault to recieve excess funds, specified by the user
    #[account(mut)]
    pub user_token_vault: Account<'info, TokenAccount>,

    /// The vault holding the quote tokens of this bond market
    #[account(mut)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

    /// The minting account for the bond tickets
    #[account(mut)]
    pub bond_ticket_mint: Account<'info, Mint>,

    /// SPL token program
    pub token_program: Program<'info, Token>,
}

impl<'info> Withdraw<'info> {
    pub fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.underlying_token_vault.to_account_info(),
                to: self.user_token_vault.to_account_info(),
                authority: self.bond_manager.to_account_info(),
            },
        )
    }
}

pub fn handler(ctx: Context<Withdraw>, amount: u64, kind: AssetKind) -> Result<()> {
    match kind {
        AssetKind::BondTicket => {
            ctx.accounts
                .orderbook_user_account
                .bond_tickets_stored
                .try_sub_assign(amount)?;

            mint_to!(ctx, bond_ticket_mint, user_token_vault, amount)
        }
        AssetKind::UnderlyingToken => {
            ctx.accounts
                .orderbook_user_account
                .underlying_token_stored
                .try_sub_assign(amount)?;

            transfer(ctx.accounts.transfer_context(), amount)
        }
    }?;

    emit!(OrderbookWithdraw {
        bond_manager: ctx.accounts.bond_manager.key(),
        orderbook_user: ctx.accounts.orderbook_user_account.key(),
        amount,
        kind,
    });

    Ok(())
}
