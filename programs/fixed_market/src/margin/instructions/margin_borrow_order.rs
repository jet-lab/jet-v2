use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use jet_margin::{AdapterResult, PositionChange};
use jet_program_common::traits::{SafeSub, TryAddAssign};
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    events::ObligationCreated,
    margin::{
        events::{OrderPlaced, OrderType},
        origination_fee::loan_to_disburse,
        state::{return_to_margin, MarginUser, Obligation, ObligationFlags},
    },
    market_token_manager::MarketTokenManager,
    orderbook::state::*,
    serialization::{self, RemainingAccounts},
    ErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct MarginBorrowOrder<'info> {
    /// The account tracking borrower debts
    #[account(
        mut,
        has_one = margin_account,
        has_one = claims @ ErrorCode::WrongClaimAccount,
        has_one = collateral @ ErrorCode::WrongCollateralAccount,
    )]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// Obligation account minted upon match
    /// CHECK: in instruction logic
    #[account(mut)]
    pub obligation: AccountInfo<'info>,

    /// The margin account for this borrow order
    pub margin_account: Signer<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    /// CHECK: borrower_account
    #[account(mut)]
    pub claims: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    /// CHECK: in instruction handler
    #[account(mut, address = orderbook_mut.claims_mint() @ ErrorCode::WrongClaimMint)]
    pub claims_mint: AccountInfo<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut, address = orderbook_mut.collateral_mint() @ ErrorCode::WrongCollateralMint)]
    pub collateral_mint: AccountInfo<'info>,

    /// The market token vault
    #[account(mut, address = orderbook_mut.vault() @ ErrorCode::WrongVault)]
    pub underlying_token_vault: AccountInfo<'info>,

    /// The market token vault
    #[account(mut, address = margin_user.underlying_settlement @ ErrorCode::WrongUnderlyingSettlementAccount)]
    pub underlying_settlement: AccountInfo<'info>,

    #[market]
    pub orderbook_mut: OrderbookMut<'info>,

    /// payer for `Obligation` initialization
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana system program
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
    // Optional event adapter account
    // pub event_adapter: AccountInfo<'info>,
}

pub fn handler(
    ctx: Context<MarginBorrowOrder>,
    mut params: OrderParams,
    seed: Vec<u8>,
) -> Result<()> {
    let origination_fee = {
        let manager = ctx.accounts.orderbook_mut.market.load()?;
        params.max_market_ticket_qty = manager.borrow_order_qty(params.max_market_ticket_qty);
        params.max_underlying_token_qty = manager.borrow_order_qty(params.max_underlying_token_qty);
        manager.origination_fee
    };
    let (callback_info, order_summary) = ctx.accounts.orderbook_mut.place_order(
        ctx.accounts.margin_account.key(),
        Side::Ask,
        params,
        ctx.accounts.margin_user.key(),
        ctx.accounts.margin_user.key(),
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
        CallbackFlags::NEW_DEBT | CallbackFlags::MARGIN,
    )?;

    let debt = &mut ctx.accounts.margin_user.debt;
    debt.post_borrow_order(order_summary.base_posted())?;
    if order_summary.base_filled() > 0 {
        let mut manager = ctx.accounts.orderbook_mut.market.load_mut()?;
        let maturation_timestamp = manager.borrow_tenor + Clock::get()?.unix_timestamp;
        let sequence_number =
            debt.new_obligation_without_posting(order_summary.base_filled(), maturation_timestamp)?;

        let mut obligation = serialization::init::<Obligation>(
            ctx.accounts.obligation.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            &Obligation::make_seeds(ctx.accounts.margin_user.key().as_ref(), seed.as_slice()),
        )?;
        let quote_filled = order_summary.quote_filled()?;
        let disburse = manager.loan_to_disburse(quote_filled);
        manager
            .collected_fees
            .try_add_assign(quote_filled.safe_sub(disburse)?)?;
        let base_filled = order_summary.base_filled();
        *obligation = Obligation {
            sequence_number,
            borrower_account: ctx.accounts.margin_user.key(),
            market: ctx.accounts.orderbook_mut.market.key(),
            order_tag: callback_info.order_tag,
            maturation_timestamp,
            balance: base_filled,
            flags: ObligationFlags::default(),
        };
        drop(manager);
        ctx.withdraw(
            &ctx.accounts.underlying_token_vault,
            &ctx.accounts.underlying_settlement,
            disburse,
        )?;

        emit!(ObligationCreated {
            obligation: obligation.key(),
            authority: ctx.accounts.margin_account.key(),
            order_id: order_summary.summary().posted_order_id,
            sequence_number,
            market: ctx.accounts.orderbook_mut.market.key(),
            maturation_timestamp,
            quote_filled,
            base_filled,
            flags: obligation.flags
        });
    }
    let total_debt = order_summary.base_combined();
    ctx.mint(&ctx.accounts.claims_mint, &ctx.accounts.claims, total_debt)?;
    ctx.mint(
        &ctx.accounts.collateral_mint,
        &ctx.accounts.collateral,
        loan_to_disburse(order_summary.quote_posted()?, origination_fee),
    )?;

    emit!(OrderPlaced {
        market: ctx.accounts.orderbook_mut.market.key(),
        authority: ctx.accounts.margin_account.key(),
        margin_user: Some(ctx.accounts.margin_user.key()),
        order_summary: order_summary.summary(),
        limit_price: params.limit_price,
        auto_stake: params.auto_stake,
        post_only: params.post_only,
        post_allowed: params.post_allowed,
        order_type: OrderType::MarginBorrow,
    });

    // this is just used to make sure the position is still registered.
    // it's actually registered by initialize_margin_user
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
