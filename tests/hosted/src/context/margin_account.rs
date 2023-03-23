use anchor_lang::prelude::Pubkey;
use jet_instructions::{
    margin::{refresh_deposit_position, MarginIxBuilder},
    test_service::derive_pyth_price,
};
use jet_solana_client::{
    signature::Authorization,
    transaction::{TransactionBuilder, WithSigner},
};
use solana_sdk::signer::Signer;
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;

use super::MarginTestContext;

impl MarginTestContext {
    /// Create and register the token account and position if missing.
    pub fn register_deposit_position(
        &self,
        mint: Pubkey,
        margin_account: Authorization,
    ) -> Vec<TransactionBuilder> {
        register_deposit_position(mint, margin_account, self.airspace, self.payer().pubkey())
    }

    pub fn refresh_deposit(&self, mint: Pubkey, margin_account: Pubkey) -> TransactionBuilder {
        refresh_deposit(mint, margin_account, &self.airspace)
    }
}

pub(super) fn register_deposit_position(
    mint: Pubkey,
    margin_account: Authorization,
    airspace: Pubkey,
    payer: Pubkey,
) -> Vec<TransactionBuilder> {
    let create_ata = create_associated_token_account_idempotent(
        &payer,
        &margin_account.address,
        &mint,
        &spl_token::id(),
    );
    let builder = MarginIxBuilder::new_for_address(airspace, margin_account.address, payer)
        .with_authority(margin_account.authority.pubkey());
    let register = builder
        .create_deposit_position(mint)
        .with_signer(margin_account.authority);

    vec![create_ata.into(), register]
}

pub(super) fn refresh_deposit(
    mint: Pubkey,
    margin_account: Pubkey,
    airspace: &Pubkey,
) -> TransactionBuilder {
    refresh_deposit_position(airspace, margin_account, mint, derive_pyth_price(&mint)).into()
}
