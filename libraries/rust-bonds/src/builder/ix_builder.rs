use std::collections::HashMap;

use anchor_lang::{
    prelude::{AccountMeta, Pubkey},
    InstructionData, ToAccountMetas,
};
use jet_bonds::{
    control::instructions::{InitializeBondManagerParams, InitializeOrderbookParams},
    orderbook::state::OrderParams,
    tickets::instructions::StakeBondTicketsParams,
};
use jet_bonds_metadata::jet_bonds_metadata;
use solana_sdk::instruction::Instruction;
use spl_associated_token_account::get_associated_token_address;

use super::error::{BondsIxError, Result};

pub struct BondsIxBuilder {
    manager: Pubkey,
    bond_ticket_mint: Pubkey,
    underlying_token_vault: Pubkey,
    claims: Pubkey,
    deposits: Pubkey,
    orderbook_market_state: Pubkey,
    keys: Keys,
}

#[derive(Debug, Default)]
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
    pub fn new(manager: Pubkey) -> Self {
        let bond_ticket_mint = bonds_pda(&[jet_bonds::seeds::BOND_TICKET_MINT, manager.as_ref()]);
        let underlying_token_vault =
            bonds_pda(&[jet_bonds::seeds::UNDERLYING_TOKEN_VAULT, manager.as_ref()]);
        let orderbook_market_state =
            bonds_pda(&[jet_bonds::seeds::ORDERBOOK_MARKET_STATE, manager.as_ref()]);
        let claims = bonds_pda(&[jet_bonds::seeds::CLAIM_NOTES, manager.as_ref()]);
        let deposits = bonds_pda(&[jet_bonds::seeds::DEPOSIT_NOTES, manager.as_ref()]);
        let keys = Keys::default();
        Self {
            manager,
            bond_ticket_mint,
            underlying_token_vault,
            claims,
            deposits,
            orderbook_market_state,
            keys,
        }
    }

    /// derives the bond manager key from a mint and seed
    pub fn new_from_seed(mint: &Pubkey, seed: u64) -> Self {
        let builder = Self::new(Self::bond_manager_key(mint, seed));
        builder.with_mint(mint)
    }

    pub fn with_payer(mut self, payer: &Pubkey) -> Self {
        self.keys.insert("payer", *payer);
        self
    }
    pub fn with_crank(mut self, crank: &Pubkey) -> Self {
        let crank_metadata = Pubkey::find_program_address(
            &[jet_bonds_metadata::seeds::CRANK_SIGNER, crank.as_ref()],
            &jet_bonds_metadata::ID,
        )
        .0;
        self.keys.insert("crank", *crank);
        self.keys.insert("crank_metadata", crank_metadata);
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
    pub fn with_authority(mut self, authority: &Pubkey) -> Self {
        self.keys.insert("authority", *authority);
        self
    }
}

impl BondsIxBuilder {
    pub fn ticket_mint(&self) -> Pubkey {
        self.bond_ticket_mint
    }
    pub fn manager(&self) -> Pubkey {
        self.manager
    }
    pub fn orderbook_state(&self) -> Pubkey {
        self.orderbook_market_state
    }
}

impl BondsIxBuilder {
    pub fn consume_events(
        &self,
        remaining_accounts: Vec<&Pubkey>,
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
            crank_metadata: self.keys.unwrap("crank_metadata")?,
            crank_signer: self.keys.unwrap("crank")?,
            payer: self.keys.unwrap("payer")?,
            system_program: solana_sdk::system_program::ID,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);
        accounts.extend(
            remaining_accounts
                .into_iter()
                .map(|k| AccountMeta::new(*k, false)),
        );
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn initialize_manager(
        &self,
        version_tag: u64,
        seed: u64,
        duration: i64,
        conversion_factor: i8,
        underlying_mint: &Pubkey,
        underlying_oracle: &Pubkey,
        ticket_oracle: &Pubkey,
    ) -> Result<Instruction> {
        let data = jet_bonds::instruction::InitializeBondManager {
            params: InitializeBondManagerParams {
                version_tag,
                seed,
                duration,
                conversion_factor,
            },
        }
        .data();
        let accounts = jet_bonds::accounts::InitializeBondManager {
            bond_manager: self.manager,
            underlying_token_mint: *underlying_mint,
            underlying_token_vault: self.underlying_token_vault,
            bond_ticket_mint: self.bond_ticket_mint,
            claims: self.claims,
            deposits: self.deposits,
            program_authority: self.keys.unwrap("authority")?,
            underlying_oracle: *underlying_oracle,
            ticket_oracle: *ticket_oracle,
            payer: self.keys.unwrap("payer")?,
            rent: solana_sdk::sysvar::rent::ID,
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn initialize_orderbook(
        &self,
        program_authority: &Pubkey,
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
            event_queue: self.keys.unwrap("event_queue")?,
            bids: self.keys.unwrap("bids")?,
            asks: self.keys.unwrap("asks")?,
            program_authority: *program_authority,
            payer: self.keys.unwrap("payer")?,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    /// can derive keys from `user`
    /// else needs vault addresses and authority
    pub fn convert_tokens(
        &self,
        user: Option<&Pubkey>,
        token_vault: Option<&Pubkey>,
        ticket_vault: Option<&Pubkey>,
        vault_authority: Option<&Pubkey>,
        amount: u64,
    ) -> Result<Instruction> {
        let user_bond_ticket_vault = match ticket_vault {
            Some(vault) => *vault,
            None => get_associated_token_address(&user.unwrap_key("user")?, &self.bond_ticket_mint),
        };
        let user_underlying_token_vault = match token_vault {
            Some(vault) => *vault,
            None => get_associated_token_address(
                &user.unwrap_key("user")?,
                &self.keys.unwrap("underlying_mint")?,
            ),
        };
        let user_authority = match vault_authority {
            Some(auth) => *auth,
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
        ticket_holder: &Pubkey,
        ticket_vault: Option<&Pubkey>,
        amount: u64,
        seed: u64,
    ) -> Result<Instruction> {
        let claim_ticket = self.claim_ticket_key(ticket_holder, seed);

        let bond_ticket_token_account = match ticket_vault {
            Some(vault) => *vault,
            None => get_associated_token_address(ticket_holder, &self.bond_ticket_mint),
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
            ticket_holder: *ticket_holder,
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
        holder: &Pubkey,
        ticket: &Pubkey,
        token_vault: Option<&Pubkey>,
    ) -> Result<Instruction> {
        let claimant_token_account = match token_vault {
            Some(vault) => *vault,
            None => get_associated_token_address(holder, &self.keys.unwrap("underlying_mint")?),
        };
        let data = jet_bonds::instruction::RedeemTicket {}.data();
        let accounts = jet_bonds::accounts::RedeemTicket {
            ticket: *ticket,
            ticket_holder: *holder,
            claimant_token_account,
            bond_manager: self.manager,
            underlying_token_vault: self.underlying_token_vault,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn borrow_order(
        &self,
        user: &Pubkey,
        ticket_vault: Option<&Pubkey>,
        token_vault: Option<&Pubkey>,
        params: OrderParams,
    ) -> Result<Instruction> {
        let user_ticket_vault = match ticket_vault {
            Some(vault) => *vault,
            None => get_associated_token_address(user, &self.bond_ticket_mint),
        };
        let user_token_vault = match token_vault {
            Some(vault) => *vault,
            None => get_associated_token_address(user, &self.keys.unwrap("underlying_mint")?),
        };
        let data = jet_bonds::instruction::BorrowOrder { params }.data();
        let accounts = jet_bonds::accounts::BorrowOrder {
            user: *user,
            user_ticket_vault,
            user_token_vault,
            bond_manager: self.manager,
            bond_ticket_mint: self.bond_ticket_mint,
            orderbook_market_state: self.orderbook_market_state,
            event_queue: self.keys.unwrap("event_queue")?,
            bids: self.keys.unwrap("bids")?,
            asks: self.keys.unwrap("asks")?,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }
    pub fn lend_order(
        &self,
        user: &Pubkey,
        ticket_vault: Option<&Pubkey>,
        token_vault: Option<&Pubkey>,
        params: OrderParams,
        seed: u64,
    ) -> Result<Instruction> {
        let user_ticket_vault = match ticket_vault {
            Some(vault) => *vault,
            None => get_associated_token_address(user, &self.bond_ticket_mint),
        };
        let user_token_vault = match token_vault {
            Some(vault) => *vault,
            None => get_associated_token_address(user, &self.keys.unwrap("underlying_mint")?),
        };
        let split_ticket = bonds_pda(&[
            jet_bonds::seeds::SPLIT_TICKET,
            user.as_ref(),
            &seed.to_le_bytes(),
        ]);
        let data = jet_bonds::instruction::LendOrder { params, seed }.data();
        let accounts = jet_bonds::accounts::LendOrder {
            user: *user,
            user_ticket_vault,
            user_token_vault,
            bond_manager: self.manager,
            underlying_token_vault: self.underlying_token_vault,
            split_ticket,
            orderbook_market_state: self.orderbook_market_state,
            event_queue: self.keys.unwrap("event_queue")?,
            bids: self.keys.unwrap("bids")?,
            asks: self.keys.unwrap("asks")?,
            payer: self.keys.unwrap("payer")?,
            token_program: spl_token::ID,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn pause_order_matching(&self) -> Result<Instruction> {
        let data = jet_bonds::instruction::PauseOrderMatching {}.data();
        let accounts = jet_bonds::accounts::PauseOrderMatching {
            bond_manager: self.manager,
            orderbook_market_state: self.orderbook_market_state,
            program_authority: self.keys.unwrap("authority")?,
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
            program_authority: self.keys.unwrap("authority")?,
        }
        .to_account_metas(None);

        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn pause_ticket_redemption(&self) -> Result<Instruction> {
        self.modify_manager([true as u8].into(), 8 + 32 * 12 + 8 + 3)
    }
    pub fn resume_ticket_redemption(&self) -> Result<Instruction> {
        self.modify_manager([false as u8].into(), 8 + 32 * 12 + 8 + 3)
    }

    pub fn modify_manager(&self, data: Vec<u8>, offset: usize) -> Result<Instruction> {
        let data = jet_bonds::instruction::ModifyBondManager { data, offset }.data();
        let accounts = jet_bonds::accounts::ModifyBondManager {
            bond_manager: self.manager,
            program_authority: self.keys.unwrap("authority")?,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(jet_bonds::ID, &data, accounts))
    }

    pub fn authorize_crank_instruction(&self) -> Result<Instruction> {
        let data = jet_bonds_metadata::instruction::AuthorizeCrankSigner {}.data();
        let crank = self.keys.unwrap("crank")?;
        let metadata_account = self.crank_metadata_key(&crank);
        let accounts = jet_bonds_metadata::accounts::AuthorizeCrankSigner {
            crank_signer: crank,
            metadata_account,
            authority: self.keys.unwrap("authority")?,
            payer: self.keys.unwrap("payer")?,
            system_program: solana_sdk::system_program::ID,
        }
        .to_account_metas(None);
        Ok(Instruction::new_with_bytes(
            jet_bonds_metadata::ID,
            &data,
            accounts,
        ))
    }
}

impl BondsIxBuilder {
    pub fn bond_manager_key(mint: &Pubkey, seed: u64) -> Pubkey {
        bonds_pda(&[
            jet_bonds::seeds::BOND_MANAGER,
            mint.as_ref(),
            &seed.to_be_bytes(),
        ])
    }
    pub fn claim_ticket_key(&self, ticket_holder: &Pubkey, seed: u64) -> Pubkey {
        bonds_pda(&[
            jet_bonds::seeds::CLAIM_TICKET,
            self.manager.as_ref(),
            ticket_holder.as_ref(),
            &seed.to_le_bytes(),
        ])
    }
    pub fn crank_metadata_key(&self, crank: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[jet_bonds_metadata::seeds::CRANK_SIGNER, crank.as_ref()],
            &jet_bonds_metadata::ID,
        )
        .0
    }

    pub fn jet_bonds_id() -> Pubkey {
        jet_bonds::ID
    }
}

pub fn bonds_pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &jet_bonds::ID).0
}
