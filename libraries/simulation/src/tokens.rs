// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright (C) 2022 JET PROTOCOL HOLDINGS, LLC.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::io::Write;
use std::sync::Arc;

use anyhow::{bail, Error};
use bytemuck::Zeroable;

use pyth_client::PriceStatus;
use solana_sdk::instruction::Instruction;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use solana_sdk::{system_instruction, system_program};

use anchor_lang::{InstructionData, ToAccountMetas};

use jet_solana_rpc_api::SolanaRpcClient;

use crate::send_and_confirm;

/// Utility for managing the creation of tokens and their prices
/// in some kind of testing environment
pub struct TokenManager {
    rpc: Arc<dyn SolanaRpcClient>,
}

impl TokenManager {
    pub fn new(rpc: Arc<dyn SolanaRpcClient>) -> Self {
        Self { rpc }
    }

    /// Create a new token mint, with optional mint and freeze authorities.
    ///
    /// # Params
    ///
    /// `decimals` - the number of decimal places the mint should have
    /// `mint_authority` - optional authority to mint tokens, defaults to the payer
    /// `freeze_authority` - optional authority to freeze tokens, has no default
    pub async fn create_token(
        &self,
        decimals: u8,
        mint_authority: Option<&Pubkey>,
        freeze_authority: Option<&Pubkey>,
    ) -> Result<Pubkey, Error> {
        let keypair = crate::generate_keypair();
        let payer = self.rpc.payer();
        let space = spl_token::state::Mint::LEN;
        let rent_lamports = self
            .rpc
            .get_minimum_balance_for_rent_exemption(space)
            .await?;

        let ix_create_account = system_instruction::create_account(
            &payer.pubkey(),
            &keypair.pubkey(),
            rent_lamports,
            space as u64,
            &spl_token::ID,
        );

        let ix_initialize = spl_token::instruction::initialize_mint(
            &spl_token::ID,
            &keypair.pubkey(),
            mint_authority.unwrap_or(&payer.pubkey()),
            freeze_authority,
            decimals,
        )?;

        send_and_confirm(&self.rpc, &[ix_create_account, ix_initialize], &[&keypair]).await?;

        Ok(keypair.pubkey())
    }

    /// Create a new token account belonging to the owner, with the supplied mint
    pub async fn create_account(&self, mint: &Pubkey, owner: &Pubkey) -> Result<Pubkey, Error> {
        let keypair = crate::generate_keypair();
        let payer = self.rpc.payer();
        let space = spl_token::state::Account::LEN;
        let rent_lamports = self
            .rpc
            .get_minimum_balance_for_rent_exemption(space)
            .await?;

        let ix_create_account = system_instruction::create_account(
            &payer.pubkey(),
            &keypair.pubkey(),
            rent_lamports,
            space as u64,
            &spl_token::ID,
        );

        let ix_initialize = spl_token::instruction::initialize_account(
            &spl_token::ID,
            &keypair.pubkey(),
            mint,
            owner,
        )?;

        send_and_confirm(&self.rpc, &[ix_create_account, ix_initialize], &[&keypair]).await?;

        Ok(keypair.pubkey())
    }

    /// Create a new token account with some initial balance
    pub async fn create_account_funded(
        &self,
        mint: &Pubkey,
        owner: &Pubkey,
        amount: u64,
    ) -> Result<Pubkey, Error> {
        let account = self.create_account(mint, owner).await?;

        self.mint(mint, &account, amount).await?;
        Ok(account)
    }

    /// Create oracle accounts for a token
    pub async fn create_oracle(&self, mint: &Pubkey) -> Result<TokenOracle, Error> {
        let payer = self.rpc.payer();
        let (price_address, price_bump) = Pubkey::find_program_address(
            &[mint.as_ref(), b"oracle:price".as_ref()],
            &jet_metadata::ID,
        );
        let (product_address, product_bump) = Pubkey::find_program_address(
            &[mint.as_ref(), b"oracle:product".as_ref()],
            &jet_metadata::ID,
        );

        let ix_create = |address, _, seed| Instruction {
            program_id: jet_metadata::ID,
            accounts: jet_metadata::accounts::CreateEntry {
                key_account: *mint,
                metadata_account: address,
                authority: Self::get_authority_address(),
                payer: payer.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: jet_metadata::instruction::CreateEntry {
                seed,
                space: std::mem::size_of::<pyth_client::Price>() as u64,
            }
            .data(),
        };

        let ix_create_price = ix_create(price_address, price_bump, "oracle:price".to_string());
        let ix_create_product =
            ix_create(product_address, product_bump, "oracle:product".to_string());

        send_and_confirm(&self.rpc, &[ix_create_price, ix_create_product], &[]).await?;

        let mut product_account = pyth_client::Product {
            ver: pyth_client::VERSION,
            magic: pyth_client::MAGIC,
            size: std::mem::size_of::<pyth_client::Price>() as u32,
            atype: pyth_client::AccountType::Product as u32,
            px_acc: pyth_client::AccKey {
                val: price_address.to_bytes(),
            },
            attr: [0u8; pyth_client::PROD_ATTR_SIZE],
        };

        write_pyth_product_attributes(
            &mut product_account.attr,
            &[("asset_type", "Crypto"), ("quote_currency", "USD")],
        );

        self.set_pod_metadata(&product_address, &product_account)
            .await?;
        self.set_pod_metadata(&price_address, &default_price())
            .await?;

        Ok(TokenOracle {
            price: price_address,
            product: product_address,
        })
    }

    /// Mint tokens to an account
    pub async fn mint(
        &self,
        mint: &Pubkey,
        destination: &Pubkey,
        amount: u64,
    ) -> Result<(), Error> {
        let payer = self.rpc.payer();

        send_and_confirm(
            &self.rpc,
            &[spl_token::instruction::mint_to(
                &spl_token::ID,
                mint,
                destination,
                &payer.pubkey(),
                &[],
                amount,
            )?],
            &[],
        )
        .await?;

        Ok(())
    }

    /// Set the oracle price of a token
    pub async fn set_price(&self, mint: &Pubkey, price: &TokenPrice) -> Result<(), Error> {
        let mut price_data = default_price();

        let price_value = jet_proto_math::Number128::from_decimal(price.price, price.exponent)
            .as_u64(price_data.expo) as i64;
        let twap_value = jet_proto_math::Number128::from_decimal(price.twap, price.exponent)
            .as_u64(price_data.expo) as i64;

        price_data.agg.price = price_value;
        price_data.agg.conf = price.confidence;
        price_data.agg.status = PriceStatus::Trading;
        price_data.twap.val = twap_value;

        let (price_address, _) = Pubkey::find_program_address(
            &[mint.as_ref(), b"oracle:price".as_ref()],
            &jet_metadata::ID,
        );

        self.set_pod_metadata(&price_address, &price_data).await
    }

    /// Get the current balance of a token account
    pub async fn get_balance(&self, account: &Pubkey) -> Result<u64, Error> {
        let account_data = self.rpc.get_account(account).await?;

        if account_data.is_none() {
            bail!("account {} does not exist", account);
        }

        let state = spl_token::state::Account::unpack(&account_data.unwrap().data)?;

        Ok(state.amount)
    }

    async fn set_pod_metadata<T: bytemuck::Pod>(
        &self,
        address: &Pubkey,
        value: &T,
    ) -> Result<(), Error> {
        let mut data = vec![0u8; std::mem::size_of::<T>()];
        (&mut data[..])
            .write_all(bytemuck::bytes_of(value))
            .unwrap();

        // FIXME: allow more than 512 bytes
        data.resize(std::cmp::min(512, data.len()), 0);

        let ix_write = Instruction {
            program_id: jet_metadata::ID,
            accounts: jet_metadata::accounts::SetEntry {
                metadata_account: *address,
                authority: Self::get_authority_address(),
            }
            .to_account_metas(None),
            data: jet_metadata::instruction::SetEntry { data, offset: 0 }.data(),
        };

        send_and_confirm(&self.rpc, &[ix_write], &[]).await?;

        Ok(())
    }

    /// Get the authority address of the Jet Control program.
    fn get_authority_address() -> Pubkey {
        Pubkey::find_program_address(&[], &jet_control::ID).0
    }
}

#[derive(Clone, Copy)]
pub struct TokenOracle {
    pub price: Pubkey,
    pub product: Pubkey,
}

#[derive(Clone, Copy)]
pub struct TokenPrice {
    pub exponent: i32,
    pub price: i64,
    pub confidence: u64,
    pub twap: u64,
}

fn write_pyth_product_attributes(mut storage: &mut [u8], attributes: &[(&str, &str)]) {
    for (key, value) in attributes {
        storage.write_all(&[key.len() as u8]).unwrap();
        storage.write_all(key.as_bytes()).unwrap();
        storage.write_all(&[value.len() as u8]).unwrap();
        storage.write_all(value.as_bytes()).unwrap();
    }
}

fn default_price() -> pyth_client::Price {
    pyth_client::Price {
        ver: pyth_client::VERSION,
        magic: pyth_client::MAGIC,
        atype: pyth_client::AccountType::Price as u32,
        size: std::mem::size_of::<pyth_client::Price>() as u32,
        expo: -8,
        next: pyth_client::AccKey { val: [0u8; 32] },
        ptype: pyth_client::PriceType::Price,
        ..pyth_client::Price::zeroed()
    }
}
