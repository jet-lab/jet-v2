use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::get_associated_token_address, token::Token};
use jet_margin::{AdapterResult, PositionChange};
use jet_program_common::traits::SafeSub;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    margin::{
        events::{OrderPlaced, OrderType},
        state::{return_to_margin, AutoRollConfig, MarginUser, TermLoanBuilder},
    },
    market_token_manager::MarketTokenManager,
    orderbook::state::*,
    serialization::RemainingAccounts,
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct MarginBorrowOrder<'info> {
    /// The account tracking borrower debts
    #[account(
        mut,
        has_one = margin_account,
        has_one = claims @ FixedTermErrorCode::WrongClaimAccount,
        has_one = underlying_collateral @ FixedTermErrorCode::WrongTicketCollateralAccount,
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
    pub underlying_collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut, address = orderbook_mut.underlying_collateral_mint() @ FixedTermErrorCode::WrongCollateralMint)]
    pub underlying_collateral_mint: AccountInfo<'info>,

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

impl<'info> MarginBorrowOrder<'info> {
    pub fn callback_flags(&self, params: &OrderParams) -> Result<CallbackFlags> {
        let auto_roll = if params.auto_roll {
            if self.margin_user.borrow_roll_config == AutoRollConfig::default() {
                msg!(
                    "Auto roll settings have not been configured for margin user [{}]",
                    self.margin_user.key()
                );
                return err!(FixedTermErrorCode::InvalidAutoRollConfig);
            }
            CallbackFlags::AUTO_ROLL
        } else {
            CallbackFlags::default()
        };

        let flags = CallbackFlags::NEW_DEBT | CallbackFlags::MARGIN | auto_roll;
        Ok(flags)
    }
}

/// Accounting for the posted portion of the borrow order
fn handle_posted(
    ctx: &mut Context<MarginBorrowOrder>,
    summary: &SensibleOrderSummary,
) -> Result<()> {
    let posted_token_value = summary.quote_posted(RoundingAction::PostBorrow.direction())?;
    let posted_ticket_value = summary.base_posted();

    ctx.accounts
        .margin_user
        .post_borrow_order(posted_token_value, posted_ticket_value)?;

    // collateralize the tokens involved in the order
    ctx.mint(
        &ctx.accounts.underlying_collateral_mint,
        &ctx.accounts.underlying_collateral,
        posted_token_value,
    )?;

    Ok(())
}

/// Handle the accounting for the filled portion of the order.
/// Returns total disbursed after fee accounting.
fn handle_filled(
    ctx: &mut Context<MarginBorrowOrder>,
    summary: &SensibleOrderSummary,
    info: &MarginCallbackInfo,
) -> Result<u64> {
    let filled_ticket_value = summary.base_filled();
    let filled_token_value = summary.quote_filled(RoundingAction::FillBorrow.direction())?;
    let current_time = Clock::get()?.unix_timestamp;
    let maturation_timestamp =
        ctx.accounts.orderbook_mut.market.load()?.borrow_tenor as i64 + current_time;

    let sequence_number = ctx
        .accounts
        .margin_user
        .taker_fill_borrow_order(filled_ticket_value, maturation_timestamp)?;

    let disburse = ctx
        .accounts
        .orderbook_mut
        .market
        .load()?
        .loan_to_disburse(filled_token_value);
    let fees = filled_token_value.safe_sub(disburse)?;

    // write a TermLoan account
    let builder = TermLoanBuilder::new_from_order(
        ctx.accounts,
        summary,
        info,
        current_time,
        maturation_timestamp,
        fees,
        sequence_number,
    )?;
    builder.init_and_write(
        &ctx.accounts.term_loan,
        &ctx.accounts.payer,
        &ctx.accounts.system_program,
    )?;

    // Allot the borrower the tokens from the filled order
    ctx.withdraw(
        &ctx.accounts.underlying_token_vault,
        &ctx.accounts.underlying_settlement,
        disburse,
    )?;

    // Collect fees from the order fill
    ctx.withdraw(
        &ctx.accounts.underlying_token_vault,
        &ctx.accounts.fee_vault,
        fees,
    )?;

    Ok(disburse)
}

pub fn handler(mut ctx: Context<MarginBorrowOrder>, mut params: OrderParams) -> Result<()> {
    ctx.accounts
        .orderbook_mut
        .market
        .load()?
        .add_origination_fee(&mut params);

    let (callback_info, order_summary) = ctx.accounts.orderbook_mut.place_margin_order(
        Side::Ask,
        params,
        ctx.accounts.margin_account.key(),
        ctx.accounts.margin_user.key(),
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
        ctx.accounts.callback_flags(&params)?,
    )?;

    handle_posted(&mut ctx, &order_summary)?;
    if order_summary.base_filled() > 0 {
        handle_filled(&mut ctx, &order_summary, &callback_info)?;
    }

    // place a claim for the borrowed tokens
    ctx.mint(
        &ctx.accounts.claims_mint,
        &ctx.accounts.claims,
        order_summary.base_combined(),
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
