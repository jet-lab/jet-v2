use anchor_lang::prelude::*;

use crate::orderbook::state::*;

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    /// The owner of the order
    pub owner: Signer<'info>,

    pub orderbook_mut: OrderbookMut<'info>,
}

/// remove order from the book
pub fn handler(ctx: Context<CancelOrder>, order_id: u128) -> Result<()> {
    ctx.accounts
        .orderbook_mut
        .cancel_order(order_id, ctx.accounts.owner.key())?;

    Ok(())
}
