use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::{
    orderbook::state::*,
    utils::{mint_to, withdraw},
    BondsError,
};

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    /// The signing authority for this user account
    pub user: Signer<'info>,

    /// The vault to collect regained funds
    /// CHECK: Serialization and checks handled by program logic
    pub user_vault: AccountInfo<'info>,

    /// Account controlled by the market to disperse funds
    /// Bond ticket mint or underlying vault depending on book side
    /// CHECK: handled by program logic
    pub market_account: AccountInfo<'info>,

    pub orderbook_mut: OrderbookMut<'info>,

    /// Spl token program
    pub token_program: Program<'info, Token>,
}

/// remove order from the book
pub fn handler(ctx: Context<CancelOrder>, order_id: u128) -> Result<()> {
    let (side, callback_flags, order_summary) = ctx
        .accounts
        .orderbook_mut
        .cancel_order(order_id, ctx.accounts.user.key())?;

    require!(
        !callback_flags.contains(CallbackFlags::MARGIN),
        BondsError::UnauthorizedCaller
    );
    // credit the user account with unused funds
    // todo is this redundant with consume_events for Out events?
    match side {
        Side::Bid => {
            withdraw!(
                ctx,
                market_account,
                user_vault,
                order_summary.total_quote_qty,
                orderbook_mut
            )?;
        }
        Side::Ask => {
            if !callback_flags.contains(CallbackFlags::NEW_DEBT) {
                mint_to!(
                    ctx,
                    market_account,
                    user_vault,
                    order_summary.total_base_qty,
                    orderbook_mut
                )?;
            }
        }
    }

    Ok(())
}
