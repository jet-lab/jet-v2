use agnostic_orderbook::{
    instruction::new_order,
    state::{SelfTradeBehavior, Side},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount};

use crate::{
    control::state::BondManager,
    orderbook::state::{CallbackInfo, OrderParams},
    serialization,
    tickets::state::SplitTicket,
    utils::{orderbook_accounts, transfer_context},
    BondsError,
};

#[derive(Accounts)]
pub struct LendOrder<'info> {
    /// Signing authority over the token vault transferring for a lend order
    pub user: Signer<'info>,

    /// If auto stake is not enabled, the ticket account that will recieve the bond tickets
    pub user_ticket_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_vault: Account<'info, TokenAccount>,

    // split ticket minted upon match if `auto_stake` is enabled
    /// CHECK: initialized in instruction
    #[account(mut)]
    pub split_ticket: AccountInfo<'info>,

    /// The `BondManager` account tracks global information related to this particular bond market
    #[account(
        mut,
        has_one = underlying_token_vault @ BondsError::WrongVault,
        has_one = orderbook_market_state @ BondsError::WrongMarketState,
        has_one = bids @ BondsError::WrongBids,
        has_one = asks @ BondsError::WrongAsks,
        has_one = event_queue @ BondsError::WrongEventQueue,
        constraint = !bond_manager.load()?.orderbook_paused,
    )]
    pub bond_manager: AccountLoader<'info, BondManager>,

    /// The market token vault
    #[account(mut)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

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

    /// payer for `Obligation` initialization
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    // Optional event adapter account
    // pub event_adapter: AccountInfo<'info>,
}

pub fn handler(ctx: Context<LendOrder>, params: OrderParams, seed: Vec<u8>) -> Result<()> {
    let OrderParams {
        max_bond_ticket_qty,
        max_underlying_token_qty,
        limit_price,
        match_limit,
        post_only,
        post_allowed,
        auto_stake,
    } = params;

    let adapter_key = match ctx.remaining_accounts.iter().next() {
        Some(adapter) => *adapter.key,
        None => Pubkey::default(),
    };

    let mut manager = ctx.accounts.bond_manager.load_mut()?;
    let callback_info = CallbackInfo::new(
        ctx.accounts.bond_manager.key(),
        ctx.accounts.user.key(),
        ctx.accounts.user_ticket_vault.key(),
        adapter_key,
        params.callback_flags(),
        manager.nonce,
    );
    manager.nonce += 1;

    let order_params = new_order::Params {
        max_base_qty: max_bond_ticket_qty,
        max_quote_qty: max_underlying_token_qty,
        limit_price,
        match_limit,
        side: Side::Bid,
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

    if auto_stake {
        let mut split_ticket = serialization::init::<SplitTicket>(
            ctx.accounts.split_ticket.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            &[
                crate::seeds::SPLIT_TICKET,
                ctx.accounts.user.key().as_ref(),
                seed.as_slice(),
            ],
        )?;
        let timestamp = Clock::get()?.unix_timestamp;

        *split_ticket = SplitTicket {
            owner: ctx.accounts.user.key(),
            bond_manager: ctx.accounts.bond_manager.key(),
            order_tag: callback_info.order_tag,
            struck_timestamp: timestamp,
            maturation_timestamp: timestamp + manager.duration,
            principal: order_summary.total_quote_qty,
            interest: order_summary.total_base_qty - order_summary.total_quote_qty,
        }
    }
    transfer(
        transfer_context!(ctx, underlying_token_vault, user_token_vault, user),
        order_summary.total_quote_qty,
    )?;
    emit!(crate::events::LendOrder {
        bond_market: ctx.accounts.bond_manager.key(),
        lender: ctx.accounts.user.key(),
        order_summary,
    });

    Ok(())
}
