use agnostic_orderbook::{
    instruction::new_order,
    state::{SelfTradeBehavior, Side},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token};
use jet_margin::{AdapterResult, PositionChange};

use crate::{
    control::state::BondManager,
    margin::{
        events::MarginBorrow,
        state::{return_to_margin, MarginUser, Obligation, ObligationFlags},
    },
    orderbook::state::{CallbackFlags, CallbackInfo, OrderParams},
    serialization,
    utils::{mint_to, orderbook_accounts},
    BondsError,
};

#[derive(Accounts)]
pub struct MarginBorrowOrder<'info> {
    /// The account tracking borrower debts
    #[account(
        mut,
        has_one = margin_account,
    )]
    pub borrower_account: Box<Account<'info, MarginUser>>,

    /// Token vault to recieve borrowed tokens
    /// CHECK: assertion in instruction logic
    pub user_token_vault: AccountInfo<'info>,

    /// Obligation account minted upon match
    /// CHECK: in instruction logic
    #[account(mut)]
    pub obligation: AccountInfo<'info>,

    /// The margin account for this borrow order
    pub margin_account: Signer<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    /// CHECK: constraint
    #[account(
        mut,
        constraint =
            borrower_account.claims == claims.key()
            @ BondsError::WrongClaimAccount
    )]
    pub claims: UncheckedAccount<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    /// CHECK: in instruction handler
    #[account(mut)]
    pub claims_mint: UncheckedAccount<'info>,

    /// The `BondManager` account tracks global information related to this particular bond market
    #[account(
            mut,
            has_one = orderbook_market_state @ BondsError::WrongMarketState,
            has_one = bids @ BondsError::WrongBids,
            has_one = asks @ BondsError::WrongAsks,
            has_one = event_queue @ BondsError::WrongEventQueue,
        )]
    pub bond_manager: AccountLoader<'info, BondManager>,

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

    /// Solana system program
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
    // Optional event adapter account
    // pub event_adapter: AccountInfo<'info>,
}

pub fn handler(ctx: Context<MarginBorrowOrder>, params: OrderParams, seed: u64) -> Result<()> {
    let mut manager = ctx.accounts.bond_manager.load_mut()?;

    // asserts
    if manager.orderbook_paused {
        return err!(BondsError::OrderbookPaused);
    }
    if manager.claims_mint != ctx.accounts.claims_mint.key() {
        return err!(BondsError::WrongClaimMint);
    }
    if manager.underlying_token_mint != token::accessor::mint(&ctx.accounts.user_token_vault)? {
        return err!(BondsError::WrongVault);
    }

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

    let callback_info = CallbackInfo::new(
        ctx.accounts.bond_manager.key(),
        ctx.accounts.borrower_account.key(),
        ctx.accounts.user_token_vault.key(),
        adapter_key,
        params.callback_flags() | CallbackFlags::NEW_DEBT,
        manager.nonce,
    );
    manager.nonce += 1;

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

    if order_summary.total_base_qty > 0 {
        let mut obligation = serialization::init::<Obligation>(
            ctx.accounts.obligation.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            &[
                crate::seeds::OBLIGATION,
                ctx.accounts.borrower_account.key().as_ref(),
                &seed.to_le_bytes(),
            ],
        )?;
        *obligation = Obligation {
            borrower_account: ctx.accounts.borrower_account.key(),
            bond_manager: ctx.accounts.bond_manager.key(),
            order_tag: callback_info.order_tag,
            maturation_timestamp: manager.duration + Clock::get()?.unix_timestamp,
            balance: order_summary.total_base_qty,
            flags: ObligationFlags::default(),
        };
    }
    let total_debt = order_summary.total_base_qty_posted + order_summary.total_base_qty;
    mint_to!(ctx, claims_mint, claims, total_debt)?;
    ctx.accounts.borrower_account.borrow(&order_summary)?;

    emit!(MarginBorrow {
        bond_manager: ctx.accounts.bond_manager.key(),
        margin_account: ctx.accounts.margin_account.key(),
        borrower_account: ctx.accounts.borrower_account.key(),
        order_summary,
    });

    return_to_margin(
        &ctx.accounts.margin_account.to_account_info(),
        &AdapterResult {
            position_changes: vec![(
                ctx.accounts.claims_mint.key(),
                vec![PositionChange::Register(ctx.accounts.claims.key())],
            )],
        },
    )
}
