use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use jet_margin::MarginAccount;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    control::state::Market,
    margin::state::MarginUser,
    tickets::state::{MarginRedeemDepositAccounts, RedeemDepositAccounts, TermDeposit},
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct MarginRedeemDeposit<'info> {
    #[account(mut,
		has_one = margin_account @ FixedTermErrorCode::WrongMarginUserAuthority,
        has_one = ticket_collateral,
	)]
    pub margin_user: Box<Account<'info, MarginUser>>,

    #[account(signer)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// Token account used by the margin program to track the collateral value of assets custodied by fixed-term market
    #[account(mut)]
    pub ticket_collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the collateral value of assets custodied by fixed-term market
    #[account(mut, address = market.load()?.ticket_collateral_mint)]
    pub ticket_collateral_mint: AccountInfo<'info>,

    /// The tracking account for the deposit
    #[account(mut,
        close = payer,
        constraint = deposit.owner == margin_account.key() @ FixedTermErrorCode::WrongDepositOwner,
        has_one = payer
)]
    pub deposit: Account<'info, TermDeposit>,

    /// Receiver for the rent used to track the deposit
    #[account(mut)]
    pub payer: AccountInfo<'info>,

    /// The token account designated to receive the assets underlying the claim
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,

    /// The Market responsible for the asset
    #[account(
        has_one = underlying_token_vault @ FixedTermErrorCode::WrongVault,
        constraint = !market.load()?.tickets_paused.as_bool() @ FixedTermErrorCode::TicketsPaused,
    )]
    pub market: AccountLoader<'info, Market>,

    /// The vault stores the tokens of the underlying asset managed by the Market
    #[account(mut)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

    /// SPL token program
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<MarginRedeemDeposit>) -> Result<()> {
    let accs = ctx.accounts;
    let accounts = &mut MarginRedeemDepositAccounts {
        margin_user: &mut accs.margin_user,
        ticket_collateral: &accs.ticket_collateral,
        ticket_collateral_mint: &accs.ticket_collateral_mint,
        inner: &RedeemDepositAccounts {
            deposit: &accs.deposit,
            owner: accs.margin_account.as_ref(),
            payer: &accs.payer,
            token_account: accs.token_account.as_ref(),
            market: &accs.market,
            underlying_token_vault: &accs.underlying_token_vault,
            token_program: &accs.token_program,
        },
    };
    accounts.margin_redeem(true)
}
