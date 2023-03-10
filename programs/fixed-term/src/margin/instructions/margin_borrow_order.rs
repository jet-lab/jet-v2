use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::get_associated_token_address, token::Token};
use jet_margin::{AdapterResult, PositionChange};
use jet_program_common::traits::SafeSub;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    events::TermLoanCreated,
    margin::{
        events::{OrderPlaced, OrderType},
        origination_fee::loan_to_disburse,
        state::{return_to_margin, AutoRollConfig, MarginUser, TermLoan, TermLoanFlags},
    },
    market_token_manager::MarketTokenManager,
    orderbook::state::*,
    serialization::{self, RemainingAccounts},
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct MarginBorrowOrder<'info> {
    /// The account tracking borrower debts
    #[account(
        mut,
        has_one = margin_account,
        has_one = claims @ FixedTermErrorCode::WrongClaimAccount,
        has_one = ticket_collateral @ FixedTermErrorCode::WrongTicketCollateralAccount,
    )]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// TermLoan account minted upon match
    /// CHECK: in instruction logic
    #[account(mut)]
    pub term_loan: AccountInfo<'info>,

    /// The margin account for this borrow order
    pub margin_account: Signer<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    /// CHECK: margin_user
    #[account(mut)]
    pub claims: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    /// CHECK: in instruction handler
    #[account(mut, address = orderbook_mut.claims_mint() @ FixedTermErrorCode::WrongClaimMint)]
    pub claims_mint: AccountInfo<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub ticket_collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut, address = orderbook_mut.ticket_collateral_mint() @ FixedTermErrorCode::WrongCollateralMint)]
    pub ticket_collateral_mint: AccountInfo<'info>,

    /// The market token vault
    #[account(mut, address = orderbook_mut.vault() @ FixedTermErrorCode::WrongVault)]
    pub underlying_token_vault: AccountInfo<'info>,

    /// The market fee vault
    #[account(mut, address = orderbook_mut.fee_vault() @ FixedTermErrorCode::WrongVault)]
    pub fee_vault: AccountInfo<'info>,

    /// Where to receive borrowed tokens
    #[account(mut, address = get_associated_token_address(
        &margin_user.margin_account,
        &orderbook_mut.underlying_mint(),
    ))]
    pub underlying_settlement: AccountInfo<'info>,

    #[market]
    pub orderbook_mut: OrderbookMut<'info>,

    /// payer for `TermLoan` initialization
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana system program
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
    // Optional event adapter account
    // pub event_adapter: AccountInfo<'info>,
}

pub fn handler(ctx: Context<MarginBorrowOrder>, mut params: OrderParams) -> Result<()> {
    let origination_fee = {
        let manager = ctx.accounts.orderbook_mut.market.load()?;
        params.max_ticket_qty = manager.borrow_order_qty(params.max_ticket_qty);
        params.max_underlying_token_qty = manager.borrow_order_qty(params.max_underlying_token_qty);
        manager.origination_fee
    };
    let auto_roll = if params.auto_roll {
        if ctx.accounts.margin_user.borrow_roll_config == AutoRollConfig::default() {
            msg!(
                "Auto roll settings have not been configured for margin user [{}]",
                ctx.accounts.margin_user.key()
            );
            return err!(FixedTermErrorCode::InvalidAutoRollConfig);
        }
        CallbackFlags::AUTO_ROLL
    } else {
        CallbackFlags::default()
    };
    let (callback_info, order_summary) = ctx.accounts.orderbook_mut.place_margin_order(
        Side::Ask,
        params,
        ctx.accounts.margin_account.key(),
        ctx.accounts.margin_user.key(),
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
        CallbackFlags::NEW_DEBT | CallbackFlags::MARGIN | auto_roll,
    )?;

    let debt = &mut ctx.accounts.margin_user.debt;
    debt.post_borrow_order(order_summary.base_posted())?;
    if order_summary.base_filled() > 0 {
        let manager = ctx.accounts.orderbook_mut.market.load()?;
        let maturation_timestamp = manager.borrow_tenor as i64 + Clock::get()?.unix_timestamp;
        let sequence_number =
            debt.new_term_loan_without_posting(order_summary.base_filled(), maturation_timestamp)?;

        let mut term_loan = serialization::init::<TermLoan>(
            ctx.accounts.term_loan.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            &[
                crate::seeds::TERM_LOAN,
                ctx.accounts.orderbook_mut.market.key().as_ref(),
                ctx.accounts.margin_user.key().as_ref(),
                &sequence_number.to_le_bytes(),
            ],
        )?;
        let quote_filled = order_summary.quote_filled()?;
        let disburse = manager.loan_to_disburse(quote_filled);
        let fees = quote_filled.safe_sub(disburse)?;
        ctx.withdraw(
            &ctx.accounts.underlying_token_vault,
            &ctx.accounts.fee_vault,
            fees,
        )?;
        let base_filled = order_summary.base_filled();
        *term_loan = TermLoan {
            sequence_number,
            margin_user: ctx.accounts.margin_user.key(),
            market: ctx.accounts.orderbook_mut.market.key(),
            payer: ctx.accounts.payer.key(),
            order_tag: callback_info.order_tag,
            maturation_timestamp,
            balance: base_filled,
            flags: TermLoanFlags::default(),
        };
        drop(manager);
        ctx.withdraw(
            &ctx.accounts.underlying_token_vault,
            &ctx.accounts.underlying_settlement,
            disburse,
        )?;

        emit!(TermLoanCreated {
            term_loan: term_loan.key(),
            authority: ctx.accounts.margin_account.key(),
            payer: ctx.accounts.payer.key(),
            order_tag: callback_info.order_tag.as_u128(),
            sequence_number,
            market: ctx.accounts.orderbook_mut.market.key(),
            maturation_timestamp,
            quote_filled,
            base_filled,
            flags: term_loan.flags,
            fees,
        });
    }
    let total_debt = order_summary.base_combined();
    ctx.mint(&ctx.accounts.claims_mint, &ctx.accounts.claims, total_debt)?;
    ctx.mint(
        &ctx.accounts.ticket_collateral_mint,
        &ctx.accounts.ticket_collateral,
        loan_to_disburse(order_summary.quote_posted()?, origination_fee),
    )?;

    emit!(OrderPlaced {
        market: ctx.accounts.orderbook_mut.market.key(),
        authority: ctx.accounts.margin_account.key(),
        margin_user: Some(ctx.accounts.margin_user.key()),
        order_tag: callback_info.order_tag.as_u128(),
        order_summary: order_summary.summary(),
        limit_price: params.limit_price,
        auto_stake: params.auto_stake,
        post_only: params.post_only,
        post_allowed: params.post_allowed,
        order_type: OrderType::MarginBorrow,
    });
    ctx.accounts.margin_user.emit_debt_balances();

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
