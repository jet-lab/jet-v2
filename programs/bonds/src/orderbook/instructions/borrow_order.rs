use agnostic_orderbook::{
    instruction::new_order,
    state::{SelfTradeBehavior, Side},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    control::state::BondManager,
    orderbook::state::{CallbackInfo, OrderParams},
    utils::orderbook_accounts,
    BondsError,
};

#[derive(Accounts)]
pub struct BorrowOrder<'info> {
    /// Signing authority over the ticket vault transferring for a borrow order
    pub user: Signer<'info>,

    /// Account containing the bond tickets being sold
    #[account(mut)]
    pub user_ticket_vault: Account<'info, TokenAccount>,

    /// The account to recieve the matched tokens
    #[account(mut)]
    pub user_token_vault: Account<'info, TokenAccount>,

    /// The `BondManager` account tracks global information related to this particular bond market
    #[account(
        mut,
        has_one = bond_ticket_mint @ BondsError::WrongTicketMint,
        has_one = orderbook_market_state @ BondsError::WrongMarketState,
        has_one = bids @ BondsError::WrongBids,
        has_one = asks @ BondsError::WrongAsks,
        has_one = event_queue @ BondsError::WrongEventQueue,
        constraint = !bond_manager.load()?.orderbook_paused,
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,

    /// The market ticket mint
    #[account(mut)]
    pub bond_ticket_mint: Account<'info, Mint>,

    // aaob accounts
    /// CHECK: handled by has_one on bond_manager
    #[account(mut)]
    pub orderbook_market_state: AccountInfo<'info>,
    /// CHECK: handled by has_one on bond_manager
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    /// CHECK: handled by has_one on bond_manager
    #[account(mut)]
    pub bids: AccountInfo<'info>,
    /// CHECK: handled by has_one on bond_manager
    #[account(mut)]
    pub asks: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    // Optional event adapter account
    // pub event_adapter: AccountInfo<'info>,
}

pub fn handler(ctx: Context<BorrowOrder>, params: OrderParams) -> Result<()> {
    let OrderParams {
        max_bond_ticket_qty,
        max_underlying_token_qty,
        limit_price,
        match_limit,
        post_only,
        post_allowed,
        auto_stake: _,
    } = params;

    let adapter_key = match ctx.remaining_accounts.iter().next() {
        Some(adapter) => *adapter.key,
        None => Pubkey::default(),
    };

    let mut manager = ctx.accounts.bond_manager.load_mut()?;
    let callback_info = CallbackInfo::new(
        ctx.accounts.bond_manager.key(),
        ctx.accounts.user.key(),
        ctx.accounts.user_token_vault.key(),
        adapter_key,
        params.callback_flags(),
        manager.nonce,
    );
    manager.nonce += 1;
    drop(manager);

    let order_params = new_order::Params {
        max_base_qty: max_bond_ticket_qty,
        max_quote_qty: max_underlying_token_qty,
        limit_price,
        match_limit,
        side: Side::Ask,
        callback_info,
        post_only,
        post_allowed,
        self_trade_behavior: SelfTradeBehavior::AbortTransaction,
    };
    let order_summary = new_order::process(
        ctx.program_id,
        orderbook_accounts!(ctx, new_order),
        order_params,
    )?;

    anchor_spl::token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Burn {
                mint: ctx.accounts.bond_ticket_mint.to_account_info(),
                from: ctx.accounts.user_ticket_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        order_summary.total_base_qty,
    )?;
    emit!(crate::events::BorrowOrder {
        bond_market: ctx.accounts.bond_manager.key(),
        borrower: ctx.accounts.user.key(),
        order_summary,
    });

    Ok(())
}
