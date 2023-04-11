#![allow(clippy::too_many_arguments)]

use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address;

use jet_fixed_term::{
    control::{instructions::InitializeMarketParams, state::Market},
    margin::state::AutoRollConfig,
    orderbook::state::OrderParams,
};

use super::{derive, ix, MarginUser, MarketAdmin, OrderbookAddresses};

#[derive(Clone, Debug)]
pub struct FixedTermIxBuilder {
    airspace: Pubkey,
    authority: Pubkey,
    market: Pubkey,
    underlying_mint: Pubkey,
    underlying_oracle: Pubkey,
    ticket_oracle: Pubkey,
    fee_destination: Pubkey,
    orderbook: OrderbookAddresses,
    payer: Pubkey,
}

/// Constructors
impl FixedTermIxBuilder {
    pub fn new(
        payer: Pubkey,
        airspace: Pubkey,
        underlying_mint: Pubkey,
        market: Pubkey,
        authority: Pubkey,
        underlying_oracle: Pubkey,
        ticket_oracle: Pubkey,
        fee_destination: Option<Pubkey>, // omit to use authority's ATA
        orderbook: OrderbookAddresses,
    ) -> Self {
        Self {
            airspace,
            authority,
            market,
            underlying_mint,
            underlying_oracle,
            ticket_oracle,
            fee_destination: fee_destination
                .unwrap_or_else(|| get_associated_token_address(&authority, &underlying_mint)),
            payer,
            orderbook,
        }
    }

    pub fn new_from_state(payer: Pubkey, market: &Market) -> Self {
        FixedTermIxBuilder {
            airspace: market.airspace,
            authority: Pubkey::default(), //todo
            market: derive::market(&market.airspace, &market.underlying_token_mint, market.seed),
            underlying_mint: market.underlying_token_mint,
            underlying_oracle: market.underlying_oracle,
            ticket_oracle: market.ticket_oracle,
            fee_destination: market.fee_destination,
            orderbook: OrderbookAddresses {
                bids: market.bids,
                asks: market.asks,
                event_queue: market.event_queue,
            },
            payer,
        }
    }

    /// derives the market key from a mint and seed
    pub fn new_from_seed(
        payer: Pubkey,
        airspace: &Pubkey,
        mint: &Pubkey,
        seed: [u8; 32],
        authority: Pubkey,
        underlying_oracle: Pubkey,
        ticket_oracle: Pubkey,
        fee_destination: Option<Pubkey>,
        orderbook: OrderbookAddresses,
    ) -> Self {
        Self::new(
            payer,
            *airspace,
            *mint,
            derive::market(airspace, mint, seed),
            authority,
            underlying_oracle,
            ticket_oracle,
            fee_destination,
            orderbook,
        )
    }
}

/// Getters
impl FixedTermIxBuilder {
    pub fn airspace(&self) -> Pubkey {
        self.airspace
    }
    pub fn token_mint(&self) -> Pubkey {
        self.underlying_mint
    }
    pub fn ticket_mint(&self) -> Pubkey {
        derive::ticket_mint(&self.market)
    }
    pub fn market(&self) -> Pubkey {
        self.market
    }
    pub fn vault(&self) -> Pubkey {
        derive::underlying_token_vault(&self.market)
    }
    pub fn orderbook_state(&self) -> Pubkey {
        derive::orderbook_market_state(&self.market)
    }
    pub fn claims(&self) -> Pubkey {
        derive::claims_mint(&self.market)
    }
    pub fn collateral(&self) -> Pubkey {
        derive::ticket_collateral_mint(&self.market)
    }
    pub fn event_queue(&self) -> Pubkey {
        self.orderbook.event_queue
    }
    pub fn bids(&self) -> Pubkey {
        self.orderbook.bids
    }
    pub fn asks(&self) -> Pubkey {
        self.orderbook.asks
    }
}

/// Instructions - simply delegate to `ix` module
impl FixedTermIxBuilder {
    pub fn orderbook_mut(&self) -> jet_fixed_term::accounts::OrderbookMut {
        jet_fixed_term::accounts::OrderbookMut {
            market: self.market,
            orderbook_market_state: derive::orderbook_market_state(&self.market),
            event_queue: self.orderbook.event_queue,
            bids: self.orderbook.bids,
            asks: self.orderbook.asks,
        }
    }

    pub fn market_admin(&self) -> MarketAdmin {
        MarketAdmin {
            market: self.market,
            authority: self.authority,
            airspace: self.airspace,
        }
    }

    pub fn consume_events(
        &self,
        seed: &[u8],
        events: impl IntoIterator<Item = impl Into<Vec<Pubkey>>>,
    ) -> Instruction {
        ix::consume_events(
            seed,
            events,
            self.market,
            self.orderbook.event_queue,
            self.payer,
            self.payer,
        )
    }

    /// initializes the associated token account for the underlying mint owned
    /// by the authority of the market. this only returns an instruction if
    /// you've opted to use the default fee_destination, which is the ata for
    /// the authority. otherwise this returns nothing
    pub fn init_default_fee_destination(&self, payer: &Pubkey) -> Option<Instruction> {
        ix::init_default_fee_destination(
            &self.fee_destination,
            &self.authority,
            &self.underlying_mint,
            payer,
        )
    }

    pub fn initialize_market(&self, payer: Pubkey, params: InitializeMarketParams) -> Instruction {
        ix::initialize_market(
            params,
            self.underlying_mint,
            self.airspace,
            self.authority,
            self.underlying_oracle,
            self.ticket_oracle,
            self.fee_destination,
            payer,
        )
    }

    pub fn initialize_orderbook_slab(
        &self,
        slab: &Pubkey,
        capacity: usize,
        rent: u64,
    ) -> Instruction {
        ix::init_orderbook_slab(slab, capacity, rent, &self.payer)
    }

    pub fn initialize_event_queue(
        &self,
        queue: &Pubkey,
        capacity: usize,
        rent: u64,
    ) -> Instruction {
        ix::init_event_queue(queue, capacity, rent, &self.payer)
    }

    pub fn initialize_orderbook(&self, payer: Pubkey, min_base_order_size: u64) -> Instruction {
        ix::initialize_orderbook(
            min_base_order_size,
            self.orderbook,
            self.market_admin(),
            payer,
        )
    }

    pub fn initialize_margin_user(&self, owner: Pubkey) -> Instruction {
        ix::initialize_margin_user(owner, self.market, self.airspace, self.payer)
    }

    /// can derive keys from `owner`. else needs vault addresses
    pub fn convert_tokens(
        &self,
        owner: Pubkey,
        underlying_token_source: Option<Pubkey>,
        ticket_destination: Option<Pubkey>,
        amount: u64,
    ) -> Instruction {
        ix::convert_tokens(
            amount,
            self.market,
            owner,
            underlying_token_source,
            ticket_destination,
            self.underlying_mint,
        )
    }

    pub fn stake_tickets(
        &self,
        ticket_holder: Pubkey,
        ticket_source: Option<Pubkey>,
        amount: u64,
        seed: &[u8],
    ) -> Instruction {
        ix::stake_tickets(
            amount,
            seed,
            self.market,
            ticket_holder,
            ticket_source,
            self.payer,
        )
    }

    pub fn redeem_deposit(
        &self,
        ticket_holder: Pubkey,
        deposit: Pubkey,
        token_destination: Option<Pubkey>,
    ) -> Instruction {
        ix::redeem_deposit(ix::redeem_deposit_accounts(
            self.market,
            ticket_holder,
            self.underlying_mint,
            deposit,
            token_destination,
            self.payer,
        ))
    }

    pub fn settle(&self, margin_account: Pubkey) -> Instruction {
        ix::settle(self.market, self.underlying_mint, margin_account)
    }

    pub fn margin_redeem_deposit(
        &self,
        margin_account: Pubkey,
        deposit: Pubkey,
        token_destination: Option<Pubkey>,
    ) -> Instruction {
        let margin_user = derive::margin_user(&self.market, &margin_account);
        ix::margin_redeem_deposit(
            &self.market,
            margin_user,
            ix::redeem_deposit_accounts(
                self.market,
                margin_account,
                self.underlying_mint,
                deposit,
                token_destination,
                self.payer,
            ),
        )
    }

    pub fn refresh_position(&self, margin_account: Pubkey, expect_price: bool) -> Instruction {
        ix::refresh_position(
            expect_price,
            self.market,
            margin_account,
            self.underlying_oracle,
            self.ticket_oracle,
        )
    }

    pub fn sell_tickets_order(
        &self,
        user: Pubkey,
        ticket_source: Option<Pubkey>,
        token_destination: Option<Pubkey>,
        params: OrderParams,
    ) -> Instruction {
        ix::sell_tickets_order(
            params,
            ix::sell_tickets_order_accounts(
                self.orderbook_mut(),
                user,
                &self.underlying_mint,
                ticket_source,
                token_destination,
            ),
        )
    }

    pub fn margin_sell_tickets_order(
        &self,
        margin_account: Pubkey,
        ticket_source: Option<Pubkey>,
        token_destination: Option<Pubkey>,
        params: OrderParams,
    ) -> Instruction {
        let margin_user = derive::margin_user(&self.market, &margin_account);
        ix::margin_sell_tickets_order(
            params,
            margin_user,
            ix::sell_tickets_order_accounts(
                self.orderbook_mut(),
                margin_account,
                &self.underlying_mint,
                ticket_source,
                token_destination,
            ),
        )
    }

    pub fn margin_borrow_order(
        &self,
        margin_account: Pubkey,
        params: OrderParams,
        debt_seqno: u64,
    ) -> Instruction {
        ix::margin_borrow_order(
            params,
            debt_seqno,
            self.orderbook_mut(),
            margin_account,
            &self.underlying_mint,
            self.payer,
        )
    }

    pub fn lend_order(
        &self,
        user: Pubkey,
        lender_tickets: Option<Pubkey>,
        lender_tokens: Option<Pubkey>,
        params: OrderParams,
        seed: &[u8],
    ) -> Instruction {
        ix::lend_order(
            params,
            seed,
            &self.market,
            user,
            lender_tickets,
            lender_tokens,
            self.orderbook_mut(),
            self.underlying_mint,
            self.payer,
        )
    }

    pub fn margin_lend_order(
        &self,
        margin_account: Pubkey,
        lender_tokens: Option<Pubkey>,
        params: OrderParams,
        deposit_seqno: u64,
    ) -> Instruction {
        ix::margin_lend_order(
            params,
            deposit_seqno,
            self.market,
            margin_account,
            lender_tokens,
            self.orderbook_mut(),
            self.underlying_mint,
            self.payer,
        )
    }

    pub fn auto_roll_lend_order(
        &self,
        margin_account: Pubkey,
        deposit: Pubkey,
        rent_receiver: Pubkey,
        deposit_seqno: u64,
    ) -> Instruction {
        ix::auto_roll_lend_order(
            deposit_seqno,
            margin_account,
            deposit,
            rent_receiver,
            self.orderbook_mut(),
            self.payer,
        )
    }

    pub fn auto_roll_borrow_order(
        &self,
        margin_account: Pubkey,
        loan: Pubkey,
        rent_receiver: Pubkey,
        next_debt_seqno: u64,
    ) -> Instruction {
        ix::auto_roll_borrow_order(
            next_debt_seqno,
            margin_account,
            loan,
            rent_receiver,
            self.orderbook_mut(),
            self.payer,
        )
    }

    pub fn cancel_order(&self, owner: Pubkey, order_id: u128) -> Instruction {
        ix::cancel_order(order_id, owner, self.orderbook_mut())
    }

    pub fn pause_order_matching(&self) -> Instruction {
        ix::pause_order_matching(
            self.market_admin(),
            derive::orderbook_market_state(&self.market),
        )
    }

    pub fn resume_order_matching(&self) -> Instruction {
        ix::resume_order_matching(self.market_admin(), self.orderbook)
    }

    pub fn pause_ticket_redemption(&self) -> Instruction {
        ix::pause_ticket_redemption(self.market_admin())
    }
    pub fn resume_ticket_redemption(&self) -> Instruction {
        ix::resume_ticket_redemption(self.market_admin())
    }

    pub fn modify_market(&self, data: Vec<u8>, offset: u32) -> Instruction {
        ix::modify_market(data, offset, self.market_admin())
    }

    pub fn authorize_crank(&self, crank: Pubkey) -> Instruction {
        ix::authorize_crank(crank, self.market_admin(), self.payer)
    }

    pub fn margin_repay(
        &self,
        source_authority: &Pubkey,
        payer: &Pubkey,
        margin_account: &Pubkey,
        source: &Pubkey,
        term_loan_seqno: u64,
        amount: u64,
    ) -> Instruction {
        ix::margin_repay(
            term_loan_seqno,
            amount,
            self.market,
            *source_authority,
            *payer,
            derive::margin_user(&self.market, margin_account),
            *source,
        )
    }

    pub fn configure_auto_roll(
        &self,
        margin_account: Pubkey,
        config: AutoRollConfig,
    ) -> Instruction {
        let margin_user = derive::margin_user(&self.market, &margin_account);
        ix::configure_auto_roll(self.market, margin_account, margin_user, config)
    }

    pub fn stop_auto_roll_deposit(&self, margin_account: Pubkey, deposit: Pubkey) -> Instruction {
        ix::stop_auto_roll_deposit(margin_account, deposit)
    }

    pub fn stop_auto_roll_loan(&self, margin_account: Pubkey, loan: Pubkey) -> Instruction {
        let margin_user = derive::margin_user(&self.market, &margin_account);
        ix::stop_auto_roll_loan(margin_account, margin_user, loan)
    }
}

/// Derived addresses
impl FixedTermIxBuilder {
    pub fn margin_user(&self, margin_account: Pubkey) -> MarginUser {
        let address = derive::margin_user(&self.market, &margin_account);
        MarginUser {
            address,
            ticket_collateral: derive::user_ticket_collateral(&address),
            underlying_collateral: derive::user_underlying_collateral(&address),
            claims: derive::user_claims(&address),
        }
    }

    pub fn term_deposit_key(&self, ticket_holder: &Pubkey, seed: &[u8]) -> Pubkey {
        derive::term_deposit_bytes(&self.market, ticket_holder, seed)
    }

    pub fn term_loan_key(&self, margin_user: &Pubkey, seed: &[u8]) -> Pubkey {
        derive::term_loan_bytes(&self.market, margin_user, seed)
    }

    pub fn margin_user_account(&self, owner: Pubkey) -> Pubkey {
        derive::margin_user(&self.market, &owner)
    }

    pub fn crank_authorization(&self, crank: &Pubkey) -> Pubkey {
        derive::crank_authorization(&self.market, crank)
    }
}
