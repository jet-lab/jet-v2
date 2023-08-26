use async_trait::async_trait;
use serde_json::json;
use std::{str::FromStr, sync::Arc};

use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::{
        RpcAccountInfoConfig, RpcProgramAccountsConfig, RpcSendTransactionConfig,
        RpcTokenAccountsFilter,
    },
    rpc_request::{RpcError, RpcRequest, RpcResponseErrorData},
    rpc_response::{Response, RpcKeyedAccount},
};
use solana_sdk::{
    account::Account,
    clock::SLOT_MS,
    commitment_config::{CommitmentConfig, CommitmentLevel},
    hash::Hash,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::Signature,
};
use spl_token::state::Account as TokenAccount;

use super::{
    AccountFilter, ClientError, ClientResult, SolanaRpc, TransactionConfirmationStatus,
    TransactionStatus,
};

/// A wrapper for an RPC client to implement `SolanaRpc` trait
#[derive(Clone)]
pub struct RpcConnection {
    rpc: Arc<RpcClient>,
    send_config: RpcSendTransactionConfig,
}

impl RpcConnection {
    pub fn new(url: &str) -> Self {
        Self {
            rpc: Arc::new(RpcClient::new(url.to_owned())),
            send_config: Default::default(),
        }
    }
    /// Optimistic = assume there is no risk. so we don't need:
    /// - finality (processed can be trusted)
    /// - preflight checks (not worried about losing sol)
    ///
    /// This is desirable for testing because:
    /// - tests can run faster (never need to wait for finality)
    /// - validator logs are more comprehensive (preflight checks obscure error logs)
    /// - there is nothing at stake in a local test validator
    pub fn new_optimistic(url: &str) -> Self {
        Self {
            rpc: Arc::new(RpcClient::new_with_commitment(
                url.to_owned(),
                CommitmentConfig::processed(),
            )),
            send_config: RpcSendTransactionConfig {
                skip_preflight: true,
                preflight_commitment: Some(CommitmentLevel::Processed),
                ..Default::default()
            },
        }
    }
}

impl From<RpcClient> for RpcConnection {
    fn from(rpc: RpcClient) -> Self {
        Self {
            rpc: Arc::new(rpc),
            send_config: Default::default(),
        }
    }
}

impl std::fmt::Debug for RpcConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RpcConnection")
            .field("url", &self.rpc.url())
            .finish()
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
            .map(|r| r.value)
    }

    async fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> ClientResult<Vec<Option<Account>>> {
        self.rpc
            .get_multiple_accounts_with_config(
                pubkeys,
                RpcAccountInfoConfig {
                    min_context_slot: None,
                    commitment: Some(CommitmentConfig::processed()),
                    ..Default::default()
                },
            )
            .await
            .map_err(convert_err)
            .map(|r| r.value)
    }

    async fn get_signature_statuses(
        &self,
        signatures: &[Signature],
    ) -> ClientResult<Vec<Option<TransactionStatus>>> {
        self.rpc
            .get_signature_statuses(signatures)
            .await
            .map_err(convert_err)
            .map(|r| r.value.into_iter()
            .map(|s| s.map(|s| TransactionStatus {
                slot: s.slot,
                err: s.err,
                status: s.status,
                confirmations: s.confirmations,
                confirmation_status: s.confirmation_status.map(|cs| match cs {
                    solana_transaction_status::TransactionConfirmationStatus::Processed => TransactionConfirmationStatus::Processed,
                    solana_transaction_status::TransactionConfirmationStatus::Confirmed => TransactionConfirmationStatus::Confirmed,
                    solana_transaction_status::TransactionConfirmationStatus::Finalized => TransactionConfirmationStatus::Finalized,
                })
            })).collect()
        )
    }

    async fn airdrop(&self, account: &Pubkey, lamports: u64) -> ClientResult<()> {
        let signature = self
            .rpc
            .request_airdrop(account, lamports)
            .await
            .map_err(convert_err)?;

        while self
            .rpc
            .get_signature_status(&signature)
            .await
            .map_err(convert_err)?
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
            .send_transaction_with_config(transaction, self.send_config)
            .await
            .map_err(convert_err)
    }

    async fn send_transaction(
        &self,
        transaction: &solana_sdk::transaction::VersionedTransaction,
    ) -> ClientResult<Signature> {
        self.rpc
            .send_transaction_with_config(transaction, self.send_config)
            .await
            .map_err(convert_err)
    }

    async fn get_program_accounts(
        &self,
        program: &Pubkey,
        filters: &[AccountFilter],
    ) -> ClientResult<Vec<(Pubkey, solana_sdk::account::Account)>> {
        use solana_client::rpc_filter::*;

        let config = RpcProgramAccountsConfig {
            filters: Some(
                filters
                    .iter()
                    .map(|filter| match filter {
                        AccountFilter::Memcmp { offset, bytes } => {
                            RpcFilterType::Memcmp(Memcmp::new_raw_bytes(*offset, bytes.clone()))
                        }
                        AccountFilter::DataSize(size) => RpcFilterType::DataSize(*size as u64),
                    })
                    .collect(),
            ),
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64Zstd),
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
        owner: &Pubkey,
    ) -> Result<Vec<(Pubkey, TokenAccount)>, ClientError> {
        let token_account_filter = RpcTokenAccountsFilter::ProgramId(spl_token::ID.to_string());

        let config = RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64Zstd),
            commitment: Some(CommitmentConfig::processed()),
            data_slice: None,
            min_context_slot: None,
        };

        let accounts: Response<Vec<RpcKeyedAccount>> = self
            .rpc
            .send(
                RpcRequest::GetTokenAccountsByOwner,
                json!([owner.to_string(), token_account_filter, config]),
            )
            .await
            .map_err(convert_err)?;

        let mut token_accounts = vec![];

        for account in accounts.value {
            let address = Pubkey::from_str(&account.pubkey).map_err(|_| {
                ClientError::InvalidResponse(format!(
                    "cannot read public key value from get_token_accounts_by_owner: '{}'",
                    account.pubkey
                ))
            })?;

            let data = account
                .account
                .decode::<Account>()
                .ok_or(ClientError::InvalidResponse(format!(
                    "cannot read account data from get_token_accounts_by_owner: '{:?}'",
                    account.account
                )))?;

            let token_account_data = TokenAccount::unpack(&data.data).map_err(|e| {
                ClientError::InvalidResponse(format!(
                    "cannot parse token account data from get_token_accounts_by_owner: '{:?}'",
                    e
                ))
            })?;

            token_accounts.push((address, token_account_data));
        }

        Ok(token_accounts)
    }
}

fn convert_err(e: solana_client::client_error::ClientError) -> ClientError {
    match e.kind {
        solana_client::client_error::ClientErrorKind::TransactionError(e) => {
            ClientError::TransactionError(e)
        }
        solana_client::client_error::ClientErrorKind::RpcError(RpcError::RpcResponseError {
            data: RpcResponseErrorData::SendTransactionPreflightFailure(failure),
            ..
        }) => ClientError::TransactionSimulationError {
            err: failure.err,
            logs: failure.logs.unwrap_or_default(),
        },
        _ => ClientError::Other(e.to_string()),
    }
}
