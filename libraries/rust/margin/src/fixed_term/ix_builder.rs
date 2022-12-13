use std::sync::Arc;

use agnostic_orderbook::state::{event_queue::EventQueue, AccountTag};
use anchor_lang::{InstructionData, ToAccountMetas};
use jet_fixed_term::{
    margin::state::TermLoan, orderbook::state::CallbackInfo, seeds,
    tickets::instructions::StakeTicketsParams,
};
use jet_simulation::solana_rpc_api::SolanaRpcClient;
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

use crate::ix_builder::{get_metadata_address, test_service::if_not_initialized};

use super::{
    error::{client_err, FixedTermMarketIxError, Result},
    event_builder::{ConsumeEventsInfo, ConsumeEventsParams},
};

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
    orderbook: Option<OrderBookAddresses>,
    payer: Option<Pubkey>,
    crank: Option<Pubkey>,
}

#[derive(Clone, Debug)]
pub struct OrderBookAddresses {
    bids: Pubkey,
    asks: Pubkey,
    event_queue: Pubkey,
}

trait UnwrapKey {
    fn unwrap_key(&self, msg: &str) -> Result<Pubkey>;
}

impl UnwrapKey for Option<Pubkey> {
    fn unwrap_key(&self, msg: &str) -> Result<Pubkey> {
        self.ok_or(FixedTermMarketIxError::MissingPubkey(msg.into()))
    }
}

impl UnwrapKey for Option<&Pubkey> {
    fn unwrap_key(&self, msg: &str) -> Result<Pubkey> {
        Ok(*self.ok_or(FixedTermMarketIxError::MissingPubkey(msg.into()))?)
    }
}

impl From<Market> for FixedTermIxBuilder {
    fn from(market: Market) -> Self {
        FixedTermIxBuilder {
            airspace: market.airspace,
            authority: Pubkey::default(), //todo
            market: fixed_term_market_pda(&[
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
            orderbook: Some(OrderBookAddresses {
                bids: market.bids,
                asks: market.asks,
                event_queue: market.event_queue,
            }),
            payer: None,
            crank: None,
        }
    }
}

impl FixedTermIxBuilder {
    pub fn new(
        airspace: Pubkey,
        underlying_mint: Pubkey,
        market: Pubkey,
        authority: Pubkey,
        underlying_oracle: Pubkey,
        ticket_oracle: Pubkey,
        fee_destination: Option<Pubkey>,
    ) -> Self {
        let ticket_mint =
            fixed_term_market_pda(&[jet_fixed_term::seeds::TICKET_MINT, market.as_ref()]);
        let underlying_token_vault = fixed_term_market_pda(&[
            jet_fixed_term::seeds::UNDERLYING_TOKEN_VAULT,
            market.as_ref(),
        ]);
        let orderbook_market_state = fixed_term_market_pda(&[
            jet_fixed_term::seeds::ORDERBOOK_MARKET_STATE,
            market.as_ref(),
        ]);
        let claims = fixed_term_market_pda(&[jet_fixed_term::seeds::CLAIM_NOTES, market.as_ref()]);
        let collateral = fixed_term_market_pda(&[
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
            orderbook: None,
            payer: None,
            crank: None,
        }
    }

    /// derives the market key from a mint and seed
    pub fn new_from_seed(
        airspace: &Pubkey,
        mint: &Pubkey,
        seed: [u8; 32],
        authority: Pubkey,
        underlying_oracle: Pubkey,
        ticket_oracle: Pubkey,
        fee_destination: Option<Pubkey>,
    ) -> Self {
        Self::new(
            *airspace,
            *mint,
            Self::market_key(airspace, mint, seed),
            authority,
            underlying_oracle,
            ticket_oracle,
            fee_destination,
        )
    }

    pub fn with_payer(mut self, payer: &Pubkey) -> Self {
        self.payer = Some(*payer);
        self
    }

    pub fn with_crank(mut self, crank: &Pubkey) -> Self {
        self.crank = Some(*crank);
        self
    }

    pub fn with_orderbook_accounts(
        mut self,
        bids: Pubkey,
        asks: Pubkey,
        event_queue: Pubkey,
    ) -> Self {
        self.orderbook = Some(OrderBookAddresses {
            bids,
            asks,
            event_queue,
        });

        self
    }
}

impl FixedTermIxBuilder {
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
        self.orderbook.as_ref().unwrap().event_queue
    }
    pub fn bids(&self) -> Pubkey {
        self.orderbook.as_ref().unwrap().bids
    }
    pub fn asks(&self) -> Pubkey {
        self.orderbook.as_ref().unwrap().asks
    }
}

impl FixedTermIxBuilder {
    pub fn orderbook_mut(&self) -> Result<jet_fixed_term::accounts::OrderbookMut> {
        Ok(jet_fixed_term::accounts::OrderbookMut {
            market: self.market,
            orderbook_market_state: self.orderbook_market_state,
            event_queue: self.orderbook.as_ref().unwrap().event_queue,
            bids: self.orderbook.as_ref().unwrap().bids,
            asks: self.orderbook.as_ref().unwrap().asks,
        })
    }

    pub fn consume_events(&self, params: &ConsumeEventsParams) -> Result<Instruction> {
        let data = jet_fixed_term::instruction::ConsumeEvents {
            num_events: params.num_events,
            seed_bytes: params.seeds.clone(),
        }
        .data();
        let mut accounts = jet_fixed_term::accounts::ConsumeEvents {
            market: self.market,
            ticket_mint: self.ticket_mint,
            underlying_token_vault: self.underlying_token_vault,
            orderbook_market_state: self.orderbook_market_state,
            event_queue: self.orderbook.as_ref().unwrap().event_queue,
            crank_authorization: self.crank_authorization()?,
            crank: self.crank.unwrap(),
            payer: self.payer.unwrap(),
            system_program: solana_sdk::system_program::ID,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);
        accounts.extend(
            params
                .account_keys
                .clone()
                .into_iter()
                .map(|k| AccountMeta::new(k, false)),
        );
        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &data,
            accounts,
        ))
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
                create_associated_token_account(payer, &self.authority, &self.underlying_mint),
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
        borrow_tenor: i64,
        lend_tenor: i64,
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
    ) -> Result<Instruction> {
        Ok(solana_sdk::system_instruction::create_account(
            &self.payer.unwrap(),
            slab,
            rent,
            orderbook_slab_len(capacity) as u64,
            &jet_fixed_term::ID,
        ))
    }

    pub fn initialize_event_queue(
        &self,
        queue: &Pubkey,
        capacity: usize,
        rent: u64,
    ) -> Result<Instruction> {
        Ok(solana_sdk::system_instruction::create_account(
            &self.payer.unwrap(),
            queue,
            rent,
            event_queue_len(capacity) as u64,
            &jet_fixed_term::ID,
        ))
    }

    pub fn initialize_orderbook(
        &self,
        payer: Pubkey,
        event_queue: Pubkey,
        bids: Pubkey,
        asks: Pubkey,
        min_base_order_size: u64,
    ) -> Result<Instruction> {
        let data = jet_fixed_term::instruction::InitializeOrderbook {
            params: InitializeOrderbookParams {
                min_base_order_size,
            },
        }
        .data();
        let accounts = jet_fixed_term::accounts::InitializeOrderbook {
            market: self.market,
            orderbook_market_state: self.orderbook_market_state,
            event_queue,
            bids,
            asks,
            authority: self.authority,
            airspace: self.airspace,
            payer,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &data,
            accounts,
        ))
    }

    pub fn initialize_margin_user(&self, owner: Pubkey) -> Result<Instruction> {
        let borrower_account = self.margin_user_account(owner);
        let accounts = jet_fixed_term::accounts::InitializeMarginUser {
            market: self.market,
            payer: self.payer.unwrap(),
            borrower_account,
            margin_account: owner,
            claims: FixedTermIxBuilder::user_claims(borrower_account),
            ticket_collateral: FixedTermIxBuilder::user_ticket_collateral(borrower_account),
            claims_mint: self.claims,
            ticket_collateral_mint: self.ticket_collateral,
            underlying_settlement: get_associated_token_address(&owner, &self.underlying_mint),
            ticket_settlement: get_associated_token_address(&owner, &self.ticket_mint),
            rent: solana_sdk::sysvar::rent::ID,
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
            claims_metadata: get_metadata_address(&self.claims),
            ticket_collateral_metadata: get_metadata_address(&self.ticket_collateral),
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &jet_fixed_term::instruction::InitializeMarginUser {}.data(),
            accounts,
        ))
    }

    /// can derive keys from `owner`
    /// else needs vault addresses
    pub fn convert_tokens(
        &self,
        owner: Pubkey,
        token_vault: Option<Pubkey>,
        ticket_vault: Option<Pubkey>,
        amount: u64,
    ) -> Result<Instruction> {
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
        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &data,
            accounts,
        ))
    }

    pub fn stake_tickets(
        &self,
        ticket_holder: Pubkey,
        ticket_vault: Option<Pubkey>,
        amount: u64,
        seed: &[u8],
    ) -> Result<Instruction> {
        let claim_ticket = self.claim_ticket_key(&ticket_holder, seed);

        let ticket_token_account = match ticket_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&ticket_holder, &self.ticket_mint),
        };
        let data = jet_fixed_term::instruction::StakeTickets {
            params: StakeTicketsParams {
                amount,
                ticket_seed: seed.to_vec(),
            },
        }
        .data();
        let accounts = jet_fixed_term::accounts::StakeTickets {
            claim_ticket,
            market: self.market,
            ticket_holder,
            ticket_token_account,
            ticket_mint: self.ticket_mint,
            payer: self.payer.unwrap(),
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &data,
            accounts,
        ))
    }

    pub fn redeem_ticket(
        &self,
        ticket_holder: Pubkey,
        ticket: Pubkey,
        token_vault: Option<Pubkey>,
    ) -> Result<Instruction> {
        let data = jet_fixed_term::instruction::RedeemTicket {}.data();
        let accounts = self
            .redeem_ticket_accounts(ticket_holder, ticket, token_vault)
            .to_account_metas(None);
        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &data,
            accounts,
        ))
    }

    pub fn settle(
        &self,
        margin_account: Pubkey,
        underlying_settlement: Option<Pubkey>,
        ticket_settlement: Option<Pubkey>,
    ) -> Result<Instruction> {
        let user = self.margin_user(margin_account);
        let accounts = jet_fixed_term::accounts::Settle {
            market: self.market,
            ticket_mint: self.ticket_mint,
            token_program: spl_token::ID,
            margin_user: user.address,
            claims: user.claims,
            claims_mint: self.claims,
            ticket_collateral: user.ticket_collateral,
            ticket_collateral_mint: self.ticket_collateral,
            underlying_token_vault: self.underlying_token_vault,
            underlying_settlement: underlying_settlement.unwrap_or_else(|| {
                get_associated_token_address(&margin_account, &self.underlying_mint)
            }),
            ticket_settlement: ticket_settlement.unwrap_or_else(|| {
                get_associated_token_address(&margin_account, &self.ticket_mint)
            }),
        };
        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &jet_fixed_term::instruction::Settle {}.data(),
            accounts.to_account_metas(None),
        ))
    }

    pub fn margin_redeem_ticket(
        &self,
        margin_account: Pubkey,
        ticket: Pubkey,
        token_vault: Option<Pubkey>,
    ) -> Result<Instruction> {
        let margin_user = self.margin_user(margin_account);
        let data = jet_fixed_term::instruction::MarginRedeemTicket {}.data();
        let accounts = jet_fixed_term::accounts::MarginRedeemTicket {
            margin_user: margin_user.address,
            ticket_collateral: margin_user.ticket_collateral,
            ticket_collateral_mint: self.ticket_collateral,
            inner: self.redeem_ticket_accounts(margin_account, ticket, token_vault),
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &data,
            accounts,
        ))
    }

    pub fn redeem_ticket_accounts(
        &self,
        authority: Pubkey,
        ticket: Pubkey,
        token_vault: Option<Pubkey>,
    ) -> jet_fixed_term::accounts::RedeemTicket {
        let claimant_token_account = match token_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&authority, &self.underlying_mint),
        };
        jet_fixed_term::accounts::RedeemTicket {
            ticket,
            authority,
            claimant_token_account,
            market: self.market,
            underlying_token_vault: self.underlying_token_vault,
            token_program: spl_token::ID,
        }
    }

    pub fn refresh_position(
        &self,
        margin_account: Pubkey,
        expect_price: bool,
    ) -> Result<Instruction> {
        Ok(Instruction {
            program_id: jet_fixed_term::ID,
            accounts: jet_fixed_term::accounts::RefreshPosition {
                market: self.market,
                margin_user: fixed_term_market_pda(&[
                    seeds::MARGIN_BORROWER,
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
        })
    }

    pub fn sell_tickets_order(
        &self,
        user: Pubkey,
        ticket_vault: Option<Pubkey>,
        token_vault: Option<Pubkey>,
        params: OrderParams,
    ) -> Result<Instruction> {
        let data = jet_fixed_term::instruction::SellTicketsOrder { params }.data();
        let accounts = self
            .sell_tickets_order_accounts(user, ticket_vault, token_vault)?
            .to_account_metas(None);
        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &data,
            accounts,
        ))
    }

    pub fn margin_sell_tickets_order(
        &self,
        margin_account: Pubkey,
        ticket_vault: Option<Pubkey>,
        token_vault: Option<Pubkey>,
        params: OrderParams,
    ) -> Result<Instruction> {
        let margin_user = self.margin_user(margin_account);
        let data = jet_fixed_term::instruction::MarginSellTicketsOrder { params }.data();
        let accounts = jet_fixed_term::accounts::MarginSellTicketsOrder {
            margin_user: margin_user.address,
            ticket_collateral: margin_user.ticket_collateral,
            ticket_collateral_mint: self.ticket_collateral,
            inner: self.sell_tickets_order_accounts(margin_account, ticket_vault, token_vault)?,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &data,
            accounts,
        ))
    }

    fn sell_tickets_order_accounts(
        &self,
        authority: Pubkey,
        ticket_vault: Option<Pubkey>,
        token_vault: Option<Pubkey>,
    ) -> Result<jet_fixed_term::accounts::SellTicketsOrder> {
        let user_ticket_vault = match ticket_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&authority, &self.ticket_mint),
        };
        let user_token_vault = match token_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&authority, &self.underlying_mint),
        };
        Ok(jet_fixed_term::accounts::SellTicketsOrder {
            authority,
            user_ticket_vault,
            user_token_vault,
            ticket_mint: self.ticket_mint,
            underlying_token_vault: self.underlying_token_vault,
            orderbook_mut: self.orderbook_mut()?,
            token_program: spl_token::ID,
        })
    }

    pub fn margin_borrow_order(
        &self,
        margin_account: Pubkey,
        underlying_settlement: Option<Pubkey>,
        params: OrderParams,
        seed: &[u8],
    ) -> Result<Instruction> {
        let margin_user = self.margin_user(margin_account);

        let data = jet_fixed_term::instruction::MarginBorrowOrder {
            params,
            seed: seed.to_vec(),
        }
        .data();
        let accounts = jet_fixed_term::accounts::MarginBorrowOrder {
            orderbook_mut: self.orderbook_mut()?,
            margin_user: margin_user.address,
            margin_account,
            claims: margin_user.claims,
            term_loan: self.term_loan_key(&margin_user.address, seed),
            claims_mint: self.claims,
            ticket_collateral: margin_user.ticket_collateral,
            ticket_collateral_mint: self.ticket_collateral,
            underlying_token_vault: self.underlying_token_vault,
            underlying_settlement: underlying_settlement.unwrap_or_else(|| {
                get_associated_token_address(&margin_account, &self.underlying_mint)
            }),
            payer: self.payer.unwrap(),
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);

        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &data,
            accounts,
        ))
    }

    pub fn lend_order(
        &self,
        user: Pubkey,
        lender_tickets: Option<Pubkey>,
        lender_tokens: Option<Pubkey>,
        params: OrderParams,
        seed: &[u8],
    ) -> Result<Instruction> {
        let data = jet_fixed_term::instruction::LendOrder {
            params,
            seed: seed.to_vec(),
        }
        .data();
        let accounts = self
            .lend_order_accounts(user, user, lender_tickets, lender_tokens, params, seed)?
            .to_account_metas(None);
        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &data,
            accounts,
        ))
    }

    pub fn margin_lend_order(
        &self,
        margin_account: Pubkey,
        lender_tokens: Option<Pubkey>,
        params: OrderParams,
        seed: &[u8],
    ) -> Result<Instruction> {
        let margin_user = self.margin_user(margin_account);
        let data = jet_fixed_term::instruction::MarginLendOrder {
            params,
            seed: seed.to_vec(),
        }
        .data();
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
                seed,
            )?,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &data,
            accounts,
        ))
    }

    fn lend_order_accounts(
        &self,
        user: Pubkey,
        authority: Pubkey,
        lender_tickets: Option<Pubkey>,
        lender_tokens: Option<Pubkey>,
        params: OrderParams,
        seed: &[u8],
    ) -> Result<jet_fixed_term::accounts::LendOrder> {
        let lender_tickets = match lender_tickets {
            Some(vault) => vault,
            None => get_associated_token_address(&authority, &self.ticket_mint),
        };
        let lender_tokens = match lender_tokens {
            Some(vault) => vault,
            None => get_associated_token_address(&authority, &self.underlying_mint),
        };
        let split_ticket = self.split_ticket_key(&user, seed);
        Ok(jet_fixed_term::accounts::LendOrder {
            authority,
            ticket_settlement: if params.auto_stake {
                split_ticket
            } else {
                lender_tickets
            },
            lender_tokens,
            underlying_token_vault: self.underlying_token_vault,
            ticket_mint: self.ticket_mint,
            payer: self.payer.unwrap(),
            orderbook_mut: self.orderbook_mut()?,
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
        })
    }

    pub fn cancel_order(&self, owner: Pubkey, order_id: u128) -> Result<Instruction> {
        let data = jet_fixed_term::instruction::CancelOrder { order_id }.data();
        let accounts = jet_fixed_term::accounts::CancelOrder {
            owner,
            orderbook_mut: self.orderbook_mut()?,
        }
        .to_account_metas(None);

        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &data,
            accounts,
        ))
    }

    pub fn pause_order_matching(&self) -> Result<Instruction> {
        let data = jet_fixed_term::instruction::PauseOrderMatching {}.data();
        let accounts = jet_fixed_term::accounts::PauseOrderMatching {
            market: self.market,
            orderbook_market_state: self.orderbook_market_state,
            authority: self.authority,
            airspace: self.airspace,
        }
        .to_account_metas(None);

        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &data,
            accounts,
        ))
    }

    pub fn resume_order_matching(&self) -> Result<Instruction> {
        let data = jet_fixed_term::instruction::ResumeOrderMatching {}.data();
        let accounts = jet_fixed_term::accounts::ResumeOrderMatching {
            market: self.market,
            orderbook_market_state: self.orderbook_market_state,
            event_queue: self.orderbook.as_ref().unwrap().event_queue,
            bids: self.orderbook.as_ref().unwrap().bids,
            asks: self.orderbook.as_ref().unwrap().asks,
            authority: self.authority,
            airspace: self.airspace,
        }
        .to_account_metas(None);

        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &data,
            accounts,
        ))
    }

    pub fn pause_ticket_redemption(&self) -> Result<Instruction> {
        self.modify_market([true as u8].into(), 8 + 32 * 14 + 2)
    }
    pub fn resume_ticket_redemption(&self) -> Result<Instruction> {
        self.modify_market([false as u8].into(), 8 + 32 * 14 + 2)
    }

    pub fn modify_market(&self, data: Vec<u8>, offset: usize) -> Result<Instruction> {
        let data = jet_fixed_term::instruction::ModifyMarket { data, offset }.data();
        let accounts = jet_fixed_term::accounts::ModifyMarket {
            market: self.market,
            authority: self.authority,
            airspace: self.airspace,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &data,
            accounts,
        ))
    }

    pub fn authorize_crank(&self, payer: Pubkey) -> Result<Instruction> {
        let data = jet_fixed_term::instruction::AuthorizeCrank {}.data();
        let accounts = jet_fixed_term::accounts::AuthorizeCrank {
            crank: self
                .crank
                .ok_or_else(|| FixedTermMarketIxError::MissingPubkey("crank".into()))?,
            market: self.market,
            crank_authorization: self.crank_authorization()?,
            authority: self.authority,
            airspace: self.airspace,
            payer,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &data,
            accounts,
        ))
    }

    pub async fn create_orderbook_accounts(
        &self,
        rpc: Arc<dyn SolanaRpcClient>,
        event_queue: Pubkey,
        bids: Pubkey,
        asks: Pubkey,
        queue_capacity: usize,
        book_capacity: usize,
    ) -> Result<Vec<Instruction>> {
        let init_eq = {
            let rent = rpc
                .get_minimum_balance_for_rent_exemption(event_queue_len(queue_capacity))
                .await
                .map_err(client_err)?;
            self.initialize_event_queue(&event_queue, queue_capacity, rent)?
        };

        let init_bids = {
            let rent = rpc
                .get_minimum_balance_for_rent_exemption(orderbook_slab_len(book_capacity))
                .await
                .map_err(client_err)?;
            self.initialize_orderbook_slab(&bids, book_capacity, rent)?
        };
        let init_asks = {
            let rent = rpc
                .get_minimum_balance_for_rent_exemption(orderbook_slab_len(book_capacity))
                .await
                .map_err(client_err)?;
            self.initialize_orderbook_slab(&asks, book_capacity, rent)?
        };

        Ok(vec![init_eq, init_bids, init_asks])
    }

    pub fn margin_settle(&self, margin_account: Pubkey) -> Instruction {
        let data = jet_fixed_term::instruction::Settle {}.data();
        let margin_user = self.margin_user_account(margin_account);
        let claims = FixedTermIxBuilder::user_claims(margin_user);
        let ticket_collateral = FixedTermIxBuilder::user_ticket_collateral(margin_user);
        let accounts = jet_fixed_term::accounts::Settle {
            margin_user,
            market: self.market,
            token_program: spl_token::ID,
            claims,
            claims_mint: self.claims,
            ticket_collateral,
            ticket_collateral_mint: self.ticket_collateral,
            underlying_token_vault: self.underlying_token_vault,
            ticket_mint: self.ticket_mint,
            underlying_settlement: get_associated_token_address(
                &margin_account,
                &self.underlying_mint,
            ),
            ticket_settlement: get_associated_token_address(&margin_account, &self.ticket_mint),
        }
        .to_account_metas(None);

        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }

    pub fn margin_repay(
        &self,
        payer: &Pubkey,
        margin_account: &Pubkey,
        term_loan_seed: &[u8],
        next_term_loan_seed: &[u8],
        amount: u64,
    ) -> Instruction {
        let margin_user = self.margin_user(*margin_account);
        let data = jet_fixed_term::instruction::Repay { amount }.data();
        let accounts = jet_fixed_term::accounts::Repay {
            borrower_account: margin_user.address,
            term_loan: self.term_loan_key(&margin_user.address, term_loan_seed),
            next_term_loan: self.term_loan_key(&margin_user.address, next_term_loan_seed),
            source: get_associated_token_address(payer, &self.underlying_mint),
            payer: *payer,
            underlying_token_vault: self.underlying_token_vault,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);

        Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
    }
}

/// helpful addresses for a MarginUser account
pub struct MarginUser {
    pub address: Pubkey,
    pub claims: Pubkey,
    pub ticket_collateral: Pubkey,
}

impl FixedTermIxBuilder {
    pub fn margin_user(&self, margin_account: Pubkey) -> MarginUser {
        let address = fixed_term_market_pda(&[
            jet_fixed_term::seeds::MARGIN_BORROWER,
            self.market.as_ref(),
            margin_account.as_ref(),
        ]);
        MarginUser {
            address,
            ticket_collateral: fixed_term_market_pda(&[
                jet_fixed_term::seeds::TICKET_COLLATERAL_NOTES,
                address.as_ref(),
            ]),
            claims: fixed_term_market_pda(&[jet_fixed_term::seeds::CLAIM_NOTES, address.as_ref()]),
        }
    }

    pub fn market_key(airspace: &Pubkey, mint: &Pubkey, seed: [u8; 32]) -> Pubkey {
        fixed_term_market_pda(&[
            jet_fixed_term::seeds::MARKET,
            airspace.as_ref(),
            mint.as_ref(),
            &seed,
        ])
    }

    pub fn split_ticket_key(&self, user: &Pubkey, seed: &[u8]) -> Pubkey {
        fixed_term_market_pda(&[jet_fixed_term::seeds::SPLIT_TICKET, user.as_ref(), seed])
    }

    pub fn claims_mint(market_key: &Pubkey) -> Pubkey {
        fixed_term_market_pda(&[jet_fixed_term::seeds::CLAIM_NOTES, market_key.as_ref()])
    }

    pub fn collateral_mint(market_key: &Pubkey) -> Pubkey {
        fixed_term_market_pda(&[
            jet_fixed_term::seeds::TICKET_COLLATERAL_NOTES,
            market_key.as_ref(),
        ])
    }

    pub fn claim_ticket_key(&self, ticket_holder: &Pubkey, seed: &[u8]) -> Pubkey {
        fixed_term_market_pda(&[
            jet_fixed_term::seeds::CLAIM_TICKET,
            self.market.as_ref(),
            ticket_holder.as_ref(),
            seed,
        ])
    }
    pub fn term_loan_key(&self, borrower_account: &Pubkey, seed: &[u8]) -> Pubkey {
        fixed_term_market_pda(&TermLoan::make_seeds(borrower_account.as_ref(), seed))
    }

    pub fn margin_user_account(&self, owner: Pubkey) -> Pubkey {
        fixed_term_market_pda(&[
            jet_fixed_term::seeds::MARGIN_BORROWER,
            self.market.as_ref(),
            owner.as_ref(),
        ])
    }

    pub fn user_claims(borrower_account: Pubkey) -> Pubkey {
        fixed_term_market_pda(&[
            jet_fixed_term::seeds::CLAIM_NOTES,
            borrower_account.as_ref(),
        ])
    }

    pub fn user_ticket_collateral(borrower_account: Pubkey) -> Pubkey {
        fixed_term_market_pda(&[
            jet_fixed_term::seeds::TICKET_COLLATERAL_NOTES,
            borrower_account.as_ref(),
        ])
    }
    pub fn crank_authorization(&self) -> Result<Pubkey> {
        Ok(Pubkey::find_program_address(
            &[
                jet_fixed_term::seeds::CRANK_AUTHORIZATION,
                self.airspace.as_ref(),
                self.market.as_ref(),
                self.crank
                    .ok_or_else(|| FixedTermMarketIxError::MissingPubkey("crank".to_string()))?
                    .as_ref(),
            ],
            &jet_fixed_term::ID,
        )
        .0)
    }

    pub fn jet_fixed_term_id() -> Pubkey {
        jet_fixed_term::ID
    }
}

/// Convenience struct for passing around an `EventQueue`
#[derive(Clone)]
pub struct OwnedEventQueue(Vec<u8>);

impl OwnedEventQueue {
    pub fn inner(&mut self) -> Result<EventQueue<CallbackInfo>> {
        EventQueue::from_buffer(&mut self.0, AccountTag::EventQueue)
            .map_err(|e| FixedTermMarketIxError::Deserialization(e.to_string()))
    }

    pub fn is_empty(&mut self) -> Result<bool> {
        Ok(self.inner()?.iter().next().is_none())
    }

    pub fn consume_events_params(&mut self) -> Result<ConsumeEventsParams> {
        ConsumeEventsInfo::build(self.inner()?).map(|info| info.as_params())
    }
}

impl<T: Into<Vec<u8>>> From<T> for OwnedEventQueue {
    fn from(data: T) -> Self {
        Self(data.into())
    }
}

pub fn fixed_term_market_pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &jet_fixed_term::ID).0
}
