use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use jet_margin::MarginAccount;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    margin::state::MarginUser,
    orderbook::state::*,
    serialization::RemainingAccounts,
    tickets::state::{
        margin_redeem, MarginRedeemDepositAccounts, RedeemDepositAccounts, TermDeposit,
        TermDepositFlags,
    },
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct AutoRollLendOrder<'info> {
    /// The `TermDeposit` account to roll
    #[account(mut)]
    pub deposit: Box<Account<'info, TermDeposit>>,

    /// In the case the order matches, the new `TermDeposit` to account for
    #[account(mut)]
    pub new_deposit: AccountInfo<'info>,

    /// The underlying token account belonging to the lender, required for downstream checks
    pub lender_tokens: Box<Account<'info, TokenAccount>>,

    /// The `MarginAccount` this `TermDeposit` belongs to
    #[account(mut)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    /// The `MarginUser` account for this market
    #[account(mut)]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// The accounts needed to interact with the orderbook
    #[market]
    pub orderbook_mut: OrderbookMut<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub ticket_collateral: Box<Account<'info, TokenAccount>>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub ticket_collateral_mint: Box<Account<'info, Mint>>,

    /// The market token vault
    #[account(mut, address = orderbook_mut.ticket_mint() @ FixedTermErrorCode::WrongTicketMint)]
    pub ticket_mint: Account<'info, Mint>,

    /// The market token vault
    #[account(mut, address = orderbook_mut.vault() @ FixedTermErrorCode::WrongVault)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

    /// Reciever for rent from the closing of the TermDeposit
    #[account(mut)]
    pub rent_receiver: AccountInfo<'info>,

    /// Payer for PDA initialization
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> AutoRollLendOrder<'info> {
    #[inline(never)]
    fn lend_order(&self, adapter: Option<Pubkey>) -> Result<()> {
        let accounts = &mut MarginLendAccounts {
            margin_user: self.margin_user.clone(),
            ticket_collateral: &self.ticket_collateral.to_account_info(),
            ticket_collateral_mint: &self.ticket_collateral_mint.to_account_info(),
            inner: &LendOrderAccounts {
                authority: &self.margin_account.to_account_info(),
                orderbook_mut: &self.orderbook_mut,
                ticket_settlement: &self.new_deposit,
                lender_tokens: &self.lender_tokens,
                underlying_token_vault: &self.underlying_token_vault,
                ticket_mint: &self.ticket_mint,
                payer: &self.payer,
                system_program: &self.system_program,
                token_program: &self.token_program,
            },
            adapter,
        };
        accounts.margin_lend_order(&self.order_params(), false)
    }

    #[inline(never)]
    fn redeem(&mut self) -> Result<()> {
        let accounts = &mut MarginRedeemDepositAccounts {
            margin_user: self.margin_user.clone(),
            ticket_collateral: &self.ticket_collateral.to_account_info(),
            ticket_collateral_mint: &self.ticket_collateral_mint.to_account_info(),
            inner: &RedeemDepositAccounts {
                deposit: &self.deposit,
                owner: &self.margin_user.to_account_info(),
                authority: &self.margin_account.to_account_info(),
                payer: &self.rent_receiver,
                token_account: &self.lender_tokens,
                market: &self.orderbook_mut.market,
                underlying_token_vault: &self.underlying_token_vault,
                token_program: &self.token_program,
            },
        };

        margin_redeem(accounts, false)
    }

    fn order_params(&self) -> OrderParams {
        OrderParams {
            max_ticket_qty: u64::MAX,
            max_underlying_token_qty: self.deposit.amount,
            limit_price: self.margin_user.lend_roll_config.limit_price,
            match_limit: u64::MAX,
            post_only: false,
            post_allowed: true,
            auto_stake: true,
            auto_roll: true,
        }
    }

    fn assert_deposit_is_auto_roll(&self) -> Result<()> {
        if !self.deposit.flags.contains(TermDepositFlags::AUTO_ROLL) {
            return err!(FixedTermErrorCode::TermDepositIsNotAutoRoll);
        }
        Ok(())
    }
}

pub fn handler(ctx: Context<AutoRollLendOrder>) -> Result<()> {
    ctx.accounts.assert_deposit_is_auto_roll()?;
    ctx.accounts.redeem()?;
    ctx.accounts.lend_order(
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
    )
}
