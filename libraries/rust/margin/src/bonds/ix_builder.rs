use std::{collections::HashMap, sync::Arc};

use anchor_lang::{InstructionData, ToAccountMetas};
use jet_bonds::{margin::state::Obligation, seeds, tickets::instructions::StakeBondTicketsParams};
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use rand::rngs::OsRng;
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

use super::event_builder::make_seed;

use super::error::{client_err, BondsIxError, Result};

#[derive(Clone)]
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
    keys: Keys,
}

#[derive(Debug, Default, Clone)]
pub struct Keys(HashMap<String, Pubkey>);

impl Keys {
    pub fn insert(&mut self, k: &str, v: Pubkey) {
        self.0.insert(k.into(), v);
    }
    pub fn unwrap(&self, k: &str) -> Result<Pubkey> {
        self.0.get(k).unwrap_key(k)
    }
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

impl BondsIxBuilder {
    pub fn new(
        underlying_mint: Pubkey,
        manager: Pubkey,
        authority: Pubkey,
        underlying_oracle: Pubkey,
    ) -> Self {
        let bond_ticket_mint = bonds_pda(&[jet_bonds::seeds::BOND_TICKET_MINT, manager.as_ref()]);
        let underlying_token_vault =
            bonds_pda(&[jet_bonds::seeds::UNDERLYING_TOKEN_VAULT, manager.as_ref()]);
        let orderbook_market_state =
            bonds_pda(&[jet_bonds::seeds::ORDERBOOK_MARKET_STATE, manager.as_ref()]);
        let claims = bonds_pda(&[jet_bonds::seeds::CLAIM_NOTES, manager.as_ref()]);
        let collateral = bonds_pda(&[jet_bonds::seeds::DEPOSIT_NOTES, manager.as_ref()]);
        let keys = Keys::default();
        Self {
            airspace: Pubkey::default(), // fixme airspace
            authority,
            manager,
            underlying_mint,
            bond_ticket_mint,
            underlying_token_vault,
            claims,
            collateral,
            orderbook_market_state,
            underlying_oracle,
            keys,
        }
    }

    /// derives the bond manager key from a mint and seed
    pub fn new_from_seed(
        mint: &Pubkey,
        seed: [u8; 32],
        authority: Pubkey,
        underlying_oracle: Pubkey,
    ) -> Self {
        let builder = Self::new(
            *mint,
            Self::bond_manager_key(mint, seed),
            authority,
            underlying_oracle,
        );
        builder.with_mint(mint)
    }

    pub fn with_payer(mut self, payer: &Pubkey) -> Self {
        self.keys.insert("payer", *payer);
        self
    }

    pub fn with_crank(mut self, crank: &Pubkey) -> Self {
        self.keys.insert("crank", *crank);
        self
    }

    pub fn with_orderbook_accounts(
        mut self,
        bids: Option<Pubkey>,
        asks: Option<Pubkey>,
        event_queue: Option<Pubkey>,
    ) -> Self {
        if let Some(bids) = bids {
            self.keys.insert("bids", bids);
        }
        if let Some(asks) = asks {
            self.keys.insert("asks", asks);
        }
        if let Some(eq) = event_queue {
            self.keys.insert("event_queue", eq);
        }
        self
    }

    pub fn with_mint(mut self, underlying_mint: &Pubkey) -> Self {
        self.keys.insert("underlying_mint", *underlying_mint);
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
    pub fn event_queue(&self) -> Result<Pubkey> {
        self.keys.unwrap("event_queue")
    }
    pub fn bids(&self) -> Result<Pubkey> {
        self.keys.unwrap("bids")
    }
    pub fn asks(&self) -> Result<Pubkey> {
        self.keys.unwrap("asks")
    }
}

impl BondsIxBuilder {
    pub fn orderbook_mut(&self) -> Result<jet_bonds::accounts::OrderbookMut> {
        Ok(jet_bonds::accounts::OrderbookMut {
            bond_manager: self.manager,
            orderbook_market_state: self.orderbook_market_state,
            event_queue: self.keys.unwrap("event_queue")?,
            bids: self.keys.unwrap("bids")?,
            asks: self.keys.unwrap("asks")?,
        })
    }

    pub fn consume_events(
        &self,
        remaining_accounts: Vec<Pubkey>,
        num_events: u32,
        seed_bytes: Vec<Vec<u8>>,
    ) -> Result<Instruction> {
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
            event_queue: self.keys.unwrap("event_queue")?,
            crank_authorization: crank_authorization(&self.keys.unwrap("crank")?),
            crank: self.keys.unwrap("crank")?,
            payer: self.keys.unwrap("payer")?,
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
        duration: i64,
        ticket_oracle: Pubkey,
    ) -> Result<Instruction> {
        let data = jet_bonds::instruction::InitializeBondManager {
            params: InitializeBondManagerParams {
                version_tag,
                seed,
                duration,
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
            ticket_oracle,
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
            &self.keys.unwrap("payer")?,
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
            &self.keys.unwrap("payer")?,
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
            payer: self.keys.unwrap("payer")?,
            borrower_account,
            margin_account: owner,
            claims: bonds_pda(&[jet_bonds::seeds::CLAIM_NOTES, borrower_account.as_ref()]),
            collateral: bonds_pda(&[jet_bonds::seeds::DEPOSIT_NOTES, borrower_account.as_ref()]),
            claims_mint: self.claims,
            collateral_mint: self.collateral,
            underlying_settlement: get_associated_token_address(
                &owner,
                &self.keys.unwrap("underlying_mint")?,
            ),
            ticket_settlement: get_associated_token_address(&owner, &self.bond_ticket_mint),
            rent: solana_sdk::sysvar::rent::ID,
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
            claims_metadata: get_metadata_address(&self.claims),
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(
            jet_bonds::ID,
            &jet_bonds::instruction::InitializeMarginUser {}.data(),
            accounts,
        ))
    }

    /// can derive keys from `user`
    /// else needs vault addresses and authority
    pub fn convert_tokens(
        &self,
        user: Option<Pubkey>,
        token_vault: Option<Pubkey>,
        ticket_vault: Option<Pubkey>,
        vault_authority: Option<Pubkey>,
        amount: u64,
    ) -> Result<Instruction> {
        let user_bond_ticket_vault = match ticket_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&user.unwrap_key("user")?, &self.bond_ticket_mint),
        };
        let user_underlying_token_vault = match token_vault {
            Some(vault) => vault,
            None => get_associated_token_address(
                &user.unwrap_key("user")?,
                &self.keys.unwrap("underlying_mint")?,
            ),
        };
        let user_authority = match vault_authority {
            Some(auth) => auth,
            None => user.unwrap_key("user")?,
        };

        let data = jet_bonds::instruction::ExchangeTokens { amount }.data();
        let accounts = jet_bonds::accounts::ExchangeTokens {
            bond_manager: self.manager,
            underlying_token_vault: self.underlying_token_vault,
            bond_ticket_mint: self.bond_ticket_mint,
            user_bond_ticket_vault,
            user_underlying_token_vault,
            user_authority,
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
        seed: Vec<u8>,
    ) -> Result<Instruction> {
        let claim_ticket = self.claim_ticket_key(&ticket_holder, seed.clone());

        let bond_ticket_token_account = match ticket_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&ticket_holder, &self.bond_ticket_mint),
        };
        let data = jet_bonds::instruction::StakeBondTickets {
            params: StakeBondTicketsParams {
                amount,
                ticket_seed: seed,
            },
        }
        .data();
        let accounts = jet_bonds::accounts::StakeBondTickets {
            claim_ticket,
            bond_manager: self.manager,
            ticket_holder,
            bond_ticket_token_account,
            bond_ticket_mint: self.bond_ticket_mint,
            payer: self.keys.unwrap("payer")?,
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
        let claimant_token_account = match token_vault {
            Some(vault) => vault,
            None => {
                get_associated_token_address(&ticket_holder, &self.keys.unwrap("underlying_mint")?)
            }
        };
        let data = jet_bonds::instruction::RedeemTicket {}.data();
        let accounts = jet_bonds::accounts::RedeemTicket {
            ticket,
            ticket_holder,
            claimant_token_account,
            bond_manager: self.manager,
            underlying_token_vault: self.underlying_token_vault,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn refresh_position(&self, margin_account: Pubkey) -> Result<Instruction> {
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
                claims_mint: self.claims,
                underlying_oracle: self.underlying_oracle,
                token_program: spl_token::ID,
            }
            .to_account_metas(None),
            data: jet_bonds::instruction::RefreshPosition { expect_price: true }.data(),
        })
    }

    pub fn sell_tickets_order(
        &self,
        user: Pubkey,
        ticket_vault: Option<Pubkey>,
        token_vault: Option<Pubkey>,
        params: OrderParams,
    ) -> Result<Instruction> {
        let user_ticket_vault = match ticket_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&user, &self.bond_ticket_mint),
        };
        let user_token_vault = match token_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&user, &self.keys.unwrap("underlying_mint")?),
        };
        let data = jet_bonds::instruction::SellTicketsOrder { params }.data();
        let accounts = jet_bonds::accounts::SellTicketsOrder {
            user,
            user_ticket_vault,
            user_token_vault,
            bond_ticket_mint: self.bond_ticket_mint,
            orderbook_mut: self.orderbook_mut()?,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn margin_borrow_order(&self, user: Pubkey, params: OrderParams) -> Result<Instruction> {
        let borrower_account = bonds_pda(&[
            jet_bonds::seeds::MARGIN_BORROWER,
            self.manager.as_ref(),
            user.as_ref(),
        ]);

        let seed = make_seed(&mut OsRng::default());
        let data = jet_bonds::instruction::MarginBorrowOrder {
            params,
            seed: seed.clone(),
        }
        .data();
        let accounts = jet_bonds::accounts::MarginBorrowOrder {
            orderbook_mut: self.orderbook_mut()?,
            borrower_account,
            obligation: bonds_pda(&Obligation::make_seeds(borrower_account.as_ref(), &seed)),
            margin_account: user,
            claims: bonds_pda(&[jet_bonds::seeds::CLAIM_NOTES, borrower_account.as_ref()]),
            claims_mint: self.claims,
            payer: self.keys.unwrap("payer")?,
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);

        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn lend_order(
        &self,
        user: Pubkey,
        ticket_vault: Option<Pubkey>,
        token_vault: Option<Pubkey>,
        params: OrderParams,
        seed: Vec<u8>,
    ) -> Result<Instruction> {
        let user_ticket_vault = match ticket_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&user, &self.bond_ticket_mint),
        };
        let user_token_vault = match token_vault {
            Some(vault) => vault,
            None => get_associated_token_address(&user, &self.keys.unwrap("underlying_mint")?),
        };
        let split_ticket = self.split_ticket_key(&user, seed.clone());
        let data = jet_bonds::instruction::LendOrder { params, seed }.data();
        let accounts = jet_bonds::accounts::LendOrder {
            user,
            user_ticket_vault,
            user_token_vault,
            underlying_token_vault: self.underlying_token_vault,
            split_ticket,
            orderbook_mut: self.orderbook_mut()?,
            payer: self.keys.unwrap("payer")?,
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
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
            event_queue: self.keys.unwrap("event_queue")?,
            bids: self.keys.unwrap("bids")?,
            asks: self.keys.unwrap("asks")?,
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

    pub fn authorize_crank(&self, payer: Pubkey, crank: Pubkey) -> Result<Instruction> {
        let data = jet_bonds::instruction::AuthorizeCrank {}.data();
        let accounts = jet_bonds::accounts::AuthorizeCrank {
            crank,
            crank_authorization: crank_authorization(&crank),
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
}

impl BondsIxBuilder {
    pub fn bond_manager_key(mint: &Pubkey, seed: [u8; 32]) -> Pubkey {
        bonds_pda(&[jet_bonds::seeds::BOND_MANAGER, mint.as_ref(), &seed])
    }
    pub fn split_ticket_key(&self, user: &Pubkey, seed: Vec<u8>) -> Pubkey {
        bonds_pda(&[
            jet_bonds::seeds::SPLIT_TICKET,
            user.as_ref(),
            seed.as_slice(),
        ])
    }
    pub fn claim_ticket_key(&self, ticket_holder: &Pubkey, seed: Vec<u8>) -> Pubkey {
        bonds_pda(&[
            jet_bonds::seeds::CLAIM_TICKET,
            self.manager.as_ref(),
            ticket_holder.as_ref(),
            seed.as_slice(),
        ])
    }

    pub fn margin_user_account(&self, owner: Pubkey) -> Pubkey {
        bonds_pda(&[
            jet_bonds::seeds::MARGIN_BORROWER,
            self.manager.as_ref(),
            owner.as_ref(),
        ])
    }

    pub fn jet_bonds_id() -> Pubkey {
        jet_bonds::ID
    }
}

pub fn bonds_pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &jet_bonds::ID).0
}

pub fn crank_authorization(crank: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[jet_bonds::seeds::CRANK_AUTHORIZATION, crank.as_ref()],
        &jet_bonds::ID,
    )
    .0
}
