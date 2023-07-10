use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use lookup_table_registry::RegistryAccount;
use lookup_table_registry_client::instructions::InstructionBuilder;
use solana_address_lookup_table_program::state::{AddressLookupTable, LOOKUP_TABLE_MAX_ADDRESSES};
use thiserror::Error;

use solana_sdk::{
    address_lookup_table_account::AddressLookupTableAccount, instruction::Instruction,
    pubkey::Pubkey, signature::Keypair, signer::Signer,
};

use jet_environment::builder::LookupScope;
use jet_solana_client::{
    rpc::{SolanaRpc, SolanaRpcExtra},
    transaction::{create_unsigned_transaction, sign_transaction},
};

#[derive(Error, Debug)]
pub enum CreateLookupTableError {
    #[error("rpc client error: {0}")]
    Client(#[from] jet_solana_client::rpc::ClientError),

    #[error("tx build error: {0}")]
    Transaction(#[from] jet_solana_client::transaction::TransactionBuildError),
}

type Result<T> = std::result::Result<T, CreateLookupTableError>;

pub async fn create_lookup_tables(
    rpc: &Arc<dyn SolanaRpc>,
    signer: &Keypair,
    payer: &Keypair,
    config: &HashMap<LookupScope, HashSet<Pubkey>>,
) -> Result<()> {
    LookupTableManager {
        rpc: rpc.clone(),
        signer,
        payer,
        config,
        ix: InstructionBuilder::new(signer.pubkey(), payer.pubkey()),
    }
    .create_all()
    .await
}

struct LookupTableManager<'a> {
    rpc: Arc<dyn SolanaRpc>,
    signer: &'a Keypair,
    payer: &'a Keypair,
    config: &'a HashMap<LookupScope, HashSet<Pubkey>>,
    ix: InstructionBuilder,
}

impl<'a> LookupTableManager<'a> {
    async fn create_all(&self) -> Result<()> {
        let registered = self.create_registry().await?;
        log::debug!("existing lookup tables:");

        for table in &registered {
            log::debug!(
                "existing table {} with {} addressses",
                table.key,
                table.addresses.len()
            );
        }

        let missing = self.resolve_missing(&registered);
        log::debug!("missing addresses:");

        for (scope, missing) in &missing {
            log::debug!("{scope:?} missing {missing:?}");
        }

        self.append_missing(&registered, missing).await
    }

    async fn append_missing(
        &self,
        registered: &[AddressLookupTableAccount],
        missing: HashMap<LookupScope, Vec<Pubkey>>,
    ) -> Result<()> {
        for (scope, mut to_insert) in missing {
            let program = program_for_scope(scope);
            let mut next_free_table = registered.iter().find(|t| {
                t.addresses[0] == program && t.addresses.len() < LOOKUP_TABLE_MAX_ADDRESSES
            });

            while !to_insert.is_empty() {
                match next_free_table {
                    Some(table) => {
                        // There's already a free table with enough space for more addresses, so
                        // try to continue appending to it
                        let free = LOOKUP_TABLE_MAX_ADDRESSES - table.addresses.len();
                        let to_drain = std::cmp::min(free, to_insert.len());
                        let insertable = to_insert.drain(..to_drain).collect::<Vec<_>>();

                        self.send_transaction(&[self.ix.append_to_lookup_table(
                            table.key,
                            &insertable,
                            0,
                        )])
                        .await?;

                        // Append any remaining addresses to a new table
                        next_free_table = None;
                    }

                    None => {
                        // There's no free table that can hold more addresses, so create a
                        // a new one
                        let recent_slot = self.rpc.get_slot().await?;
                        let (ix, key) = self.ix.create_lookup_table(recent_slot, 0);

                        self.send_transaction(&[ix]).await?;

                        let to_drain = std::cmp::min(LOOKUP_TABLE_MAX_ADDRESSES, to_insert.len());
                        let insertable = to_insert.drain(..to_drain).collect::<Vec<_>>();

                        self.send_transaction(&[self.ix.append_to_lookup_table(
                            key,
                            &insertable,
                            0,
                        )])
                        .await?;
                    }
                }
            }
        }
        Ok(())
    }

    fn resolve_missing(
        &self,
        registered: &[AddressLookupTableAccount],
    ) -> HashMap<LookupScope, Vec<Pubkey>> {
        self.config
            .iter()
            .map(|(scope, addresses)| {
                let program = program_for_scope(*scope);
                let existing = registered
                    .iter()
                    .filter(|r| r.addresses[0] == program)
                    .flat_map(|r| r.addresses.iter().cloned())
                    .collect();

                (*scope, addresses.difference(&existing).cloned().collect())
            })
            .collect()
    }

    async fn create_registry(&self) -> Result<Vec<AddressLookupTableAccount>> {
        match self
            .rpc
            .try_get_anchor_account::<RegistryAccount>(&self.ix.registry_address())
            .await?
        {
            None => {
                // registry doesn't exist yet, so create it
                self.send_transaction(&[self.ix.init_registry()]).await?;
                Ok(vec![])
            }
            Some(registry) => {
                let tables = registry
                    .tables
                    .into_iter()
                    .map(|e| e.table)
                    .collect::<Vec<_>>();

                Ok(self
                    .rpc
                    .get_accounts_all(&tables)
                    .await?
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, maybe_account)| {
                        maybe_account.and_then(|account| {
                            let state = AddressLookupTable::deserialize(&account.data).ok()?;

                            Some(AddressLookupTableAccount {
                                key: tables[i],
                                addresses: state.addresses.iter().copied().collect(),
                            })
                        })
                    })
                    .collect())
            }
        }
    }

    async fn send_transaction(&self, instructions: &[Instruction]) -> Result<()> {
        let recent_blockhash = self.rpc.get_latest_blockhash().await?;
        let mut tx =
            create_unsigned_transaction(instructions, &self.payer.pubkey(), &[], recent_blockhash)?;

        sign_transaction(self.payer, &mut tx)?;
        sign_transaction(self.signer, &mut tx)?;

        self.rpc.send_and_confirm_transaction(&tx).await?;
        Ok(())
    }
}

fn program_for_scope(scope: LookupScope) -> Pubkey {
    match scope {
        LookupScope::Airspace => jet_instructions::airspace::AIRSPACE_PROGRAM,
        LookupScope::Pools => jet_instructions::margin_pool::MARGIN_POOL_PROGRAM,
        LookupScope::Swaps => jet_instructions::margin_swap::MARGIN_SWAP_PROGRAM,
        LookupScope::FixedTerm => jet_instructions::fixed_term::FIXED_TERM_PROGRAM,
    }
}
