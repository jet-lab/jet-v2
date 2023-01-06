use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use jet_margin::MarginAccount;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    instructions::{lend_order::LendOrder, MarginLendOrder, MarginRedeemDeposit, RedeemDeposit},
    margin::state::MarginUser,
    orderbook::state::*,
    serialization::RemainingAccounts,
    tickets::state::TermDeposit,
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct AutoRollLendOrder<'info> {
    /// The off chain service authorized to roll this lend order
    pub roll_servicer: Signer<'info>,

    pub deposit: Account<'info, TermDeposit>,

    pub new_deposit: Account<'info, TermDeposit>,

    #[account(mut)]
    pub lender_tokens: Account<'info, TokenAccount>,

    #[account(mut)]
    pub margin_account: AccountLoader<'info, MarginAccount>,

    #[account(mut)]
    pub margin_user: Box<Account<'info, MarginUser>>,

    #[market]
    pub orderbook_mut: OrderbookMut<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub ticket_collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub ticket_collateral_mint: AccountInfo<'info>,

    /// The market token vault
    #[account(mut, address = orderbook_mut.ticket_mint() @ FixedTermErrorCode::WrongTicketMint)]
    pub ticket_mint: Account<'info, Mint>,

    /// The market token vault
    #[account(mut, address = orderbook_mut.vault() @ FixedTermErrorCode::WrongVault)]
    pub underlying_token_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> AutoRollLendOrder<'info> {
    fn lend_order(&self, adapter: Option<Pubkey>) -> Result<()> {
        let params = self.order_params();
        let mut lend_accounts = MarginLendOrder {
            margin_user: self.margin_user.clone(),
            ticket_collateral: self.ticket_collateral.clone(),
            ticket_collateral_mint: self.ticket_collateral_mint.clone(),
            inner: LendOrder {
                authority: self.margin_account.to_account_info(),
                orderbook_mut: self.orderbook_mut.clone(),
                ticket_settlement: self.new_deposit.to_account_info(),
                lender_tokens: self.lender_tokens.clone(),
                underlying_token_vault: self.underlying_token_vault.clone(),
                ticket_mint: self.ticket_mint.clone(),
                payer: self.payer.clone(),
                system_program: self.system_program.clone(),
                token_program: self.token_program.clone(),
            },
        };
        lend_accounts.lend_order(params, adapter)?;
        Ok(())
    }

    fn redeem(&self) -> Result<()> {
        let mut redemption_accounts = MarginRedeemDeposit {
            margin_user: self.margin_user.clone(),
            ticket_collateral: self.ticket_collateral.clone(),
            ticket_collateral_mint: self.ticket_collateral_mint.clone(),
            inner: RedeemDeposit {
                deposit: self.deposit.clone(),
                owner: self.margin_account.to_account_info(),
                authority: self.margin_account.to_account_info(),
                payer: self.payer.to_account_info(),
                token_account: self.lender_tokens.clone(),
                market: self.orderbook_mut.market.clone(),
                underlying_token_vault: self.underlying_token_vault.clone(),
                token_program: self.token_program.clone(),
            },
        };

        redemption_accounts.redeem()
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
}

pub fn handler(ctx: Context<AutoRollLendOrder>) -> Result<()> {
    ctx.accounts.redeem()?;
    ctx.accounts.lend_order(
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
    )
}
