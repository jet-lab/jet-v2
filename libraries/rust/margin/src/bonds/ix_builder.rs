use std::sync::Arc;

use agnostic_orderbook::state::event_queue::EventQueue;
use anchor_lang::{InstructionData, ToAccountMetas};
use jet_bonds::{
    margin::state::Obligation, orderbook::state::CallbackInfo, seeds,
    tickets::instructions::StakeBondTicketsParams,
};
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use spl_associated_token_account::get_associated_token_address;

pub use jet_bonds::{
    control::{
        instructions::{InitializeBondManagerParams, InitializeOrderbookParams},
        state::BondManager,
    },
    orderbook::state::{event_queue_len, orderbook_slab_len, OrderParams},
    ID,
};

use crate::ix_builder::get_metadata_address;

use super::{
    error::{client_err, BondsIxError, Result},
    event_builder::build_consume_events_info,
};

#[derive(Clone, Debug)]
pub struct BondsIxBuilder {
    airspace: Pubkey,
    authority: Pubkey,
    manager: Pubkey,
    underlying_mint: Pubkey,
    bond_ticket_mint: Pubkey,
    underlying_token_vault: Pubkey,
    claims: Pubkey,
    collateral: Pubkey,
    orderbook_market_state: Pubkey,
    underlying_oracle: Pubkey,
    ticket_oracle: Pubkey,
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
        self.ok_or(BondsIxError::MissingPubkey(msg.into()))
    }
}

impl UnwrapKey for Option<&Pubkey> {
    fn unwrap_key(&self, msg: &str) -> Result<Pubkey> {
        Ok(*self.ok_or(BondsIxError::MissingPubkey(msg.into()))?)
    }
}

impl From<BondManager> for BondsIxBuilder {
    fn from(bond_manager: BondManager) -> Self {
        BondsIxBuilder {
            airspace: bond_manager.airspace,
            authority: Pubkey::default(), //todo
            manager: bonds_pda(&[
                seeds::BOND_MANAGER,
                bond_manager.airspace.as_ref(),
                bond_manager.underlying_token_mint.as_ref(),
                &bond_manager.seed,
            ]),
            underlying_mint: bond_manager.underlying_token_mint,
            bond_ticket_mint: bond_manager.bond_ticket_mint,
            underlying_token_vault: bond_manager.underlying_token_vault,
            claims: bond_manager.claims_mint,
            collateral: bond_manager.collateral_mint,
            orderbook_market_state: bond_manager.orderbook_market_state,
            underlying_oracle: bond_manager.underlying_oracle,
            ticket_oracle: bond_manager.ticket_oracle,
            orderbook: Some(OrderBookAddresses {
                bids: bond_manager.bids,
                asks: bond_manager.asks,
                event_queue: bond_manager.event_queue,
            }),
            payer: None,
            crank: None,
        }
    }
}

impl BondsIxBuilder {
    pub fn new(
        airspace: Pubkey,
        underlying_mint: Pubkey,
        manager: Pubkey,
        authority: Pubkey,
        underlying_oracle: Pubkey,
        ticket_oracle: Pubkey,
    ) -> Self {
        let bond_ticket_mint = bonds_pda(&[jet_bonds::seeds::BOND_TICKET_MINT, manager.as_ref()]);
        let underlying_token_vault =
            bonds_pda(&[jet_bonds::seeds::UNDERLYING_TOKEN_VAULT, manager.as_ref()]);
        let orderbook_market_state =
            bonds_pda(&[jet_bonds::seeds::ORDERBOOK_MARKET_STATE, manager.as_ref()]);
        let claims = bonds_pda(&[jet_bonds::seeds::CLAIM_NOTES, manager.as_ref()]);
        let collateral = bonds_pda(&[jet_bonds::seeds::COLLATERAL_NOTES, manager.as_ref()]);
        Self {
            airspace,
            authority,
            manager,
            underlying_mint,
            bond_ticket_mint,
            underlying_token_vault,
            claims,
            collateral,
            orderbook_market_state,
            underlying_oracle,
            ticket_oracle,
            orderbook: None,
            payer: None,
            crank: None,
        }
    }

    /// derives the bond manager key from a mint and seed
    pub fn new_from_seed(
        airspace: &Pubkey,
        mint: &Pubkey,
        seed: [u8; 32],
        authority: Pubkey,
        underlying_oracle: Pubkey,
        ticket_oracle: Pubkey,
    ) -> Self {
        Self::new(
            *airspace,
            *mint,
            Self::bond_manager_key(airspace, mint, seed),
            authority,
            underlying_oracle,
            ticket_oracle,
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

impl BondsIxBuilder {
    pub fn token_mint(&self) -> Pubkey {
        self.underlying_mint
    }
    pub fn ticket_mint(&self) -> Pubkey {
        self.bond_ticket_mint
    }
    pub fn manager(&self) -> Pubkey {
        self.manager
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
        self.collateral
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

impl BondsIxBuilder {
    pub fn orderbook_mut(&self) -> Result<jet_bonds::accounts::OrderbookMut> {
        Ok(jet_bonds::accounts::OrderbookMut {
            bond_manager: self.manager,
            orderbook_market_state: self.orderbook_market_state,
            event_queue: self.orderbook.as_ref().unwrap().event_queue,
            bids: self.orderbook.as_ref().unwrap().bids,
            asks: self.orderbook.as_ref().unwrap().asks,
        })
    }

    pub fn consume_events(&self, event_queue: EventQueue<CallbackInfo>) -> Result<Instruction> {
        let (remaining_accounts, num_events, seed_bytes) =
            build_consume_events_info(event_queue)?.as_params();
        let data = jet_bonds::instruction::ConsumeEvents {
            num_events,
            seed_bytes,
        }
        .data();
        let mut accounts = jet_bonds::accounts::ConsumeEvents {
            bond_manager: self.manager,
            bond_ticket_mint: self.bond_ticket_mint,
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
            remaining_accounts
                .into_iter()
                .map(|k| AccountMeta::new(k, false)),
        );
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn initialize_manager(
        &self,
        payer: Pubkey,
        version_tag: u64,
        seed: [u8; 32],
        borrow_duration: i64,
        lend_duration: i64,
    ) -> Result<Instruction> {
        let data = jet_bonds::instruction::InitializeBondManager {
            params: InitializeBondManagerParams {
                version_tag,
                seed,
                borrow_duration,
                lend_duration,
            },
        }
        .data();
        let accounts = jet_bonds::accounts::InitializeBondManager {
            bond_manager: self.manager,
            underlying_token_mint: self.underlying_mint,
            underlying_token_vault: self.underlying_token_vault,
            bond_ticket_mint: self.bond_ticket_mint,
            claims: self.claims,
            collateral: self.collateral,
            authority: self.authority,
            airspace: self.airspace,
            underlying_oracle: self.underlying_oracle,
            ticket_oracle: self.ticket_oracle,
            payer,
            rent: solana_sdk::sysvar::rent::ID,
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
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
            &jet_bonds::ID,
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
            &jet_bonds::ID,
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
        let data = jet_bonds::instruction::InitializeOrderbook {
            params: InitializeOrderbookParams {
                min_base_order_size,
            },
        }
        .data();
        let accounts = jet_bonds::accounts::InitializeOrderbook {
            bond_manager: self.manager,
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
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn initialize_margin_user(&self, owner: Pubkey) -> Result<Instruction> {
        let borrower_account = self.margin_user_account(owner);
        let accounts = jet_bonds::accounts::InitializeMarginUser {
            bond_manager: self.manager,
            payer: self.payer.unwrap(),
            borrower_account,
            margin_account: owner,
            claims: BondsIxBuilder::user_claims(borrower_account),
            collateral: BondsIxBuilder::user_collateral(borrower_account),
            claims_mint: self.claims,
            collateral_mint: self.collateral,
            underlying_settlement: get_associated_token_address(&owner, &self.underlying_mint),
            ticket_settlement: get_associated_token_address(&owner, &self.bond_ticket_mint),
            rent: solana_sdk::sysvar::rent::ID,
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
            claims_metadata: get_metadata_address(&self.claims),
            collateral_metadata: get_metadata_address(&self.collateral),
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(
            jet_bonds::ID,
            &jet_bonds::instruction::InitializeMarginUser {}.data(),
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
        let user_bond_ticket_vault = match ticket_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&owner, &self.bond_ticket_mint),
        };
        let user_underlying_token_vault = match token_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&owner, &self.underlying_mint),
        };

        let data = jet_bonds::instruction::ExchangeTokens { amount }.data();
        let accounts = jet_bonds::accounts::ExchangeTokens {
            bond_manager: self.manager,
            underlying_token_vault: self.underlying_token_vault,
            bond_ticket_mint: self.bond_ticket_mint,
            user_bond_ticket_vault,
            user_underlying_token_vault,
            user_authority: owner,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn stake_tickets(
        &self,
        ticket_holder: Pubkey,
        ticket_vault: Option<Pubkey>,
        amount: u64,
        seed: &[u8],
    ) -> Result<Instruction> {
        let claim_ticket = self.claim_ticket_key(&ticket_holder, seed);

        let bond_ticket_token_account = match ticket_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&ticket_holder, &self.bond_ticket_mint),
        };
        let data = jet_bonds::instruction::StakeBondTickets {
            params: StakeBondTicketsParams {
                amount,
                ticket_seed: seed.to_vec(),
            },
        }
        .data();
        let accounts = jet_bonds::accounts::StakeBondTickets {
            claim_ticket,
            bond_manager: self.manager,
            ticket_holder,
            bond_ticket_token_account,
            bond_ticket_mint: self.bond_ticket_mint,
            payer: self.payer.unwrap(),
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn redeem_ticket(
        &self,
        ticket_holder: Pubkey,
        ticket: Pubkey,
        token_vault: Option<Pubkey>,
    ) -> Result<Instruction> {
        let data = jet_bonds::instruction::RedeemTicket {}.data();
        let accounts = self
            .redeem_ticket_accounts(ticket_holder, ticket, token_vault)
            .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn settle(
        &self,
        margin_account: Pubkey,
        underlying_settlement: Option<Pubkey>,
        ticket_settlement: Option<Pubkey>,
    ) -> Result<Instruction> {
        let user = self.margin_user(margin_account);
        let accounts = jet_bonds::accounts::Settle {
            bond_manager: self.manager,
            bond_ticket_mint: self.bond_ticket_mint,
            token_program: spl_token::ID,
            margin_user: user.address,
            claims: user.claims,
            claims_mint: self.claims,
            collateral: user.collateral,
            collateral_mint: self.collateral,
            underlying_token_vault: self.underlying_token_vault,
            underlying_settlement: underlying_settlement.unwrap_or_else(|| {
                get_associated_token_address(&margin_account, &self.underlying_mint)
            }),
            ticket_settlement: ticket_settlement.unwrap_or_else(|| {
                get_associated_token_address(&margin_account, &self.bond_ticket_mint)
            }),
        };
        Ok(Instruction::new_with_bytes(
            jet_bonds::ID,
            &jet_bonds::instruction::Settle {}.data(),
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
        let data = jet_bonds::instruction::MarginRedeemTicket {}.data();
        let accounts = jet_bonds::accounts::MarginRedeemTicket {
            margin_user: margin_user.address,
            collateral: margin_user.collateral,
            collateral_mint: self.collateral,
            inner: self.redeem_ticket_accounts(margin_account, ticket, token_vault),
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn redeem_ticket_accounts(
        &self,
        authority: Pubkey,
        ticket: Pubkey,
        token_vault: Option<Pubkey>,
    ) -> jet_bonds::accounts::RedeemTicket {
        let claimant_token_account = match token_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&authority, &self.underlying_mint),
        };
        jet_bonds::accounts::RedeemTicket {
            ticket,
            authority,
            claimant_token_account,
            bond_manager: self.manager,
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
            program_id: jet_bonds::ID,
            accounts: jet_bonds::accounts::RefreshPosition {
                bond_manager: self.manager,
                margin_user: bonds_pda(&[
                    seeds::MARGIN_BORROWER,
                    self.manager.as_ref(),
                    margin_account.as_ref(),
                ]),
                margin_account,
                underlying_oracle: self.underlying_oracle,
                ticket_oracle: self.ticket_oracle,
                token_program: spl_token::ID,
            }
            .to_account_metas(None),
            data: jet_bonds::instruction::RefreshPosition { expect_price }.data(),
        })
    }

    pub fn sell_tickets_order(
        &self,
        user: Pubkey,
        ticket_vault: Option<Pubkey>,
        token_vault: Option<Pubkey>,
        params: OrderParams,
    ) -> Result<Instruction> {
        let data = jet_bonds::instruction::SellTicketsOrder { params }.data();
        let accounts = self
            .sell_tickets_order_accounts(user, ticket_vault, token_vault)?
            .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn margin_sell_tickets_order(
        &self,
        margin_account: Pubkey,
        ticket_vault: Option<Pubkey>,
        token_vault: Option<Pubkey>,
        params: OrderParams,
    ) -> Result<Instruction> {
        let margin_user = self.margin_user(margin_account);
        let data = jet_bonds::instruction::MarginSellTicketsOrder { params }.data();
        let accounts = jet_bonds::accounts::MarginSellTicketsOrder {
            margin_user: margin_user.address,
            collateral: margin_user.collateral,
            collateral_mint: self.collateral,
            inner: self.sell_tickets_order_accounts(margin_account, ticket_vault, token_vault)?,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    fn sell_tickets_order_accounts(
        &self,
        authority: Pubkey,
        ticket_vault: Option<Pubkey>,
        token_vault: Option<Pubkey>,
    ) -> Result<jet_bonds::accounts::SellTicketsOrder> {
        let user_ticket_vault = match ticket_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&authority, &self.bond_ticket_mint),
        };
        let user_token_vault = match token_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&authority, &self.underlying_mint),
        };
        Ok(jet_bonds::accounts::SellTicketsOrder {
            authority,
            user_ticket_vault,
            user_token_vault,
            bond_ticket_mint: self.bond_ticket_mint,
            underlying_token_vault: self.underlying_token_vault,
            orderbook_mut: self.orderbook_mut()?,
            token_program: spl_token::ID,
        })
    }

    pub fn margin_borrow_order(
        &self,
        margin_account: Pubkey,
        params: OrderParams,
        seed: &[u8],
    ) -> Result<Instruction> {
        let margin_user = self.margin_user(margin_account);

        let data = jet_bonds::instruction::MarginBorrowOrder {
            params,
            seed: seed.to_vec(),
        }
        .data();
        let accounts = jet_bonds::accounts::MarginBorrowOrder {
            orderbook_mut: self.orderbook_mut()?,
            margin_user: margin_user.address,
            margin_account,
            claims: margin_user.claims,
            obligation: self.obligation_key(&margin_user.address, seed),
            claims_mint: self.claims,
            collateral: margin_user.collateral,
            collateral_mint: self.collateral,
            payer: self.payer.unwrap(),
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);

        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn lend_order(
        &self,
        user: Pubkey,
        lender_tickets: Option<Pubkey>,
        lender_tokens: Option<Pubkey>,
        params: OrderParams,
        seed: &[u8],
    ) -> Result<Instruction> {
        let data = jet_bonds::instruction::LendOrder {
            params,
            seed: seed.to_vec(),
        }
        .data();
        let accounts = self
            .lend_order_accounts(user, user, lender_tickets, lender_tokens, params, seed)?
            .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn margin_lend_order(
        &self,
        margin_account: Pubkey,
        lender_tokens: Option<Pubkey>,
        params: OrderParams,
        seed: &[u8],
    ) -> Result<Instruction> {
        let margin_user = self.margin_user(margin_account);
        let data = jet_bonds::instruction::MarginLendOrder {
            params,
            seed: seed.to_vec(),
        }
        .data();
        let accounts = jet_bonds::accounts::MarginLendOrder {
            margin_user: margin_user.address,
            collateral: margin_user.collateral,
            collateral_mint: self.collateral,
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
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    fn lend_order_accounts(
        &self,
        user: Pubkey,
        authority: Pubkey,
        lender_tickets: Option<Pubkey>,
        lender_tokens: Option<Pubkey>,
        params: OrderParams,
        seed: &[u8],
    ) -> Result<jet_bonds::accounts::LendOrder> {
        let lender_tickets = match lender_tickets {
            Some(vault) => vault,
            None => get_associated_token_address(&authority, &self.bond_ticket_mint),
        };
        let lender_tokens = match lender_tokens {
            Some(vault) => vault,
            None => get_associated_token_address(&authority, &self.underlying_mint),
        };
        let split_ticket = self.split_ticket_key(&user, seed);
        Ok(jet_bonds::accounts::LendOrder {
            authority,
            ticket_settlement: if params.auto_stake {
                split_ticket
            } else {
                lender_tickets
            },
            lender_tokens,
            underlying_token_vault: self.underlying_token_vault,
            ticket_mint: self.bond_ticket_mint,
            payer: self.payer.unwrap(),
            orderbook_mut: self.orderbook_mut()?,
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
        })
    }

    pub fn cancel_order(&self, owner: Pubkey, order_id: u128) -> Result<Instruction> {
        let data = jet_bonds::instruction::CancelOrder { order_id }.data();
        let accounts = jet_bonds::accounts::CancelOrder {
            owner,
            orderbook_mut: self.orderbook_mut()?,
        }
        .to_account_metas(None);

        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn pause_order_matching(&self) -> Result<Instruction> {
        let data = jet_bonds::instruction::PauseOrderMatching {}.data();
        let accounts = jet_bonds::accounts::PauseOrderMatching {
            bond_manager: self.manager,
            orderbook_market_state: self.orderbook_market_state,
            authority: self.authority,
            airspace: self.airspace,
        }
        .to_account_metas(None);

        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn resume_order_matching(&self) -> Result<Instruction> {
        let data = jet_bonds::instruction::ResumeOrderMatching {}.data();
        let accounts = jet_bonds::accounts::ResumeOrderMatching {
            bond_manager: self.manager,
            orderbook_market_state: self.orderbook_market_state,
            event_queue: self.orderbook.as_ref().unwrap().event_queue,
            bids: self.orderbook.as_ref().unwrap().bids,
            asks: self.orderbook.as_ref().unwrap().asks,
            authority: self.authority,
            airspace: self.airspace,
        }
        .to_account_metas(None);

        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn pause_ticket_redemption(&self) -> Result<Instruction> {
        self.modify_manager([true as u8].into(), 8 + 32 * 13 + 2)
    }
    pub fn resume_ticket_redemption(&self) -> Result<Instruction> {
        self.modify_manager([false as u8].into(), 8 + 32 * 13 + 2)
    }

    pub fn modify_manager(&self, data: Vec<u8>, offset: usize) -> Result<Instruction> {
        let data = jet_bonds::instruction::ModifyBondManager { data, offset }.data();
        let accounts = jet_bonds::accounts::ModifyBondManager {
            bond_manager: self.manager,
            authority: self.authority,
            airspace: self.airspace,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn authorize_crank(&self, payer: Pubkey) -> Result<Instruction> {
        let data = jet_bonds::instruction::AuthorizeCrank {}.data();
        let accounts = jet_bonds::accounts::AuthorizeCrank {
            crank: self
                .crank
                .ok_or_else(|| BondsIxError::MissingPubkey("crank".into()))?,
            market: self.manager,
            crank_authorization: self.crank_authorization()?,
            authority: self.authority,
            airspace: self.airspace,
            payer,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
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
        let data = jet_bonds::instruction::Settle {}.data();
        let margin_user = self.margin_user_account(margin_account);
        let claims = BondsIxBuilder::user_claims(margin_user);
        let collateral = BondsIxBuilder::user_collateral(margin_user);
        let accounts = jet_bonds::accounts::Settle {
            margin_user,
            bond_manager: self.manager,
            token_program: spl_token::ID,
            claims,
            claims_mint: self.claims,
            collateral,
            collateral_mint: self.collateral,
            underlying_token_vault: self.underlying_token_vault,
            bond_ticket_mint: self.bond_ticket_mint,
            underlying_settlement: get_associated_token_address(
                &margin_account,
                &self.underlying_mint,
            ),
            ticket_settlement: get_associated_token_address(
                &margin_account,
                &self.bond_ticket_mint,
            ),
        }
        .to_account_metas(None);

        Instruction::new_with_bytes(jet_bonds::ID, &data, accounts)
    }

    pub fn margin_repay(
        &self,
        payer: &Pubkey,
        margin_account: &Pubkey,
        obligation_seed: &[u8],
        next_obligation_seed: &[u8],
        amount: u64,
    ) -> Instruction {
        let margin_user = self.margin_user(*margin_account);
        let data = jet_bonds::instruction::Repay { amount }.data();
        let accounts = jet_bonds::accounts::Repay {
            borrower_account: margin_user.address,
            obligation: self.obligation_key(&margin_user.address, obligation_seed),
            next_obligation: self.obligation_key(&margin_user.address, next_obligation_seed),
            source: get_associated_token_address(payer, &self.underlying_mint),
            payer: *payer,
            underlying_token_vault: self.underlying_token_vault,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);

        Instruction::new_with_bytes(jet_bonds::ID, &data, accounts)
    }
}

/// helpful addresses for a MarginUser account
pub struct MarginUser {
    pub address: Pubkey,
    pub claims: Pubkey,
    pub collateral: Pubkey,
}

impl BondsIxBuilder {
    pub fn margin_user(&self, margin_account: Pubkey) -> MarginUser {
        let address = bonds_pda(&[
            jet_bonds::seeds::MARGIN_BORROWER,
            self.manager.as_ref(),
            margin_account.as_ref(),
        ]);
        MarginUser {
            address,
            collateral: bonds_pda(&[jet_bonds::seeds::COLLATERAL_NOTES, address.as_ref()]),
            claims: bonds_pda(&[jet_bonds::seeds::CLAIM_NOTES, address.as_ref()]),
        }
    }

    pub fn bond_manager_key(airspace: &Pubkey, mint: &Pubkey, seed: [u8; 32]) -> Pubkey {
        bonds_pda(&[
            jet_bonds::seeds::BOND_MANAGER,
            airspace.as_ref(),
            mint.as_ref(),
            &seed,
        ])
    }

    pub fn split_ticket_key(&self, user: &Pubkey, seed: &[u8]) -> Pubkey {
        bonds_pda(&[jet_bonds::seeds::SPLIT_TICKET, user.as_ref(), seed])
    }

    pub fn claims_mint(manager_key: &Pubkey) -> Pubkey {
        bonds_pda(&[jet_bonds::seeds::CLAIM_NOTES, manager_key.as_ref()])
    }

    pub fn collateral_mint(manager_key: &Pubkey) -> Pubkey {
        bonds_pda(&[jet_bonds::seeds::COLLATERAL_NOTES, manager_key.as_ref()])
    }

    pub fn claim_ticket_key(&self, ticket_holder: &Pubkey, seed: &[u8]) -> Pubkey {
        bonds_pda(&[
            jet_bonds::seeds::CLAIM_TICKET,
            self.manager.as_ref(),
            ticket_holder.as_ref(),
            seed,
        ])
    }
    pub fn obligation_key(&self, borrower_account: &Pubkey, seed: &[u8]) -> Pubkey {
        bonds_pda(&Obligation::make_seeds(borrower_account.as_ref(), seed))
    }

    pub fn margin_user_account(&self, owner: Pubkey) -> Pubkey {
        bonds_pda(&[
            jet_bonds::seeds::MARGIN_BORROWER,
            self.manager.as_ref(),
            owner.as_ref(),
        ])
    }

    pub fn user_claims(borrower_account: Pubkey) -> Pubkey {
        bonds_pda(&[jet_bonds::seeds::CLAIM_NOTES, borrower_account.as_ref()])
    }

    pub fn user_collateral(borrower_account: Pubkey) -> Pubkey {
        bonds_pda(&[
            jet_bonds::seeds::COLLATERAL_NOTES,
            borrower_account.as_ref(),
        ])
    }
    pub fn crank_authorization(&self) -> Result<Pubkey> {
        Ok(Pubkey::find_program_address(
            &[
                jet_bonds::seeds::CRANK_AUTHORIZATION,
                self.airspace.as_ref(),
                self.manager.as_ref(),
                self.crank
                    .ok_or_else(|| BondsIxError::MissingPubkey("crank".to_string()))?
                    .as_ref(),
            ],
            &jet_bonds::ID,
        )
        .0)
    }

    pub fn jet_bonds_id() -> Pubkey {
        jet_bonds::ID
    }
}

pub fn bonds_pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &jet_bonds::ID).0
}
