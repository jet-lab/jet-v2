//! Token administration helper functions for tests.  

use anyhow::Result;
use jet_environment::builder::{configure_tokens, create_test_tokens};
use jet_environment::config::TokenDescription;
use jet_instructions::test_service::{token_request, token_update_pyth_price};
use jet_margin_sdk::cat;
use jet_margin_sdk::solana::keypair::KeypairExt;
use jet_margin_sdk::solana::transaction::TransactionBuilderExt;
use jet_solana_client::signature::Authorization;
use jet_solana_client::transaction::{TransactionBuilder, WithSigner};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use spl_associated_token_account::get_associated_token_address;

use crate::actions::Token;
use crate::context::margin_account::register_deposit_position;
use crate::environment::TestToken;
use crate::TestDefault;

use super::MarginTestContext;

/// Uses assumptions about a margin test context to construct inputs to the
/// transaction builder functions below.
impl MarginTestContext {
    pub fn mint_authority(&self) -> Keypair {
        // this cannot be easily changed since the init function uses
        // airspace_authority as the authority for everything except oracles
        self.airspace_authority.clone()
    }

    pub fn oracle_authority(&self) -> Keypair {
        self.rpc().payer().clone()
    }

    /// Just create a token. no name, no pools, no fixed term
    pub async fn basic_token(&self, price: f64) -> Result<(Token, TokenDescription)> {
        let name = self.generate_key().pubkey().to_string()[0..4].to_string();
        self.create_token(TestToken::new(&name).into(), price).await
    }

    pub async fn create_token(
        &self,
        mut desc: TokenDescription,
        price: f64,
    ) -> Result<(Token, TokenDescription)> {
        desc.name = format!("{}-{}", self.airspace_name, &desc.name);
        let mut builder = self.env_builder();
        let oracle_authority = &self.oracle_authority().pubkey();
        create_test_tokens(&mut builder, oracle_authority, vec![&desc])
            .await
            .unwrap();
        configure_tokens(
            &mut builder,
            &self.airspace,
            &[self.crank.pubkey()],
            oracle_authority,
            vec![&desc],
        )
        .await
        .unwrap();
        self.execute_plan(builder.build()).await?;
        let token = Token::from(desc.clone());
        self.set_price(token.mint, price)
            .send_and_confirm(&self.rpc())
            .await?;

        Ok((token, desc))
    }

    /// Airdrop tokens into an ATA owned by a margin account, creating and
    /// registering the token account and position if missing.
    pub fn margin_airdrop(
        &self,
        mint: Pubkey,
        margin_account: Authorization,
        amount: u64,
    ) -> Vec<TransactionBuilder> {
        margin_airdrop(
            Authorization {
                address: mint,
                authority: self.mint_authority(),
            },
            margin_account,
            self.airspace,
            self.payer().pubkey(),
            amount,
        )
    }

    /// This manages oracles with the test service. Some other code uses the
    /// metadata program. These two approaches are not compatible.
    pub fn set_price(&self, mint: Pubkey, price: f64) -> TransactionBuilder {
        self.update_price(mint, &PriceUpdate::test_default().with_price(price))
    }

    /// This manages oracles with the test service. Some other code uses the
    /// metadata program. These two approaches are not compatible.
    pub fn update_price(&self, mint: Pubkey, update: &PriceUpdate) -> TransactionBuilder {
        update_price(
            Authorization {
                address: mint,
                authority: self.oracle_authority(),
            },
            update,
        )
    }
}

/// Airdrop tokens into an ATA owned by a margin account, creating and
/// registering the token account and position if missing.
fn margin_airdrop(
    mint: Authorization,
    margin_account: Authorization,
    airspace: Pubkey,
    payer: Pubkey,
    amount: u64,
) -> Vec<TransactionBuilder> {
    let ata = get_associated_token_address(&margin_account.address, &mint.address);
    let register = register_deposit_position(mint.address, margin_account, airspace, payer);
    let airdrop = airdrop_token(mint, ata, amount);

    cat![register, vec![airdrop]]
}

/// Airdrop tokens into an existing account
fn airdrop_token(mint: Authorization, destination: Pubkey, amount: u64) -> TransactionBuilder {
    token_request(
        &mint.authority.pubkey(),
        &mint.address,
        &destination,
        amount,
    )
    .with_signer(&mint.authority)
}

/// This manages oracles with the test service. Some other code uses the
/// metadata program. These two approaches are not compatible.
fn update_price(mint: Authorization, update: &PriceUpdate) -> TransactionBuilder {
    token_update_pyth_price(
        &mint.authority.pubkey(),
        &mint.address,
        update.price,
        update.confidence,
        update.exponent,
    )
    .with_signer(&mint.authority)
}

pub struct PriceUpdate {
    pub price: i64,
    pub confidence: i64,
    pub exponent: i32,
}

impl PriceUpdate {
    pub fn with_price(mut self, price: f64) -> Self {
        self.price = (price * self.one() as f64) as i64;
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = (confidence * self.one() as f64) as i64;
        self
    }

    pub fn one(&self) -> i64 {
        10i64.pow(
            (-self.exponent)
                .try_into()
                .expect("exponent must be negative"),
        )
    }
}

impl TestDefault for PriceUpdate {
    fn test_default() -> Self {
        Self {
            price: 10_000_000,
            confidence: 0,
            exponent: -7,
        }
    }
}
