use async_trait::async_trait;

use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    transaction::VersionedTransaction,
};

/// A type representing a wallet that can sign messages
#[async_trait(?Send)]
pub trait Wallet {
    /// The public key for the signer
    fn pubkey(&self) -> Option<Pubkey>;

    /// Sign a set of messages
    async fn sign_messages(&self, messages: &[&[u8]]) -> Option<Vec<Signature>>;

    /// Sign a set of transactions
    async fn sign_transactions(
        &self,
        transactions: &[VersionedTransaction],
    ) -> Option<Vec<VersionedTransaction>>;
}

#[async_trait(?Send)]
impl Wallet for Keypair {
    fn pubkey(&self) -> Option<Pubkey> {
        use solana_sdk::signature::Signer;

        Some(Signer::pubkey(self))
    }

    /// Sign a set of messages
    async fn sign_messages(&self, messages: &[&[u8]]) -> Option<Vec<Signature>> {
        use solana_sdk::signature::Signer;

        Some(
            messages
                .iter()
                .map(|message| self.sign_message(message))
                .collect::<Vec<_>>(),
        )
    }

    /// Sign a set of transactions
    async fn sign_transactions(
        &self,
        transactions: &[VersionedTransaction],
    ) -> Option<Vec<VersionedTransaction>> {
        let messages = transactions
            .iter()
            .map(|tx| tx.message.serialize())
            .collect::<Vec<_>>();
        let messages_refs = messages
            .iter()
            .map(|msg| msg.as_slice())
            .collect::<Vec<_>>();

        let signatures = self.sign_messages(&messages_refs).await?;
        let pubkey = self.pubkey()?;

        Some(
            signatures
                .into_iter()
                .zip(transactions.iter())
                .map(|(signature, tx)| {
                    let index = tx
                        .message
                        .static_account_keys()
                        .iter()
                        .position(|key| *key == pubkey)
                        .expect("given transaction has no matching pubkey for the signer");

                    let mut tx = tx.clone();
                    tx.signatures.resize(
                        tx.message.header().num_required_signatures.into(),
                        Default::default(),
                    );
                    tx.signatures[index] = signature;
                    tx
                })
                .collect::<Vec<_>>(),
        )
    }
}
