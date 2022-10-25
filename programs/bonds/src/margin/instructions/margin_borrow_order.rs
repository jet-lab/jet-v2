use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use jet_margin::{AdapterResult, PositionChange};
use proc_macros::BondTokenManager;

use crate::{
    bond_token_manager::BondTokenManager,
    margin::{
        events::MarginBorrow,
        state::{return_to_margin, MarginUser, Obligation, ObligationFlags},
    },
    orderbook::state::*,
    serialization::{self, RemainingAccounts},
    BondsError,
};

#[derive(Accounts, BondTokenManager)]
pub struct MarginBorrowOrder<'info> {
    /// The account tracking borrower debts
    #[account(
        mut,
        has_one = margin_account,
        has_one = claims @ BondsError::WrongClaimAccount,
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
    #[account(mut)]
    pub claims_mint: AccountInfo<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub collateral_mint: AccountInfo<'info>,

    #[bond_manager]
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

pub fn handler(ctx: Context<MarginBorrowOrder>, params: OrderParams, seed: Vec<u8>) -> Result<()> {
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
    let bond_manager = &ctx.accounts.orderbook_mut.bond_manager;

    let debt = &mut ctx.accounts.margin_user.debt;
    debt.post_borrow_order(order_summary.base_posted())?;
    if order_summary.base_filled() > 0 {
        let maturation_timestamp = bond_manager.load()?.duration + Clock::get()?.unix_timestamp;
        let sequence_number =
            debt.new_obligation_without_posting(order_summary.base_filled(), maturation_timestamp)?;
        let mut obligation = serialization::init::<Obligation>(
            ctx.accounts.obligation.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            &Obligation::make_seeds(ctx.accounts.margin_user.key().as_ref(), seed.as_slice()),
        )?;
        ctx.accounts.margin_user.assets.entitled_tokens += order_summary.quote_filled()?;
        *obligation = Obligation {
            sequence_number,
            borrower_account: ctx.accounts.margin_user.key(),
            bond_manager: bond_manager.key(),
            order_tag: callback_info.order_tag,
            maturation_timestamp,
            balance: order_summary.base_filled(),
            flags: ObligationFlags::default(),
        };
    }
    let total_debt = order_summary.base_combined();
    ctx.mint(&ctx.accounts.claims_mint, &ctx.accounts.claims, total_debt)?;
    ctx.mint(
        &ctx.accounts.collateral_mint,
        &ctx.accounts.collateral,
        order_summary.quote_posted()?,
    )?;

    emit!(MarginBorrow {
        bond_manager: bond_manager.key(),
        margin_account: ctx.accounts.margin_account.key(),
        borrower_account: ctx.accounts.margin_user.key(),
        order_summary: order_summary.summary(),
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
