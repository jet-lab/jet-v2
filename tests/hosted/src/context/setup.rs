use anyhow::Result;
use jet_client::config::JetAppConfig;
use jet_environment::builder::{
    configure_environment, configure_tokens, create_test_tokens, PlanInstructions,
};
use jet_instructions::test_service::{derive_token_mint, token_update_pyth_price};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature, Signer};

use jet_environment::config::TokenDescription;
use jet_margin_sdk::solana::keypair::KeypairExt;
use jet_margin_sdk::solana::transaction::{
    InverseSendTransactionBuilder, SendTransactionBuilder, TransactionBuilderExt,
};
use jet_simulation::Keygen;
use jet_solana_client::transaction::WithSigner;

use crate::margin::MarginUser;

use super::{MarginTestContext, TestContextSetupInfo};

impl MarginTestContext {
    /// Create the airspace plus all tokens, pools, swaps, and markets.
    ///
    /// setup options:
    /// - Default::default(): a blank airspace
    /// - TestDefault::test_default(): airspace with two tokens and their pools
    pub async fn init_environment(&self, setup: &TestContextSetupInfo) -> Result<JetAppConfig> {
        let setup_config = setup.to_config(
            &self.airspace_name,
            self.solana.rpc.payer().pubkey(),
            self.crank.pubkey(),
        );
        let mut builder = self.env_builder();
        configure_environment(&mut builder, &setup_config.env_config)
            .await
            .unwrap();
        self.execute_plan(builder.build()).await?;

        Ok(setup_config.app_config)
    }

    async fn execute_plan(&self, plan: PlanInstructions) -> Result<()> {
        plan.setup
            .send_and_confirm_condensed(&self.solana.rpc)
            .await?;
        plan.propose
            .into_iter()
            .map(|tx| tx.with_signer(self.airspace_authority.clone()))
            .collect::<Vec<_>>()
            .send_and_confirm_condensed_in_order(&self.solana.rpc)
            .await?;
        // cannot use `interface` here to send the propose transactions since
        // the SimulationClient does not properly handle signatures other than
        // the payer. My guess is that existing signatures don't translate when
        // converting from VersionedTransaction to legacy transactions.

        Ok(())
    }

    pub fn generate_key(&self) -> Keypair {
        self.solana.keygen.generate_key()
    }

    pub async fn create_wallet(&self, sol_amount: u64) -> Result<Keypair> {
        self.solana.create_wallet(sol_amount).await
    }

    pub async fn issue_permit(&self, user: Pubkey) -> Result<Signature> {
        self.airspace_ix()
            .permit_create(user)
            .with_signer(self.airspace_authority.clone())
            .send_and_confirm(&self.solana.rpc)
            .await
    }

    pub async fn create_margin_user(&self, sol_amount: u64) -> Result<MarginUser> {
        let wallet = self.solana.create_wallet(sol_amount).await?;
        self.issue_permit(wallet.pubkey()).await?;
        self.margin_client().user(&wallet, 0).created().await
    }

    /// Generate a new wallet keypair for a liquidator with the pubkey that
    /// stores the [LiquidatorMetadata]
    pub async fn create_liquidator(&self, sol_amount: u64) -> Result<Keypair> {
        let liquidator = self.solana.create_wallet(sol_amount).await?;

        self.margin_client()
            .set_liquidator_metadata(liquidator.pubkey(), true)
            .await?;
        Ok(liquidator)
    }

    pub async fn create_token(&self, mut token: TokenDescription) -> Result<Pubkey> {
        token.name = format!("{}-{}", self.airspace_name, &token.name);
        let mut builder = self.env_builder();
        let oracle_authority = &self.solana.rpc.payer().pubkey();
        create_test_tokens(&mut builder, oracle_authority, vec![&token])
            .await
            .unwrap();
        configure_tokens(
            &mut builder,
            &self.airspace,
            &[self.crank.pubkey()],
            oracle_authority,
            vec![&token],
        )
        .await
        .unwrap();
        self.execute_plan(builder.build()).await?;

        Ok(derive_token_mint(&token.name))
    }

    /// This manages oracles with the test service. Some other code uses the
    /// metadata program. These two approaches are not compatible.
    pub async fn set_price(&self, mint: &Pubkey, price: f64, confidence: f64) -> Result<()> {
        let exponent = -7;
        let one = 10_000_000.0;
        let price = (one * price).round() as i64;
        let confidence = (one * confidence).round() as i64;
        self.update_price(
            mint,
            &PriceUpdate {
                price,
                confidence,
                exponent,
            },
        )
        .await
    }

    /// This manages oracles with the test service. Some other code uses the
    /// metadata program. These two approaches are not compatible.
    pub async fn update_price(&self, mint: &Pubkey, update: &PriceUpdate) -> Result<()> {
        let ix = token_update_pyth_price(
            &self.solana.rpc.payer().pubkey(),
            mint,
            update.price,
            update.confidence,
            update.exponent,
        );

        self.solana.rpc.send_and_confirm(ix.into()).await?;
        Ok(())
    }
}

pub struct PriceUpdate {
    pub price: i64,
    pub confidence: i64,
    pub exponent: i32,
}
