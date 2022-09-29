use anchor_lang::prelude::*;

use crate::{orderbook::state::*, BondsError};

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    /// The signing authority for this user account
    pub user: Signer<'info>,

    pub orderbook_mut: OrderbookMut<'info>,
}

/// remove order from the book
pub fn handler(ctx: Context<CancelOrder>, order_id: u128) -> Result<()> {
    let (_, callback_flags, _) = ctx
        .accounts
        .orderbook_mut
        .cancel_order(order_id, ctx.accounts.user.key())?;

    require!(
        !callback_flags.contains(CallbackFlags::MARGIN),
        BondsError::UnauthorizedCaller
    );

    Ok(())
}
