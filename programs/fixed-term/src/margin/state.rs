use std::ops::Range;

use anchor_lang::{prelude::*, solana_program::clock::UnixTimestamp};
use bytemuck::Zeroable;
use jet_margin::{AdapterResult, MarginAccount};
use jet_program_common::{
    interest_pricing::{InterestPricer, PricerImpl},
    traits::{SafeAdd, SafeSub, TryAddAssign, TrySubAssign},
    Fp32,
};

use crate::{
    events::{AssetsUpdated, DebtUpdated, TermLoanCreated},
    instructions::MarginBorrowOrder,
    orderbook::state::{MarginCallbackInfo, OrderTag, RoundingAction, SensibleOrderSummary},
    serialization::{self, AnchorAccount, Mut},
    FixedTermErrorCode,
};

pub const MARGIN_USER_VERSION: u8 = 0;

/// An acocunt used to track margin users of the market
#[account]
#[repr(C, align(8))]
#[derive(Debug)]
pub struct MarginUser {
    /// used to determine if a migration step is needed before user actions are allowed
    pub version: u8,
    /// The margin account used for signing actions
    pub margin_account: Pubkey,
    /// The `Market` for the market
    pub market: Pubkey,
    /// Token account used by the margin program to track the debt
    pub claims: Pubkey,
    /// Token account used by the margin program to track the collateral value of positions
    /// which are internal to fixed-term market, such as SplitTicket, ClaimTicket, and open orders.
    /// this does *not* represent underlying tokens or ticket tokens, those are registered independently in margin
    pub ticket_collateral: Pubkey,
    /// Token account used by the margin program to track the value of positions
    /// related to a collateralized value of a token as it rests in the control of the Fixed-Term orderbook
    /// for now this specifically tracks the tokens locked in an open borrow order
    pub token_collateral: Pubkey,
    /// The amount of debt that must be collateralized or repaid
    /// This debt is expressed in terms of the underlying token - not tickets
    debt: Debt,
    /// Accounting used to track assets in custody of the fixed term market
    assets: Assets,
    /// Settings for borrow order "auto rolling"
    pub borrow_roll_config: AutoRollConfig,
    /// Settings for lend order "auto rolling"
    pub lend_roll_config: AutoRollConfig,
}

impl MarginUser {
    /// Initialize a new [MarginUser]
    pub fn new(
        version: u8,
        margin_account: Pubkey,
        market: Pubkey,
        claims: Pubkey,
        ticket_collateral: Pubkey,
        token_collateral: Pubkey,
    ) -> Self {
        Self {
            version,
            margin_account,
            market,
            claims,
            ticket_collateral,
            token_collateral,
            borrow_roll_config: Default::default(),
            lend_roll_config: Default::default(),
            debt: Default::default(),
            assets: Default::default(),
        }
    }

    /// Account for a borrow order posted to the orderbook
    pub fn post_borrow_order(
        &mut self,
        token_value_posted: u64,
        ticket_value_posted: u64,
    ) -> Result<()> {
        self.assets
            .tokens_posted
            .try_add_assign(token_value_posted)?;
        self.debt.post_borrow_order(ticket_value_posted)
    }

    /// Account for the filled portion of a borrow order as a taker
    /// Returns the sequence number for the [TermLoan] to be created
    pub fn taker_fill_borrow_order(
        &mut self,
        ticket_value_filled: u64,
        maturation_timestamp: UnixTimestamp,
    ) -> Result<SequenceNumber> {
        self.debt
            .new_term_loan_without_posting(ticket_value_filled, maturation_timestamp)
    }

    /// Account for the filled portion of a borrow order as a taker
    /// Returns the sequence number for the [TermLoan] to be created    
    /// If there is no new [TermLoan] to be created, the `ticket_value_filled` and
    /// `maturation_timestamp` are not needed
    /// `tokens_disbursed` represents the token value after market fees have been collected,
    /// while `token_value_filled` represents the value from before fees are collected. Both values
    /// are required in order to properly account for collateral and entitled tokens
    pub fn maker_fill_borrow_order(
        &mut self,
        new_debt: bool,
        tokens_disbursed: u64,
        token_value_filled: u64,
        ticket_value_filled: u64,
        maturation_timestamp: UnixTimestamp,
    ) -> Result<SequenceNumber> {
        self.assets
            .borrow_order_fill(token_value_filled, tokens_disbursed)?;
        if new_debt {
            self.debt
                .new_term_loan_from_fill(ticket_value_filled, maturation_timestamp)
        } else {
            Ok(0)
        }
    }

    /// Account for a posted borrow order leaving the book
    pub fn cancel_borrow_order(
        &mut self,
        token_value: u64,
        ticket_value: u64,
        is_debt: bool,
    ) -> Result<()> {
        self.assets.tokens_posted.try_sub_assign(token_value)?;
        if is_debt {
            self.debt.process_out(ticket_value)?;
            self.emit_debt_balances();
        } else {
            self.assets.entitled_tickets.try_add_assign(ticket_value)?;
            self.emit_asset_balances()?;
        }
        Ok(())
    }

    /// Account for a lend order successfully posted to the orderbook
    pub fn post_lend_order(&mut self, tickets_staked: u64, tickets_posted: u64) -> Result<()> {
        if tickets_staked > 0 {
            self.assets.new_deposit(tickets_staked)?;
        }
        self.assets.tickets_posted.try_add_assign(tickets_posted)
    }

    /// Account for a lend order being filled as a maker
    pub fn maker_fill_lend_order(
        &mut self,
        auto_stake: bool,
        tickets: u64,
    ) -> Result<SequenceNumber> {
        self.assets.tickets_posted.try_sub_assign(tickets)?;

        if auto_stake {
            self.assets.new_deposit(tickets)
        } else {
            self.assets.entitled_tickets.try_add_assign(tickets)?;
            Ok(0)
        }
    }

    /// Account for a posted lend order leaving the book
    pub fn cancel_lend_order(&mut self, token_value: u64) -> Result<()> {
        self.assets.entitled_tokens.try_add_assign(token_value)?;
        self.emit_asset_balances()?;
        Ok(())
    }

    /// Redeem the underlying tokens from a matured [TermDeposit]
    pub fn redeem_deposit(
        &mut self,
        deposit_seqno: SequenceNumber,
        tickets_redeemed: u64,
    ) -> Result<()> {
        self.assets.redeem_deposit(deposit_seqno, tickets_redeemed)
    }

    /// Get the [SequenceNumber] of the next [TermLoan] in sequence
    pub fn next_term_loan(&self) -> SequenceNumber {
        self.debt.next_new_loan_seqno()
    }

    /// Get the [SequenceNumber] of the next [TermDeposit]
    pub fn next_term_deposit(&self) -> SequenceNumber {
        self.assets.next_new_deposit_seqno()
    }

    /// Get the [SequenceNumber] of the next [TermLoan] in need of repayment
    pub fn next_term_loan_to_repay(&self) -> Option<SequenceNumber> {
        self.debt.next_term_loan_to_repay()
    }

    /// Total number of unpaid [TermLoan]s
    pub fn outstanding_term_loans(&self) -> u64 {
        self.debt.outstanding_term_loans()
    }

    /// Range of active [TermLoan] sequence numbers
    pub fn active_loans(&self) -> Range<SequenceNumber> {
        self.debt.active_loans()
    }

    /// Range of active [TermDeposit] sequence numbers
    pub fn active_deposits(&self) -> Range<SequenceNumber> {
        self.assets.active_deposits()
    }

    pub fn ticket_collateral(&self) -> Result<u64> {
        self.assets.ticket_collateral()
    }

    pub fn token_collateral(&self) -> u64 {
        self.assets.token_collateral()
    }

    pub fn entitled_tickets(&self) -> u64 {
        self.assets.entitled_tickets
    }

    pub fn entitled_tokens(&self) -> u64 {
        self.assets.entitled_tokens
    }

    /// Total value of debt owed by the [MarginUser]
    pub fn total_debt(&self) -> u64 {
        self.debt.total()
    }

    /// Value of debt owed by the [MarginUser] by anticipated order fills
    pub fn pending_debt(&self) -> u64 {
        self.debt.pending
    }

    /// Value of debt owed by the [MarginUser] already filled
    pub fn committed_debt(&self) -> u64 {
        self.debt.committed
    }

    /// Have any of the unpaid [TermLoan]s reached maturity
    pub fn is_past_due(&self) -> bool {
        self.debt.is_past_due()
    }

    /// Account for a partial loan repayment
    pub fn partially_repay_loan(&mut self, loan: &TermLoan, amount: u64) -> Result<()> {
        self.debt
            .partially_repay_term_loan(loan.sequence_number, amount)
    }

    /// Full repay a [TermLoan]
    pub fn fully_repay_term_loan(
        &mut self,
        loan: &TermLoan,
        amount: u64,
        next_loan: Result<Account<TermLoan>>,
    ) -> Result<()> {
        self.debt
            .fully_repay_term_loan(loan.sequence_number, amount, next_loan)
    }

    /// Updates the internal state to account for a successful call to the `Settle` instruction
    pub fn settlement_complete(&mut self) {
        self.assets.entitled_tickets = 0;
        self.assets.entitled_tokens = 0;
    }

    /// Emits an Anchor event with the latest balances for [Assets] and [Debt].
    /// Callers should take care to invoke this function after mutating both assets
    /// and debts. When only mutating either, they can emit the individual balances.
    pub fn emit_all_balances(&self) -> Result<()> {
        emit!(AssetsUpdated::new(self)?);
        emit!(DebtUpdated::new(self));
        Ok(())
    }

    /// Emits an Anchor event with the latest balances for [Assets].
    pub fn emit_asset_balances(&self) -> Result<()> {
        emit!(AssetsUpdated::new(self)?);
        Ok(())
    }

    /// Emits an Anchor event with the latest balances for [Debt].
    pub fn emit_debt_balances(&self) {
        emit!(DebtUpdated::new(self));
    }

    #[inline]
    pub fn derive_address(&self) -> Pubkey {
        Pubkey::find_program_address(
            &[
                crate::seeds::MARGIN_USER,
                self.market.as_ref(),
                self.margin_account.as_ref(),
            ],
            &crate::id(),
        )
        .0
    }
}

#[derive(Zeroable, Debug, Default, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Debt {
    /// The sequence number for the next term loan to be created
    next_new_term_loan_seqno: u64,

    /// The sequence number of the next term loan to be paid
    next_unpaid_term_loan_seqno: u64,

    /// The maturation timestamp of the next term loan that is unpaid
    next_term_loan_maturity: UnixTimestamp,

    /// Amount that must be collateralized because there is an open order for it.
    /// Does not accrue interest because the loan has not been received yet.
    pending: u64,

    /// Debt that has already been borrowed because the order was matched.
    /// This debt will be due when the loan term ends.
    /// This includes all debt, including past due debt
    committed: u64,
}

pub type SequenceNumber = u64;

impl Debt {
    pub fn total(&self) -> u64 {
        self.pending.checked_add(self.committed).unwrap()
    }

    pub fn active_loans(&self) -> Range<SequenceNumber> {
        self.next_unpaid_term_loan_seqno..self.next_new_term_loan_seqno
    }

    pub fn next_new_loan_seqno(&self) -> SequenceNumber {
        self.next_new_term_loan_seqno
    }

    pub fn next_term_loan_to_repay(&self) -> Option<SequenceNumber> {
        if self.next_new_term_loan_seqno > self.next_unpaid_term_loan_seqno {
            Some(self.next_unpaid_term_loan_seqno)
        } else {
            None
        }
    }

    pub fn outstanding_term_loans(&self) -> u64 {
        self.next_new_term_loan_seqno - self.next_unpaid_term_loan_seqno
    }

    /// Accounting for a borrow order posted on the orderbook
    fn post_borrow_order(&mut self, ticket_value_posted: u64) -> Result<()> {
        self.pending.try_add_assign(ticket_value_posted)
    }

    /// A new term loan has been created from a taker fill
    pub fn new_term_loan_without_posting(
        &mut self,
        amount_filled_as_taker: u64,
        maturation_timestamp: UnixTimestamp,
    ) -> Result<SequenceNumber> {
        self.committed.try_add_assign(amount_filled_as_taker)?;
        if self.next_new_term_loan_seqno == self.next_unpaid_term_loan_seqno {
            self.next_term_loan_maturity = maturation_timestamp;
        }
        let seqno = self.next_new_term_loan_seqno;
        self.next_new_term_loan_seqno.try_add_assign(1)?;

        Ok(seqno)
    }

    pub fn new_term_loan_from_fill(
        &mut self,
        amount: u64,
        maturation_timestamp: UnixTimestamp,
    ) -> Result<SequenceNumber> {
        self.pending.try_sub_assign(amount)?;
        self.new_term_loan_without_posting(amount, maturation_timestamp)
    }

    pub fn process_out(&mut self, amount: u64) -> Result<()> {
        self.pending.try_sub_assign(amount)
    }

    pub fn partially_repay_term_loan(
        &mut self,
        sequence_number: SequenceNumber,
        amount_repaid: u64,
    ) -> Result<()> {
        if sequence_number != self.next_unpaid_term_loan_seqno {
            return err!(FixedTermErrorCode::TermLoanHasWrongSequenceNumber);
        }
        self.committed.try_sub_assign(amount_repaid)?;

        Ok(())
    }

    /// The term loan is fully repaid by this repayment, and the term loan account is being closed
    pub fn fully_repay_term_loan(
        &mut self,
        sequence_number: SequenceNumber,
        amount_repaid: u64,
        next_term_loan: Result<Account<TermLoan>>,
    ) -> Result<()> {
        if sequence_number != self.next_unpaid_term_loan_seqno {
            return err!(FixedTermErrorCode::TermLoanHasWrongSequenceNumber);
        }
        self.committed.try_sub_assign(amount_repaid)?;
        self.next_unpaid_term_loan_seqno.try_add_assign(1)?;

        if self.next_unpaid_term_loan_seqno < self.next_new_term_loan_seqno {
            let next_term_loan = next_term_loan?;
            require_eq!(
                next_term_loan.sequence_number,
                self.next_unpaid_term_loan_seqno,
                FixedTermErrorCode::TermLoanHasWrongSequenceNumber
            );
            self.next_term_loan_maturity = next_term_loan.maturation_timestamp;
        }

        Ok(())
    }

    pub fn is_past_due(&self) -> bool {
        self.outstanding_term_loans() > 0
            && self.next_term_loan_maturity <= Clock::get().unwrap().unix_timestamp
    }

    pub fn pending(&self) -> u64 {
        self.pending
    }

    pub fn committed(&self) -> u64 {
        self.committed
    }
}

#[derive(Zeroable, Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub struct Assets {
    /// tokens to transfer into settlement account
    pub entitled_tokens: u64,
    /// tickets to transfer into settlement account
    pub entitled_tickets: u64,

    /// The sequence number for the next deposit
    next_deposit_seqno: u64,

    /// The sequence number for the oldest deposit that has yet to be redeemed
    next_unredeemed_deposit_seqno: u64,

    /// The number of tickets locked up in ClaimTicket or SplitTicket
    tickets_staked: u64,

    /// The number of tickets that would be owned by the account should all
    /// open lend orders be filled.
    tickets_posted: u64,

    /// The number of tokens that would be owned by the account should all
    /// open borrow orders be filled.
    tokens_posted: u64,

    /// reserved data that may be used to determine the size of a user's collateral
    /// pessimistically prepared to persist aggregated values for:
    /// base and quote quantities, separately for bid/ask, on open orders and unsettled fills
    /// 2^3 = 8 u64's
    _reserved0: [u8; 64],
}

impl Assets {
    /// make sure the order has already been accounted for before calling this method
    pub fn new_deposit(&mut self, tickets: u64) -> Result<SequenceNumber> {
        let seqno = self.next_deposit_seqno;

        if tickets > 0 {
            self.next_deposit_seqno += 1;
            self.tickets_staked.try_add_assign(tickets)?;
        }

        Ok(seqno)
    }

    /// A [TermDeposit] has been redeemed
    pub fn redeem_deposit(&mut self, seqno: SequenceNumber, tickets: u64) -> Result<()> {
        if seqno != self.next_unredeemed_deposit_seqno {
            return Err(FixedTermErrorCode::TermDepositHasWrongSequenceNumber.into());
        }

        self.next_unredeemed_deposit_seqno += 1;
        self.tickets_staked = self.tickets_staked.saturating_sub(tickets);

        Ok(())
    }

    /// A posted borrow order has been successfully filled
    pub fn borrow_order_fill(&mut self, token_value_filled: u64, disbursement: u64) -> Result<()> {
        self.tokens_posted.try_sub_assign(token_value_filled)?;
        self.entitled_tokens.try_add_assign(disbursement)
    }

    /// represents the amount of collateral in staked tickets and open orders.
    /// does not reflect the entitled tickets/tokens because they are expected
    /// to be disbursed whenever this value is used.
    pub fn ticket_collateral(&self) -> Result<u64> {
        self.tickets_staked.safe_add(self.tickets_posted)
    }

    /// Represents the amount of token collateral in open borrow orders
    /// does not reflect the entitled tickets/tokens because they are expected
    /// to be disbursed whenever this value is used.
    pub fn token_collateral(&self) -> u64 {
        self.tokens_posted
    }

    pub fn next_new_deposit_seqno(&self) -> SequenceNumber {
        self.next_deposit_seqno
    }

    pub fn active_deposits(&self) -> Range<SequenceNumber> {
        self.next_unredeemed_deposit_seqno..self.next_deposit_seqno
    }
}

impl Default for Assets {
    fn default() -> Self {
        Assets {
            entitled_tokens: 0,
            entitled_tickets: 0,
            next_deposit_seqno: 0,
            next_unredeemed_deposit_seqno: 0,
            tickets_staked: 0,
            tickets_posted: 0,
            tokens_posted: 0,
            _reserved0: [0u8; 64],
        }
    }
}

#[derive(Zeroable, Default, Debug, Clone, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub struct AutoRollConfig {
    /// the limit price at which orders may be placed by an authority
    pub limit_price: u64,
}

#[account]
#[derive(Debug)]
pub struct TermLoan {
    pub sequence_number: SequenceNumber,

    /// The user borrower account this term loan is assigned to
    pub margin_user: Pubkey,

    /// The market where the term loan was created
    pub market: Pubkey,

    /// Which account recieves the rent when this PDA is destructed
    pub payer: Pubkey,

    /// The `OrderTag` associated with the creation of this `TermLoan`
    pub order_tag: OrderTag,

    /// The time that the term loan must be repaid
    pub maturation_timestamp: UnixTimestamp,

    /// The slot at which the term loan was struck
    pub strike_timestamp: UnixTimestamp,

    /// The total principal of the loan
    pub principal: u64,

    /// The total interest owed on the loan
    pub interest: u64,

    /// The remaining balance to repay
    pub balance: u64,

    /// Any boolean flags for this data type compressed to a single byte
    pub flags: TermLoanFlags,
}

impl TermLoan {
    pub fn seeds<'a>(market: &'a [u8], margin_user: &'a [u8], seq_no: &'a [u8]) -> [&'a [u8]; 4] {
        [crate::seeds::TERM_LOAN, market, margin_user, seq_no]
    }
}

impl TermLoan {
    /// The annualized interest rate for this loan
    pub fn rate(&self) -> Result<u64> {
        let tenor = self.tenor()?;
        let price = self
            .price()?
            .downcast_u64()
            .ok_or_else(|| error!(FixedTermErrorCode::FixedPointMath))?;
        Ok(PricerImpl::price_fp32_to_bps_yearly_interest(price, tenor))
    }

    /// The "price" (that is, the ratio of principal to total repayment) of this loan, expressed as
    /// a fp32 value
    pub fn price(&self) -> Result<Fp32> {
        let base = self.principal.safe_add(self.interest)?;
        Ok(Fp32::from(self.principal) / base)
    }

    /// Determines the loan tenor
    pub fn tenor(&self) -> Result<u64> {
        self.maturation_timestamp
            .safe_sub(self.strike_timestamp)
            .map(|t| t as u64)
    }
}

/// Struct for initializing and writing [TermLoan] accounts
pub struct TermLoanBuilder {
    pub market: Pubkey,
    pub margin_account: Pubkey,
    pub margin_user: Pubkey,
    pub payer: Pubkey,
    pub order_tag: OrderTag,
    pub sequence_number: u64,
    pub strike_timestamp: i64,
    pub maturation_timestamp: i64,
    pub base_filled: u64,
    pub quote_filled: u64,
    pub fees: u64,
    pub flags: TermLoanFlags,
}

impl TermLoanBuilder {
    /// Initialize a new builder from the information given by a borrow order
    pub fn new_from_order(
        accs: &MarginBorrowOrder,
        summary: &SensibleOrderSummary,
        info: &MarginCallbackInfo,
        strike_timestamp: UnixTimestamp,
        maturation_timestamp: UnixTimestamp,
        fees: u64,
        seq_no: u64,
    ) -> Result<Self> {
        Ok(Self {
            market: accs.orderbook_mut.market.key(),
            margin_account: accs.margin_account.key(),
            margin_user: accs.margin_user.key(),
            payer: accs.payer.key(),
            order_tag: info.order_tag,
            sequence_number: seq_no,
            strike_timestamp,
            maturation_timestamp,
            base_filled: summary.base_filled(),
            quote_filled: summary.quote_filled(RoundingAction::FillBorrow)?,
            flags: TermLoanFlags::default(),
            fees,
        })
    }

    pub fn init_and_write<'info>(
        self,
        loan: impl ToAccountInfo<'info>,
        payer: impl ToAccountInfo<'info>,
        system_program: impl ToAccountInfo<'info>,
    ) -> Result<()> {
        let mut term_loan = self.init(loan, payer, system_program)?;

        *term_loan = TermLoan {
            sequence_number: self.sequence_number,
            margin_user: self.margin_user,
            market: self.market,
            payer: self.payer,
            order_tag: self.order_tag,
            strike_timestamp: self.strike_timestamp,
            maturation_timestamp: self.maturation_timestamp,
            balance: self.base_filled,
            principal: self.quote_filled,
            interest: self.base_filled.safe_sub(self.quote_filled)?,
            flags: self.flags,
        };

        emit!(TermLoanCreated {
            term_loan: term_loan.key(),
            authority: self.margin_account,
            payer: self.payer,
            order_tag: self.order_tag.as_u128(),
            sequence_number: self.sequence_number,
            market: term_loan.market,
            maturation_timestamp: self.maturation_timestamp,
            quote_filled: self.quote_filled,
            base_filled: self.base_filled,
            flags: term_loan.flags,
            fees: self.fees,
        });
        Ok(())
    }

    pub fn init<'info>(
        &self,
        loan: impl ToAccountInfo<'info>,
        payer: impl ToAccountInfo<'info>,
        system_program: impl ToAccountInfo<'info>,
    ) -> Result<AnchorAccount<'info, TermLoan, Mut>> {
        serialization::init(
            loan.to_account_info(),
            payer.to_account_info(),
            system_program.to_account_info(),
            &TermLoan::seeds(
                &self.market.to_bytes(),
                &self.margin_user.to_bytes(),
                &self.sequence_number.to_le_bytes(),
            ),
        )
    }
}

bitflags! {
    #[derive(Default, AnchorSerialize, AnchorDeserialize)]
    pub struct TermLoanFlags: u8 {
        /// This term loan has already been marked as due.
        const MARKED_DUE = 0b00000001;
    }
}

#[cfg(not(feature = "mock-margin"))]
pub fn return_to_margin(user: &AccountInfo, adapter_result: &AdapterResult) -> Result<()> {
    let loader = AccountLoader::<MarginAccount>::try_from(user)?;
    let margin_account = loader.load()?;
    jet_margin::write_adapter_result(&margin_account, adapter_result)
}

#[cfg(feature = "mock-margin")]
pub fn return_to_margin(_user: &AccountInfo, _adapter_result: &AdapterResult) -> Result<()> {
    Ok(())
}
