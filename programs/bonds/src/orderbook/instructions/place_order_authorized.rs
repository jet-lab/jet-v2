use std::collections::BTreeMap;

use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use jet_margin::{AdapterResult, PositionChange};
use jet_proto_math::traits::TrySubAssign;

use crate::{
    control::state::BondManager,
    events::MarginBorrow,
    margin::return_to_margin,
    orderbook::state::{CallbackFlags, OrderParams, OrderSide},
    utils::mint_to,
    BondsError,
};

use super::place_order::*;

#[derive(Accounts)]
pub struct PlaceOrderAuthorized<'info> {
    /// All the same accounts are required as place_order. Just extra authority is needed
    pub base_accounts: PlaceOrder<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut, constraint =
        base_accounts.orderbook_user_account.claims == claims.key()
        @ BondsError::WrongClaimAccount
    )]
    pub claims: UncheckedAccount<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut, constraint =
        base_accounts.bond_manager.load()?.claims_mint == claims_mint.key()
        @ BondsError::WrongClaimMint
    )]
    pub claims_mint: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

impl<'info> PlaceOrderAuthorized<'info> {
    fn bond_manager(&self) -> &AccountLoader<'info, BondManager> {
        &self.base_accounts.bond_manager
    }
}

pub fn handler<'a, 'b, 'info>(
    ctx: Context<'a, 'b, 'b, 'info, PlaceOrderAuthorized<'info>>,
    side: OrderSide,
    params: OrderParams,
) -> Result<()> {
    let order_summary = place_order_blindly(
        Context::new(
            ctx.program_id,
            &mut ctx.accounts.base_accounts,
            ctx.remaining_accounts,
            BTreeMap::new(),
        ),
        side,
        params,
        CallbackFlags::NEW_DEBT,
    )?;

    let adapter_result = match side {
        OrderSide::Borrow => {
            // borrower is allowed to take on debt instead of selling a bond ticket
            ctx.accounts
                .base_accounts
                .orderbook_user_account
                .debt
                .add_pending_debt(order_summary.total_quote_qty)?;
            mint_to!(ctx, claims_mint, claims, order_summary.total_quote_qty, ())?;
            AdapterResult {
                position_changes: vec![(
                    ctx.accounts.claims_mint.key(),
                    vec![PositionChange::Register(ctx.accounts.claims.key())],
                )],
            }
        }
        OrderSide::Lend => {
            // lenders are not allowed to take on debt to lend
            ctx.accounts
                .base_accounts
                .orderbook_user_account
                .underlying_token_stored
                .try_sub_assign(order_summary.total_quote_qty)?;
            AdapterResult::default()
        }
    };

    emit!(MarginBorrow {
        bond_manager: ctx.accounts.bond_manager().key(),
        orderbook_user: ctx.accounts.base_accounts.orderbook_user_account.key(),
        order_summary,
    });

    return_to_margin(
        &ctx.accounts.base_accounts.user.to_account_info(),
        &adapter_result,
    )
}
