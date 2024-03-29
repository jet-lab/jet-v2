use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use jet_airspace::state::AirspacePermit;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    control::state::Market,
    tickets::state::{RedeemDepositAccounts, TermDeposit, TermDepositFlags},
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct RedeemDeposit<'info> {
    /// Metadata permit allowing this user to interact with this market
    #[account(
        has_one = owner @ FixedTermErrorCode::WrongAirspaceAuthorization,
        constraint = permit.airspace == market.load()?.airspace @ FixedTermErrorCode::WrongAirspaceAuthorization,
    )]
    pub permit: Account<'info, AirspacePermit>,

    /// The tracking account for the deposit
    #[account(mut,
              close = payer,
              has_one = market @ FixedTermErrorCode::WrongMarket,
              has_one = owner @ FixedTermErrorCode::WrongDepositOwner,
              has_one = payer
    )]
    pub deposit: Account<'info, TermDeposit>,

    /// The account that owns the deposit
    #[account(mut)]
    pub owner: Signer<'info>,

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

impl<'info> RedeemDeposit<'info> {
    fn check(&self) -> Result<()> {
        if self.deposit.flags.contains(TermDepositFlags::MARGIN) {
            return err!(FixedTermErrorCode::MarginUserCannotUseInstruction);
        }

        Ok(())
    }
}

pub fn handler(ctx: Context<RedeemDeposit>) -> Result<()> {
    let accs = ctx.accounts;
    accs.check()?;

    let accounts = RedeemDepositAccounts {
        deposit: &accs.deposit,
        owner: &accs.owner,
        payer: &accs.payer,
        token_account: accs.token_account.as_ref(),
        market: &accs.market,
        underlying_token_vault: &accs.underlying_token_vault,
        token_program: &accs.token_program,
    };
    accounts.redeem(true)?;

    Ok(())
}
