#![allow(clippy::too_many_arguments)]

use anchor_lang::{InstructionData, ToAccountMetas};
use jet_fixed_term::{
    margin::{instructions::MarketSide, state::AutoRollConfig},
    seeds,
    tickets::instructions::StakeTicketsParams,
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};

pub use jet_fixed_term::{
    control::{
        instructions::{InitializeMarketParams, InitializeOrderbookParams},
        state::Market,
    },
    orderbook::state::{event_queue_len, orderbook_slab_len, OrderParams},
    ID,
};

use crate::{
    airspace::derive_governor_id, margin::derive_token_config, test_service::if_not_initialized,
};

pub use jet_fixed_term::ID as FIXED_TERM_PROGRAM;

#[derive(Clone, Debug)]
pub struct FixedTermIxBuilder {
    airspace: Pubkey,
    authority: Pubkey,
    market: Pubkey,
    underlying_mint: Pubkey,
    ticket_mint: Pubkey,
    underlying_token_vault: Pubkey,
    claims: Pubkey,
    ticket_collateral: Pubkey,
    orderbook_market_state: Pubkey,
    underlying_oracle: Pubkey,
    ticket_oracle: Pubkey,
    fee_destination: Pubkey,
    orderbook: OrderBookAddresses,
    payer: Pubkey,
}

#[derive(Clone, Debug)]
pub struct OrderBookAddresses {
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub event_queue: Pubkey,
}

impl FixedTermIxBuilder {
    pub fn new(
        payer: Pubkey,
        airspace: Pubkey,
        underlying_mint: Pubkey,
        market: Pubkey,
        authority: Pubkey,
        underlying_oracle: Pubkey,
        ticket_oracle: Pubkey,
        fee_destination: Option<Pubkey>,
        orderbook: OrderBookAddresses,
    ) -> Self {
        let ticket_mint =
            fixed_term_address(&[jet_fixed_term::seeds::TICKET_MINT, market.as_ref()]);
        let underlying_token_vault = fixed_term_address(&[
            jet_fixed_term::seeds::UNDERLYING_TOKEN_VAULT,
            market.as_ref(),
        ]);
        let orderbook_market_state = fixed_term_address(&[
            jet_fixed_term::seeds::ORDERBOOK_MARKET_STATE,
            market.as_ref(),
        ]);
        let claims = fixed_term_address(&[jet_fixed_term::seeds::CLAIM_NOTES, market.as_ref()]);
        let collateral = fixed_term_address(&[
            jet_fixed_term::seeds::TICKET_COLLATERAL_NOTES,
            market.as_ref(),
        ]);
        Self {
            airspace,
            authority,
            market,
            underlying_mint,
            ticket_mint,
            underlying_token_vault,
            claims,
            ticket_collateral: collateral,
            orderbook_market_state,
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
            market: fixed_term_address(&[
                seeds::MARKET,
                market.airspace.as_ref(),
                market.underlying_token_mint.as_ref(),
                &market.seed,
            ]),
            underlying_mint: market.underlying_token_mint,
            ticket_mint: market.ticket_mint,
            underlying_token_vault: market.underlying_token_vault,
            claims: market.claims_mint,
            ticket_collateral: market.ticket_collateral_mint,
            orderbook_market_state: market.orderbook_market_state,
            underlying_oracle: market.underlying_oracle,
            ticket_oracle: market.ticket_oracle,
            fee_destination: market.fee_destination,
            orderbook: OrderBookAddresses {
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
        orderbook: OrderBookAddresses,
    ) -> Self {
        Self::new(
            payer,
            *airspace,
            *mint,
            derive_market(airspace, mint, seed),
            authority,
            underlying_oracle,
            ticket_oracle,
            fee_destination,
            orderbook,
        )
    }
}

impl FixedTermIxBuilder {
    pub fn airspace(&self) -> Pubkey {
        self.airspace
    }

    pub fn token_mint(&self) -> Pubkey {
        self.underlying_mint
    }
    pub fn ticket_mint(&self) -> Pubkey {
        self.ticket_mint
    }
    pub fn market(&self) -> Pubkey {
        self.market
    }
    pub fn vault(&self) -> Pubkey {
        self.underlying_token_vault
    }
    pub fn orderbook_state(&self) -> Pubkey {
        self.orderbook_market_state
    }
    pub fn claims(&self) -> Pubkey {
        self.claims
    }
    pub fn collateral(&self) -> Pubkey {
        self.ticket_collateral
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

impl FixedTermIxBuilder {
    pub fn orderbook_mut(&self) -> jet_fixed_term::accounts::OrderbookMut {
        jet_fixed_term::accounts::OrderbookMut {
            market: self.market,
            orderbook_market_state: self.orderbook_market_state,
            event_queue: self.orderbook.event_queue,
            bids: self.orderbook.bids,
            asks: self.orderbook.asks,
        }
    }

    pub fn consume_events(
        &self,
        seed: &[u8],
        events: impl IntoIterator<Item = impl Into<Vec<Pubkey>>>,
    ) -> Instruction {
        let mut accounts = jet_fixed_term::accounts::ConsumeEvents {
            market: self.market,
            ticket_mint: self.ticket_mint,
            underlying_token_vault: self.underlying_token_vault,
            orderbook_market_state: self.orderbook_market_state,
            event_queue: self.orderbook.event_queue,
            crank_authorization: self.crank_authorization(&self.payer),
            crank: self.payer,
            payer: self.payer,
            system_program: solana_sdk::system_program::ID,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);

        let events = events.into_iter().map(Into::into).collect::<Vec<_>>();

        accounts.extend(events.iter().flat_map(|event_accounts: &Vec<Pubkey>| {
            event_accounts.iter().map(|a| AccountMeta::new(*a, false))
        }));

        let data = jet_fixed_term::instruction::ConsumeEvents {
            num_events: events.len() as u32,
            seed_bytes: seed.to_vec(),
        }
        .data();

        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    /// initializes the associated token account for the underlying mint owned
    /// by the authority of the market. this only returns an instruction if
    /// you've opted to use the default fee_destination, which is the ata for
    /// the authority. otherwise this returns nothing
    pub fn init_default_fee_destination(&self, payer: &Pubkey) -> Option<Instruction> {
        let ata = get_associated_token_address(&self.authority, &self.underlying_mint);
        if self.fee_destination == ata {
            Some(if_not_initialized(
                ata,
                create_associated_token_account(
                    payer,
                    &self.authority,
                    &self.underlying_mint,
                    &spl_token::id(),
                ),
            ))
        } else {
            None
        }
    }

    pub fn initialize_market(
        &self,
        payer: Pubkey,
        version_tag: u64,
        seed: [u8; 32],
        borrow_tenor: u64,
        lend_tenor: u64,
        origination_fee: u64,
    ) -> Instruction {
        let data = jet_fixed_term::instruction::InitializeMarket {
            params: InitializeMarketParams {
                version_tag,
                seed,
                borrow_tenor,
                lend_tenor,
                origination_fee,
            },
        }
        .data();
        let accounts = jet_fixed_term::accounts::InitializeMarket {
            market: self.market,
            underlying_token_mint: self.underlying_mint,
            underlying_token_vault: self.underlying_token_vault,
            ticket_mint: self.ticket_mint,
            claims: self.claims,
            collateral: self.ticket_collateral,
            authority: self.authority,
            airspace: self.airspace,
            underlying_oracle: self.underlying_oracle,
            ticket_oracle: self.ticket_oracle,
            fee_destination: self.fee_destination,
            payer,
            rent: solana_sdk::sysvar::rent::ID,
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);
        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    pub fn initialize_orderbook_slab(
        &self,
        slab: &Pubkey,
        capacity: usize,
        rent: u64,
    ) -> Instruction {
        solana_sdk::system_instruction::create_account(
            &self.payer,
            slab,
            rent,
            orderbook_slab_len(capacity) as u64,
            &jet_fixed_term::ID,
        )
    }

    pub fn initialize_event_queue(
        &self,
        queue: &Pubkey,
        capacity: usize,
        rent: u64,
    ) -> Instruction {
        solana_sdk::system_instruction::create_account(
            &self.payer,
            queue,
            rent,
            event_queue_len(capacity) as u64,
            &jet_fixed_term::ID,
        )
    }

    pub fn initialize_orderbook(&self, payer: Pubkey, min_base_order_size: u64) -> Instruction {
        let data = jet_fixed_term::instruction::InitializeOrderbook {
            params: InitializeOrderbookParams {
                min_base_order_size,
            },
        }
        .data();
        let accounts = jet_fixed_term::accounts::InitializeOrderbook {
            market: self.market,
            orderbook_market_state: self.orderbook_market_state,
            event_queue: self.orderbook.event_queue,
            bids: self.orderbook.bids,
            asks: self.orderbook.asks,
            authority: self.authority,
            airspace: self.airspace,
            payer,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);
        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    pub fn initialize_margin_user(&self, owner: Pubkey) -> Instruction {
        let margin_user = self.margin_user_account(owner);
        let accounts = jet_fixed_term::accounts::InitializeMarginUser {
            market: self.market,
            payer: self.payer,
            margin_user,
            margin_account: owner,
            claims: FixedTermIxBuilder::user_claims(margin_user),
            ticket_collateral: FixedTermIxBuilder::user_ticket_collateral(margin_user),
            claims_mint: self.claims,
            ticket_collateral_mint: self.ticket_collateral,
            rent: solana_sdk::sysvar::rent::ID,
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
            claims_metadata: derive_token_config(&self.airspace, &self.claims),
            ticket_collateral_metadata: derive_token_config(
                &self.airspace,
                &self.ticket_collateral,
            ),
        }
        .to_account_metas(None);
        Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &jet_fixed_term::instruction::InitializeMarginUser {}.data(),
            accounts,
        )
    }

    /// can derive keys from `owner`
    /// else needs vault addresses
    pub fn convert_tokens(
        &self,
        owner: Pubkey,
        token_vault: Option<Pubkey>,
        ticket_vault: Option<Pubkey>,
        amount: u64,
    ) -> Instruction {
        let user_ticket_vault = match ticket_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&owner, &self.ticket_mint),
        };
        let user_underlying_token_vault = match token_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&owner, &self.underlying_mint),
        };

        let data = jet_fixed_term::instruction::ExchangeTokens { amount }.data();
        let accounts = jet_fixed_term::accounts::ExchangeTokens {
            market: self.market,
            underlying_token_vault: self.underlying_token_vault,
            ticket_mint: self.ticket_mint,
            user_ticket_vault,
            user_underlying_token_vault,
            user_authority: owner,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);
        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    pub fn stake_tickets(
        &self,
        ticket_holder: Pubkey,
        ticket_vault: Option<Pubkey>,
        amount: u64,
        seed: &[u8],
    ) -> Instruction {
        let deposit = self.term_deposit_key(&ticket_holder, seed);

        let ticket_token_account = match ticket_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&ticket_holder, &self.ticket_mint),
        };
        let data = jet_fixed_term::instruction::StakeTickets {
            params: StakeTicketsParams {
                amount,
                seed: seed.to_vec(),
            },
        }
        .data();
        let accounts = jet_fixed_term::accounts::StakeTickets {
            deposit,
            market: self.market,
            ticket_holder,
            ticket_token_account,
            ticket_mint: self.ticket_mint,
            payer: self.payer,
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);
        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    pub fn redeem_ticket(
        &self,
        ticket_holder: Pubkey,
        ticket: Pubkey,
        token_vault: Option<Pubkey>,
    ) -> Instruction {
        let data = jet_fixed_term::instruction::RedeemDeposit {}.data();
        let accounts = self
            .redeem_deposit_accounts(ticket_holder, ticket_holder, ticket, token_vault)
            .to_account_metas(None);
        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    pub fn settle(&self, margin_account: Pubkey) -> Instruction {
        let user = self.margin_user(margin_account);
        let accounts = jet_fixed_term::accounts::Settle {
            market: self.market,
            ticket_mint: self.ticket_mint,
            token_program: spl_token::ID,
            margin_user: user.address,
            margin_account,
            claims: user.claims,
            claims_mint: self.claims,
            ticket_collateral: user.ticket_collateral,
            ticket_collateral_mint: self.ticket_collateral,
            underlying_token_vault: self.underlying_token_vault,
            underlying_settlement: get_associated_token_address(
                &margin_account,
                &self.underlying_mint,
            ),
            ticket_settlement: get_associated_token_address(&margin_account, &self.ticket_mint),
        };
        Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &jet_fixed_term::instruction::Settle {}.data(),
            accounts.to_account_metas(None),
        )
    }

    pub fn margin_redeem_deposit(
        &self,
        margin_account: Pubkey,
        ticket: Pubkey,
        token_vault: Option<Pubkey>,
    ) -> Instruction {
        let margin_user = self.margin_user(margin_account);
        let data = jet_fixed_term::instruction::MarginRedeemDeposit {}.data();
        let accounts = jet_fixed_term::accounts::MarginRedeemDeposit {
            margin_user: margin_user.address,
            ticket_collateral: margin_user.ticket_collateral,
            ticket_collateral_mint: self.ticket_collateral,
            inner: self.redeem_deposit_accounts(
                margin_user.address,
                margin_account,
                ticket,
                token_vault,
            ),
        }
        .to_account_metas(None);
        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    pub fn redeem_deposit_accounts(
        &self,
        owner: Pubkey,
        authority: Pubkey,
        deposit: Pubkey,
        token_vault: Option<Pubkey>,
    ) -> jet_fixed_term::accounts::RedeemDeposit {
        let token_account = match token_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&owner, &self.underlying_mint),
        };

        jet_fixed_term::accounts::RedeemDeposit {
            deposit,
            owner,
            authority,
            token_account,
            payer: self.payer,
            market: self.market,
            underlying_token_vault: self.underlying_token_vault,
            token_program: spl_token::ID,
        }
    }

    pub fn refresh_position(&self, margin_account: Pubkey, expect_price: bool) -> Instruction {
        Instruction {
            program_id: jet_fixed_term::ID,
            accounts: jet_fixed_term::accounts::RefreshPosition {
                market: self.market,
                margin_user: fixed_term_address(&[
                    seeds::MARGIN_USER,
                    self.market.as_ref(),
                    margin_account.as_ref(),
                ]),
                margin_account,
                underlying_oracle: self.underlying_oracle,
                ticket_oracle: self.ticket_oracle,
                token_program: spl_token::ID,
            }
            .to_account_metas(None),
            data: jet_fixed_term::instruction::RefreshPosition { expect_price }.data(),
        }
    }

    pub fn sell_tickets_order(
        &self,
        user: Pubkey,
        ticket_vault: Option<Pubkey>,
        token_vault: Option<Pubkey>,
        params: OrderParams,
    ) -> Instruction {
        let data = jet_fixed_term::instruction::SellTicketsOrder { params }.data();
        let accounts = self
            .sell_tickets_order_accounts(user, ticket_vault, token_vault)
            .to_account_metas(None);
        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    pub fn margin_sell_tickets_order(
        &self,
        margin_account: Pubkey,
        ticket_vault: Option<Pubkey>,
        token_vault: Option<Pubkey>,
        params: OrderParams,
    ) -> Instruction {
        let margin_user = self.margin_user(margin_account);
        let data = jet_fixed_term::instruction::MarginSellTicketsOrder { params }.data();
        let accounts = jet_fixed_term::accounts::MarginSellTicketsOrder {
            margin_user: margin_user.address,
            ticket_collateral: margin_user.ticket_collateral,
            ticket_collateral_mint: self.ticket_collateral,
            inner: self.sell_tickets_order_accounts(margin_account, ticket_vault, token_vault),
        }
        .to_account_metas(None);
        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    fn sell_tickets_order_accounts(
        &self,
        authority: Pubkey,
        ticket_vault: Option<Pubkey>,
        token_vault: Option<Pubkey>,
    ) -> jet_fixed_term::accounts::SellTicketsOrder {
        let user_ticket_vault = match ticket_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&authority, &self.ticket_mint),
        };
        let user_token_vault = match token_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&authority, &self.underlying_mint),
        };
        jet_fixed_term::accounts::SellTicketsOrder {
            authority,
            user_ticket_vault,
            user_token_vault,
            ticket_mint: self.ticket_mint,
            underlying_token_vault: self.underlying_token_vault,
            orderbook_mut: self.orderbook_mut(),
            token_program: spl_token::ID,
        }
    }

    pub fn margin_borrow_order(
        &self,
        margin_account: Pubkey,
        underlying_settlement: Option<Pubkey>,
        params: OrderParams,
        debt_seqno: u64,
    ) -> Instruction {
        let margin_user = self.margin_user(margin_account);

        let data = jet_fixed_term::instruction::MarginBorrowOrder { params }.data();
        let accounts = jet_fixed_term::accounts::MarginBorrowOrder {
            orderbook_mut: self.orderbook_mut(),
            margin_user: margin_user.address,
            margin_account,
            claims: margin_user.claims,
            term_loan: self.term_loan_key(&margin_user.address, &debt_seqno.to_le_bytes()),
            claims_mint: self.claims,
            ticket_collateral: margin_user.ticket_collateral,
            ticket_collateral_mint: self.ticket_collateral,
            underlying_token_vault: self.underlying_token_vault,
            underlying_settlement: underlying_settlement.unwrap_or_else(|| {
                get_associated_token_address(&margin_account, &self.underlying_mint)
            }),
            payer: self.payer,
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);

        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    pub fn lend_order(
        &self,
        user: Pubkey,
        lender_tickets: Option<Pubkey>,
        lender_tokens: Option<Pubkey>,
        params: OrderParams,
        seed: &[u8],
    ) -> Instruction {
        let data = jet_fixed_term::instruction::LendOrder {
            params,
            seed: seed.to_vec(),
        }
        .data();
        let accounts = self
            .lend_order_accounts(user, user, lender_tickets, lender_tokens, params, seed)
            .to_account_metas(None);
        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    pub fn margin_lend_order(
        &self,
        margin_account: Pubkey,
        lender_tokens: Option<Pubkey>,
        params: OrderParams,
        deposit_seqno: u64,
    ) -> Instruction {
        let margin_user = self.margin_user(margin_account);
        let data = jet_fixed_term::instruction::MarginLendOrder { params }.data();
        let accounts = jet_fixed_term::accounts::MarginLendOrder {
            margin_user: margin_user.address,
            ticket_collateral: margin_user.ticket_collateral,
            ticket_collateral_mint: self.ticket_collateral,
            inner: self.lend_order_accounts(
                margin_user.address,
                margin_account,
                None,
                lender_tokens,
                params,
                &deposit_seqno.to_le_bytes(),
            ),
        }
        .to_account_metas(None);
        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    fn lend_order_accounts(
        &self,
        user: Pubkey,
        authority: Pubkey,
        lender_tickets: Option<Pubkey>,
        lender_tokens: Option<Pubkey>,
        params: OrderParams,
        seed: &[u8],
    ) -> jet_fixed_term::accounts::LendOrder {
        let lender_tickets = match lender_tickets {
            Some(vault) => vault,
            None => get_associated_token_address(&authority, &self.ticket_mint),
        };
        let lender_tokens = match lender_tokens {
            Some(vault) => vault,
            None => get_associated_token_address(&authority, &self.underlying_mint),
        };
        let deposit = self.term_deposit_key(&user, seed);
        jet_fixed_term::accounts::LendOrder {
            authority,
            ticket_settlement: if params.auto_stake {
                deposit
            } else {
                lender_tickets
            },
            lender_tokens,
            underlying_token_vault: self.underlying_token_vault,
            ticket_mint: self.ticket_mint,
            payer: self.payer,
            orderbook_mut: self.orderbook_mut(),
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
        }
    }

    pub fn cancel_order(&self, owner: Pubkey, order_id: u128) -> Instruction {
        let data = jet_fixed_term::instruction::CancelOrder { order_id }.data();
        let accounts = jet_fixed_term::accounts::CancelOrder {
            owner,
            orderbook_mut: self.orderbook_mut(),
        }
        .to_account_metas(None);

        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    pub fn pause_order_matching(&self) -> Instruction {
        let data = jet_fixed_term::instruction::PauseOrderMatching {}.data();
        let accounts = jet_fixed_term::accounts::PauseOrderMatching {
            market: self.market,
            orderbook_market_state: self.orderbook_market_state,
            authority: self.authority,
            airspace: self.airspace,
        }
        .to_account_metas(None);

        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    pub fn resume_order_matching(&self) -> Instruction {
        let data = jet_fixed_term::instruction::ResumeOrderMatching {}.data();
        let accounts = jet_fixed_term::accounts::ResumeOrderMatching {
            market: self.market,
            orderbook_market_state: self.orderbook_market_state,
            event_queue: self.orderbook.event_queue,
            bids: self.orderbook.bids,
            asks: self.orderbook.asks,
            authority: self.authority,
            airspace: self.airspace,
        }
        .to_account_metas(None);

        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    pub fn pause_ticket_redemption(&self) -> Instruction {
        self.modify_market([true as u8].into(), 8 + 32 * 14 + 2)
    }
    pub fn resume_ticket_redemption(&self) -> Instruction {
        self.modify_market([false as u8].into(), 8 + 32 * 14 + 2)
    }

    pub fn modify_market(&self, data: Vec<u8>, offset: u32) -> Instruction {
        let data = jet_fixed_term::instruction::ModifyMarket { data, offset }.data();
        let accounts = jet_fixed_term::accounts::ModifyMarket {
            market: self.market,
            authority: self.authority,
            airspace: self.airspace,
        }
        .to_account_metas(None);
        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    pub fn authorize_crank(&self, crank: Pubkey) -> Instruction {
        let data = jet_fixed_term::instruction::AuthorizeCrank {}.data();
        let accounts = jet_fixed_term::accounts::AuthorizeCrank {
            crank,
            market: self.market,
            crank_authorization: self.crank_authorization(&crank),
            authority: self.authority,
            airspace: self.airspace,
            payer: self.payer,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);
        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    pub fn margin_repay(
        &self,
        source_authority: &Pubkey,
        payer: &Pubkey,
        margin_account: &Pubkey,
        source_account: &Pubkey,
        term_loan_seqno: u64,
        amount: u64,
    ) -> Instruction {
        let margin_user = self.margin_user(*margin_account);
        let data = jet_fixed_term::instruction::Repay { amount }.data();
        let accounts = jet_fixed_term::accounts::Repay {
            margin_user: margin_user.address,
            term_loan: derive_term_loan(&self.market, &margin_user.address, term_loan_seqno),
            next_term_loan: derive_term_loan(
                &self.market,
                &margin_user.address,
                term_loan_seqno + 1,
            ),
            source: *source_account,
            payer: *payer,
            source_authority: *source_authority,
            underlying_token_vault: self.underlying_token_vault,
            token_program: spl_token::ID,
            claims: margin_user.claims,
            claims_mint: self.claims,
            market: self.market,
        }
        .to_account_metas(None);

        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    pub fn configure_auto_roll(
        &self,
        margin_account: Pubkey,
        side: MarketSide,
        config: AutoRollConfig,
    ) -> Instruction {
        let data = jet_fixed_term::instruction::ConfigureAutoRoll {
            side: side as u8,
            config,
        }
        .data();
        let accounts = jet_fixed_term::accounts::ConfigureAutoRoll {
            margin_user: self.margin_user(margin_account).address,
            margin_account,
        }
        .to_account_metas(None);

        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }
}

pub enum FixedTermPosition {
    Liability,
    UnderlyingCollateral,
    TicketCollateral,
}

/// helpful addresses for a MarginUser account
pub struct MarginUser {
    pub address: Pubkey,
    pub claims: Pubkey,
    pub ticket_collateral: Pubkey,
}

impl FixedTermIxBuilder {
    pub fn margin_user(&self, margin_account: Pubkey) -> MarginUser {
        let address = fixed_term_address(&[
            jet_fixed_term::seeds::MARGIN_USER,
            self.market.as_ref(),
            margin_account.as_ref(),
        ]);
        MarginUser {
            address,
            ticket_collateral: fixed_term_address(&[
                jet_fixed_term::seeds::TICKET_COLLATERAL_NOTES,
                address.as_ref(),
            ]),
            claims: fixed_term_address(&[jet_fixed_term::seeds::CLAIM_NOTES, address.as_ref()]),
        }
    }

    pub fn claims_mint(market_key: &Pubkey) -> Pubkey {
        fixed_term_address(&[jet_fixed_term::seeds::CLAIM_NOTES, market_key.as_ref()])
    }

    pub fn ticket_collateral_mint(market_key: &Pubkey) -> Pubkey {
        fixed_term_address(&[
            jet_fixed_term::seeds::TICKET_COLLATERAL_NOTES,
            market_key.as_ref(),
        ])
    }

    pub fn term_deposit_key(&self, ticket_holder: &Pubkey, seed: &[u8]) -> Pubkey {
        fixed_term_address(&[
            jet_fixed_term::seeds::TERM_DEPOSIT,
            self.market.as_ref(),
            ticket_holder.as_ref(),
            seed,
        ])
    }
    pub fn term_loan_key(&self, margin_user: &Pubkey, seed: &[u8]) -> Pubkey {
        fixed_term_address(&[
            jet_fixed_term::seeds::TERM_LOAN,
            self.market.as_ref(),
            margin_user.as_ref(),
            seed,
        ])
    }

    pub fn margin_user_account(&self, owner: Pubkey) -> Pubkey {
        fixed_term_address(&[
            jet_fixed_term::seeds::MARGIN_USER,
            self.market.as_ref(),
            owner.as_ref(),
        ])
    }

    pub fn user_claims(margin_user: Pubkey) -> Pubkey {
        fixed_term_address(&[jet_fixed_term::seeds::CLAIM_NOTES, margin_user.as_ref()])
    }

    pub fn user_ticket_collateral(margin_user: Pubkey) -> Pubkey {
        fixed_term_address(&[
            jet_fixed_term::seeds::TICKET_COLLATERAL_NOTES,
            margin_user.as_ref(),
        ])
    }
    pub fn crank_authorization(&self, crank: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[
                jet_fixed_term::seeds::CRANK_AUTHORIZATION,
                self.market.as_ref(),
                crank.as_ref(),
            ],
            &jet_fixed_term::ID,
        )
        .0
    }

    pub fn jet_fixed_term_id() -> Pubkey {
        jet_fixed_term::ID
    }
}

pub fn recover_uninitialized(
    governor: Pubkey,
    uninitialized: Pubkey,
    recipient: Pubkey,
) -> Instruction {
    let data = jet_fixed_term::instruction::RecoverUninitialized {}.data();
    let accounts = jet_fixed_term::accounts::RecoverUninitialized {
        governor,
        governor_id: derive_governor_id(),
        uninitialized,
        recipient,
    }
    .to_account_metas(None);

    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}

pub fn derive_market(airspace: &Pubkey, mint: &Pubkey, seed: [u8; 32]) -> Pubkey {
    fixed_term_address(&[
        jet_fixed_term::seeds::MARKET,
        airspace.as_ref(),
        mint.as_ref(),
        &seed,
    ])
}

pub fn derive_market_from_tenor(airspace: &Pubkey, token_mint: &Pubkey, tenor: u64) -> Pubkey {
    let mut seed = [0u8; 32];
    seed[..8].copy_from_slice(&tenor.to_le_bytes());

    derive_market(airspace, token_mint, seed)
}

pub fn derive_margin_user(market: &Pubkey, margin_account: &Pubkey) -> Pubkey {
    fixed_term_address(&[
        jet_fixed_term::seeds::MARGIN_USER,
        market.as_ref(),
        margin_account.as_ref(),
    ])
}

pub fn derive_term_loan(market: &Pubkey, margin_user: &Pubkey, debt_seqno: u64) -> Pubkey {
    fixed_term_address(&[
        jet_fixed_term::seeds::TERM_LOAN,
        market.as_ref(),
        margin_user.as_ref(),
        &debt_seqno.to_le_bytes(),
    ])
}

pub fn derive_term_deposit(market: &Pubkey, owner: &Pubkey, deposit_seqno: u64) -> Pubkey {
    fixed_term_address(&[
        jet_fixed_term::seeds::TERM_DEPOSIT,
        market.as_ref(),
        owner.as_ref(),
        &deposit_seqno.to_le_bytes(),
    ])
}

pub fn derive_crank_authorization(market: &Pubkey, crank: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            jet_fixed_term::seeds::CRANK_AUTHORIZATION,
            market.as_ref(),
            crank.as_ref(),
        ],
        &jet_fixed_term::ID,
    )
    .0
}

pub fn fixed_term_address(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &jet_fixed_term::ID).0
}
