use async_trait::async_trait;
use std::sync::Arc;

use solana_client_wasm::{
    utils::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig, RpcSendTransactionConfig},
    WasmClient,
};
use solana_extra_wasm::{
    account_decoder::UiAccountEncoding, transaction_status::UiTransactionEncoding,
};
use solana_sdk::{
    account::Account,
    clock::SLOT_MS,
    commitment_config::{CommitmentConfig, CommitmentLevel},
    hash::Hash,
    pubkey::Pubkey,
    signature::Signature,
};
use solana_transaction_status::{TransactionConfirmationStatus, TransactionStatus};
use spl_token::state::Account as TokenAccount;

use super::{AccountFilter, ClientError, ClientResult, SolanaRpc};

/// A wrapper for an RPC client to implement `SolanaRpc` trait
#[derive(Clone)]
pub struct RpcConnection {
    rpc: Arc<WasmClient>,
}

impl RpcConnection {
    pub fn new(url: &str) -> Self {
        Self {
            rpc: Arc::new(WasmClient::new(url)),
        }
    }
}

impl From<WasmClient> for RpcConnection {
    fn from(rpc: WasmClient) -> Self {
        Self { rpc: Arc::new(rpc) }
    }
}

#[async_trait]
impl SolanaRpc for RpcConnection {
    async fn get_genesis_hash(&self) -> ClientResult<Hash> {
        self.rpc.get_genesis_hash().await.map_err(convert_err)
    }

    async fn get_latest_blockhash(&self) -> ClientResult<Hash> {
        self.rpc.get_latest_blockhash().await.map_err(convert_err)
    }

    async fn get_slot(&self) -> ClientResult<u64> {
        self.rpc.get_slot().await.map_err(convert_err)
    }

    async fn get_block_time(&self, slot: u64) -> ClientResult<i64> {
        self.rpc.get_block_time(slot).await.map_err(convert_err)
    }

    async fn get_account(&self, address: &Pubkey) -> ClientResult<Option<Account>> {
        self.rpc
            .get_account_with_commitment(address, CommitmentConfig::processed())
            .await
            .map_err(convert_err)
    }

    async fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> ClientResult<Vec<Option<Account>>> {
        let slot = self.get_slot().await?;

        self.rpc
            .get_multiple_accounts_with_config(
                pubkeys,
                RpcAccountInfoConfig {
                    min_context_slot: Some(slot),
                    commitment: Some(CommitmentConfig::processed()),
                    ..Default::default()
                },
            )
            .await
            .map_err(convert_err)
    }

    async fn get_signature_statuses(
        &self,
        signatures: &[Signature],
    ) -> ClientResult<Vec<Option<solana_transaction_status::TransactionStatus>>> {
        use solana_extra_wasm::transaction_status::TransactionConfirmationStatus as WasmConfirmationStatus;

        self.rpc
            .get_signature_statuses(signatures)
            .await
            .map_err(convert_err)
            .map(|statuses| {
                statuses
                    .into_iter()
                    .map(|status| {
                        status.map(|value| TransactionStatus {
                            status: match &value.err {
                                None => Ok(()),
                                Some(err) => Err(err.clone()),
                            },
                            confirmations: value.confirmations.map(|c| c as usize),
                            err: value.err,
                            confirmation_status: value.confirmation_status.map(|cs| match cs {
                                WasmConfirmationStatus::Finalized => {
                                    TransactionConfirmationStatus::Finalized
                                }
                                WasmConfirmationStatus::Confirmed => {
                                    TransactionConfirmationStatus::Confirmed
                                }
                                WasmConfirmationStatus::Processed => {
                                    TransactionConfirmationStatus::Processed
                                }
                            }),
                            slot: value.slot,
                        })
                    })
                    .collect::<Vec<_>>()
            })
    }

    async fn airdrop(&self, account: &Pubkey, lamports: u64) -> ClientResult<()> {
        let signature = self
            .rpc
            .request_airdrop(account, lamports)
            .await
            .map_err(convert_err)?;

        while self
            .rpc
            .get_signature_statuses(&[signature])
            .await
            .map_err(convert_err)?[0]
            .is_none()
        {
            tokio::time::sleep(std::time::Duration::from_millis(SLOT_MS)).await;
        }

        Ok(())
    }

    async fn send_transaction_legacy(
        &self,
        transaction: &solana_sdk::transaction::Transaction,
    ) -> ClientResult<Signature> {
        self.rpc
            .send_transaction_with_config(
                transaction,
                RpcSendTransactionConfig {
                    skip_preflight: false,
                    preflight_commitment: Some(CommitmentLevel::Processed),
                    encoding: Some(UiTransactionEncoding::Base64),
                    ..Default::default()
                },
            )
            .await
            .map_err(convert_err)
    }

    async fn send_transaction(
        &self,
        _transaction: &solana_sdk::transaction::VersionedTransaction,
    ) -> ClientResult<Signature> {
        unimplemented!()
    }

    async fn get_program_accounts(
        &self,
        program: &Pubkey,
        filters: &[AccountFilter],
    ) -> ClientResult<Vec<(Pubkey, solana_sdk::account::Account)>> {
        use solana_client_wasm::utils::rpc_filter::*;

        let config = RpcProgramAccountsConfig {
            filters: Some(
                filters
                    .iter()
                    .map(|filter| match filter {
                        AccountFilter::Memcmp { offset, bytes } => RpcFilterType::Memcmp(Memcmp {
                            offset: *offset,
                            bytes: MemcmpEncodedBytes::Bytes(bytes.clone()),
                            encoding: None,
                        }),
                        AccountFilter::DataSize(size) => RpcFilterType::DataSize(*size as u64),
                    })
                    .collect(),
            ),
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                data_slice: None,
                commitment: Some(CommitmentConfig::processed()),
                min_context_slot: None,
            },
            with_context: None,
        };

        self.rpc
            .get_program_accounts_with_config(program, config)
            .await
            .map_err(convert_err)
    }

    async fn get_token_accounts_by_owner(
        &self,
        _owner: &Pubkey,
    ) -> Result<Vec<(Pubkey, TokenAccount)>, ClientError> {
        unimplemented!()
    }
}

fn convert_err(e: solana_client_wasm::ClientError) -> ClientError {
    ClientError::Other(e.to_string())
}
