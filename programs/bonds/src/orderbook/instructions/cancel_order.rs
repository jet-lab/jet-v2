use agnostic_orderbook::{
    instruction::cancel_order,
    state::{critbit::Slab, get_side_from_order_id, OrderSummary, Side},
};
use anchor_lang::prelude::*;
use jet_proto_math::traits::TryAddAssign;

use crate::{
    control::state::BondManager,
    events::OrderCancelled,
    orderbook::state::{user::OrderbookUser, CallbackFlags, CallbackInfo},
    BondsError,
};

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    /// The account tracking information related to this particular user
    #[account(
        mut,
        has_one = bond_manager @ BondsError::UserNotInMarket,
        has_one = user @ BondsError::UserDoesNotOwnAccount
    )]
    pub orderbook_user_account: Account<'info, OrderbookUser>,

    /// The signing authority for this user account
    pub user: Signer<'info>,

    /// The `BondManager` account tracks global information related to this particular bond market
    #[account(
        has_one = orderbook_market_state @ BondsError::WrongMarketState
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,

    // aaob accounts
    /// CHECK: handled by aaob
    #[account(mut, owner = crate::ID @ BondsError::MarketStateNotProgramOwned)]
    pub orderbook_market_state: AccountInfo<'info>,
    /// CHECK: handled by aaob
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    /// CHECK: handled by aaob
    #[account(mut)]
    pub bids: AccountInfo<'info>,
    /// CHECK: handled by aaob
    #[account(mut)]
    pub asks: AccountInfo<'info>,
}

impl<'info> CancelOrder<'info> {
    pub fn check_order_ownership(&self, callback: &CallbackInfo) -> Result<()> {
        if callback.orderbook_account_key == self.orderbook_user_account.key().to_bytes() {
            Ok(())
        } else {
            err!(BondsError::WrongOrderbookUser)
        }
    }

    /// This executes the logic instead of returning CallbackInfo to avoid having to clone CallbackInfo
    pub fn use_callback<T, F: Fn(&CallbackInfo) -> T>(
        &self,
        side: Side,
        order_id: u128,
        logic: F,
    ) -> Result<T> {
        let mut buf;
        let mut slab = match side {
            Side::Bid => {
                buf = self.bids.data.borrow_mut();
                Slab::from_buffer(&mut buf, agnostic_orderbook::state::AccountTag::Bids)?
            }
            Side::Ask => {
                buf = self.bids.data.borrow_mut();
                Slab::from_buffer(&mut buf, agnostic_orderbook::state::AccountTag::Asks)?
            }
        };

        let (_, info) = slab.remove_by_key(order_id).unwrap();

        Ok(logic(info))
    }
}

/// remove order from the book
pub fn handler(ctx: Context<CancelOrder>, order_id: u128) -> Result<()> {
    let side = get_side_from_order_id(order_id);
    let callback_flags =
        ctx.accounts
            .use_callback(side, order_id, |callback| -> Result<CallbackFlags> {
                ctx.accounts.check_order_ownership(callback)?;

                Ok(callback.flags)
            })??;

    let orderbook_accounts = cancel_order::Accounts {
        market: &ctx.accounts.orderbook_market_state.to_account_info(),
        event_queue: &ctx.accounts.event_queue.to_account_info(),
        bids: &ctx.accounts.bids.to_account_info(),
        asks: &ctx.accounts.asks.to_account_info(),
    };
    let orderbook_params = cancel_order::Params { order_id };
    let order_summary: OrderSummary = agnostic_orderbook::instruction::cancel_order::process::<
        CallbackInfo,
    >(ctx.program_id, orderbook_accounts, orderbook_params)?;

    // credit the user account with unused funds
    match side {
        Side::Bid => {
            ctx.accounts
                .orderbook_user_account
                .underlying_token_stored
                .try_add_assign(order_summary.total_quote_qty)?;
        }
        Side::Ask => {
            if callback_flags.contains(CallbackFlags::NEW_DEBT) {
                ctx.accounts
                    .orderbook_user_account
                    .debt
                    .cancel_pending(order_summary.total_quote_qty)?;
            } else {
                ctx.accounts
                    .orderbook_user_account
                    .bond_tickets_stored
                    .try_add_assign(order_summary.total_base_qty)?;
            }
        }
    }

    emit!(OrderCancelled {
        bond_manager: ctx.accounts.bond_manager.key(),
        orderbook_user: ctx.accounts.orderbook_user_account.key(),
        order_id,
    });

    Ok(())
}
