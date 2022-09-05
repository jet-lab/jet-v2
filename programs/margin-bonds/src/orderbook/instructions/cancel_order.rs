use agnostic_orderbook::{
    instruction::cancel_order,
    state::{critbit::Slab, get_side_from_order_id, Side},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token};

use crate::{
    control::state::BondManager,
    events::OrderCancelled,
    orderbook::state::{CallbackFlags, CallbackInfo},
    utils::{mint_to, orderbook_accounts, transfer_context},
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

    /// The `BondManager` account tracks global information related to this particular bond market
    #[account(
        has_one = orderbook_market_state @ BondsError::WrongMarketState,
        has_one = bids @ BondsError::WrongBids,
        has_one = asks @ BondsError::WrongAsks,
        has_one = event_queue @ BondsError::WrongEventQueue,
        constraint = !bond_manager.load()?.orderbook_paused,
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

    /// Spl token program
    pub token_program: Program<'info, Token>,
}

impl<'info> CancelOrder<'info> {
    pub fn check_order_ownership(&self, callback: &CallbackInfo) -> Result<()> {
        if callback.account_key == self.user.key().to_bytes() {
            Ok(())
        } else {
            err!(BondsError::WrongMarginUser)
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

    let orderbook_params = cancel_order::Params { order_id };
    let order_summary = agnostic_orderbook::instruction::cancel_order::process::<CallbackInfo>(
        ctx.program_id,
        orderbook_accounts!(ctx, cancel_order),
        orderbook_params,
    )?;

    // credit the user account with unused funds
    match side {
        Side::Bid => {
            transfer(
                transfer_context!(ctx, user_vault, market_account, bond_manager)
                    .with_signer(&[&ctx.accounts.bond_manager.load()?.authority_seeds()]),
                order_summary.total_quote_qty,
            )?;
        }
        Side::Ask => {
            if !callback_flags.contains(CallbackFlags::NEW_DEBT) {
                mint_to!(
                    ctx,
                    market_account,
                    user_vault,
                    order_summary.total_base_qty
                )?;
            }
        }
    }

    emit!(OrderCancelled {
        bond_manager: ctx.accounts.bond_manager.key(),
        user: ctx.accounts.user.key(),
        order_id,
    });

    Ok(())
}
