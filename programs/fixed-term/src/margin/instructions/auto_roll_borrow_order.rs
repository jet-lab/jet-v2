use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use jet_program_proc_macros::MarketTokenManager;

use crate::{
    margin::state::{MarginUser, RepayAccounts, TermLoan, TermLoanFlags},
    orderbook::state::*,
    serialization::RemainingAccounts,
    FixedTermErrorCode,
};

#[derive(Accounts, MarketTokenManager)]
pub struct AutoRollBorrowOrder<'info> {
    /// The `MarginUser` account for this market
    #[account(
        mut,
        constraint = margin_user.market == orderbook_mut.market.key() @ FixedTermErrorCode::WrongMarket,
        has_one = underlying_collateral @ FixedTermErrorCode::WrongUnderlyingCollateralAccount,
        has_one = margin_account @ FixedTermErrorCode::WrongMarginAccount,
        has_one = claims @ FixedTermErrorCode::WrongClaimAccount,
	)]
    pub margin_user: Box<Account<'info, MarginUser>>,

    /// The `MarginAccount` this `TermDeposit` belongs to
    pub margin_account: AccountInfo<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    /// CHECK: margin_user
    #[account(mut)]
    pub claims: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut, address = orderbook_mut.claims_mint() @ FixedTermErrorCode::WrongClaimMint)]
    pub claims_mint: AccountInfo<'info>,

    /// Token account used by the margin program to track the debt that must be collateralized
    #[account(mut)]
    pub underlying_collateral: AccountInfo<'info>,

    /// Token mint used by the margin program to track the debt that must be collateralized
    #[account(mut, address = orderbook_mut.underlying_collateral_mint() @ FixedTermErrorCode::WrongUnderlyingCollateralMint)]
    pub underlying_collateral_mint: AccountInfo<'info>,

    /// The market token vault
    #[account(mut, address = orderbook_mut.vault() @ FixedTermErrorCode::WrongVault)]
    pub underlying_token_vault: AccountInfo<'info>,

    /// The market fee vault
    #[account(mut, address = orderbook_mut.fee_vault() @ FixedTermErrorCode::WrongVault)]
    pub fee_vault: AccountInfo<'info>,

    /// The `TermDeposit` account to roll
    #[account(
        mut,
        has_one = margin_user @ FixedTermErrorCode::WrongMarginUser,
        constraint = loan.payer == rent_receiver.key() @ FixedTermErrorCode::WrongRentReceiver,
    )]
    pub loan: Box<Account<'info, TermLoan>>,

    /// In the case the order matches, the new `TermLoan` to account for
    #[account(mut)]
    pub new_loan: AccountInfo<'info>,

    /// The next `TermLoan` that is due after the repayment of the current
    pub next_unpaid_loan: AccountInfo<'info>,

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

impl<'info> AutoRollBorrowOrder<'info> {
    /// Take any available liquidity on the book in order to repay the loan
    /// Uses the limit price set by the user in the `BorrowRollConfig`
    pub fn borrow_now(
        &mut self,
        params: OrderParams,
        event_adapter: Option<Pubkey>,
    ) -> Result<u64> {
        MarginBorrowOrderAccounts {
            margin_user: &mut self.margin_user,
            term_loan: &self.new_loan,
            margin_account: &self.margin_account.to_account_info(),
            claims: &self.claims,
            claims_mint: &self.claims_mint,
            underlying_collateral: &self.underlying_collateral,
            underlying_collateral_mint: &self.underlying_collateral_mint,
            underlying_token_vault: &self.underlying_token_vault,
            fee_vault: &self.fee_vault,
            // returns funds directly to the market
            underlying_settlement: &self.underlying_token_vault,
            orderbook_mut: &mut self.orderbook_mut,
            payer: &self.payer.to_account_info(),
            system_program: &self.system_program.to_account_info(),
            token_program: &self.token_program.to_account_info(),
            event_adapter,
        }
        .borrow_order(params)
    }

    /// Uses the newly borrowed tokens to repay the loan
    pub fn repay(&mut self, amount: u64) -> Result<()> {
        RepayAccounts {
            margin_user: &mut self.margin_user,
            term_loan: &mut self.loan,
            next_term_loan: &self.next_unpaid_loan,
            // arbitrary here
            source: &self.margin_account,
            source_authority: &self.margin_account,

            claims: &self.claims,
            claims_mint: &self.claims_mint,
            payer: &self.payer,
            underlying_token_vault: &self.underlying_token_vault,
            market: &self.orderbook_mut.market,
            token_program: &self.token_program,
        }
        .repay(amount, true)
    }

    fn params(&self) -> Result<OrderParams> {
        let config = match &self.margin_user.borrow_roll_config {
            Some(config) => config,
            None => return err!(FixedTermErrorCode::InvalidAutoRollConfig),
        };

        let mut params = OrderParams {
            max_ticket_qty: u64::MAX,
            max_underlying_token_qty: self.loan.balance,
            limit_price: config.limit_price,
            match_limit: u64::MAX,
            post_only: false,
            post_allowed: false,
            auto_stake: false,
            auto_roll: true,
        };
        self.orderbook_mut
            .market
            .load()?
            .add_origination_fee(&mut params);

        Ok(params)
    }

    fn assert_can_auto_roll(&self) -> Result<()> {
        if !self.loan.flags.contains(TermLoanFlags::AUTO_ROLL) {
            return err!(FixedTermErrorCode::AutoRollDisabled);
        }

        let config = match &self.margin_user.borrow_roll_config {
            Some(config) => config,
            None => return err!(FixedTermErrorCode::InvalidAutoRollConfig),
        };

        let current_time = Clock::get()?.unix_timestamp as u64;
        let auto_roll_threshold = config.roll_tenor + self.loan.strike_timestamp as u64;

        if current_time < auto_roll_threshold {
            msg!(
                "cannot auto roll before {} (currently {})",
                auto_roll_threshold,
                current_time
            );
            return err!(FixedTermErrorCode::AutoRollNotReady);
        }

        Ok(())
    }
}

pub fn handler(ctx: Context<AutoRollBorrowOrder>) -> Result<()> {
    ctx.accounts.assert_can_auto_roll()?;

    let filled = ctx.accounts.borrow_now(
        ctx.accounts.params()?,
        ctx.remaining_accounts
            .iter()
            .maybe_next_adapter()?
            .map(|a| a.key()),
    )?;
    ctx.accounts.repay(filled)
}
