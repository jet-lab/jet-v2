use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use jet_margin::MarginAccount;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    margin::state::MarginUser,
    orderbook::state::*,
    serialization::RemainingAccounts,
    tickets::state::{
        MarginRedeemDepositAccounts, RedeemDepositAccounts, TermDeposit, TermDepositFlags,
    },
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct AutoRollLendOrder<'info> {
    /// The `MarginUser` account for this market
    #[account(
        mut,
        constraint = margin_user.market == orderbook_mut.market.key() @ FixedTermErrorCode::WrongMarket,
        has_one = margin_account @ FixedTermErrorCode::WrongMarginAccount,
        has_one = ticket_collateral @ FixedTermErrorCode::WrongTicketCollateralAccount ,
	)]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// The `MarginAccount` this `TermDeposit` belongs to
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The `TermDeposit` account to roll
    #[account(
        mut,
        constraint = deposit.owner == margin_account.key() @ FixedTermErrorCode::WrongDepositOwner,
        constraint = deposit.payer == rent_receiver.key() @ FixedTermErrorCode::WrongRentReceiver,
    )]
    pub deposit: Box<Account<'info, TermDeposit>>,

    /// In the case the order matches, the new `TermDeposit` to account for
    #[account(mut)]
    pub new_deposit: AccountInfo<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(
        mut,
        constraint = ticket_collateral.mint == ticket_collateral_mint.key() @ FixedTermErrorCode::WrongTicketCollateralAccount,
    )]
    pub ticket_collateral: Box<Account<'info, TokenAccount>>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(
        mut,
        address = orderbook_mut.ticket_collateral_mint() @ FixedTermErrorCode::WrongTicketCollateralMint,
    )]
    pub ticket_collateral_mint: Box<Account<'info, Mint>>,

    /// The market token vault
    #[account(
        mut,
        address = orderbook_mut.ticket_mint() @ FixedTermErrorCode::WrongTicketMint
    )]
    pub ticket_mint: Account<'info, Mint>,

    /// The market token vault
    #[account(mut, address = orderbook_mut.vault() @ FixedTermErrorCode::WrongVault)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

    /// Reciever for rent from the closing of the TermDeposit
    #[account(mut)]
    pub rent_receiver: AccountInfo<'info>,

    /// The accounts needed to interact with the orderbook
    #[market]
    pub orderbook_mut: OrderbookMut<'info>,

    /// Payer for PDA initialization
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> AutoRollLendOrder<'info> {
    fn margin_redeem(&mut self) -> Result<()> {
        let accounts = &mut MarginRedeemDepositAccounts {
            margin_user: &mut self.margin_user.clone(),
            ticket_collateral: self.ticket_collateral.as_ref().as_ref(),
            ticket_collateral_mint: self.ticket_collateral_mint.as_ref().as_ref(),
            inner: &RedeemDepositAccounts {
                deposit: &self.deposit,
                owner: self.margin_account.as_ref(),
                payer: &self.rent_receiver,
                token_account: &self.new_deposit, // not needed for this instruction, arbitrary account
                market: &self.orderbook_mut.market,
                underlying_token_vault: &self.underlying_token_vault,
                token_program: &self.token_program,
            },
        };
        accounts.margin_redeem(false)
    }

    fn margin_lend_order(&mut self, params: &OrderParams, adapter: Option<Pubkey>) -> Result<()> {
        let accounts = &mut MarginLendAccounts {
            margin_user: &mut self.margin_user,
            ticket_collateral: self.ticket_collateral.as_ref().as_ref(),
            ticket_collateral_mint: self.ticket_collateral_mint.as_ref().as_ref(),
            inner: &mut LendOrderAccounts {
                authority: self.margin_account.as_ref(),
                orderbook_mut: &mut self.orderbook_mut,
                ticket_settlement: &self.new_deposit,
                lender_tokens: &self.new_deposit, // not needed for this instruction, arbitrary account
                underlying_token_vault: &self.underlying_token_vault,
                ticket_mint: &self.ticket_mint,
                payer: &self.payer,
                system_program: &self.system_program,
                token_program: &self.token_program,
            },
        };
        accounts.margin_lend_order(params, adapter, false)
    }

    fn order_params(&self) -> Result<OrderParams> {
        let config = match &self.margin_user.lend_roll_config {
            Some(config) => config,
            None => return err!(FixedTermErrorCode::InvalidAutoRollConfig),
        };

        Ok(OrderParams {
            max_ticket_qty: u64::MAX,
            max_underlying_token_qty: self.deposit.amount,
            limit_price: config.limit_price,
            match_limit: u64::MAX,
            post_only: false,
            post_allowed: true,
            auto_stake: true,
            auto_roll: true,
        })
    }

    fn assert_deposit_can_auto_roll(&self) -> Result<()> {
        if !self.deposit.flags.contains(TermDepositFlags::AUTO_ROLL) {
            return err!(FixedTermErrorCode::AutoRollDisabled);
        }
        Ok(())
    }
}

pub fn handler(ctx: Context<AutoRollLendOrder>) -> Result<()> {
    ctx.accounts.assert_deposit_can_auto_roll()?;
    ctx.accounts.margin_redeem()?;
    let params = ctx.accounts.order_params()?;
    let adapter = ctx
        .remaining_accounts
        .iter()
        .maybe_next_adapter()?
        .map(|a| a.key());
    ctx.accounts.margin_lend_order(&params, adapter)
}
