use std::{collections::HashSet, time::Duration};

use anchor_lang::prelude::Pubkey;
use anyhow::{bail, Context, Result};
use jet_margin_sdk::jet_fixed_term::ID as FIXED_TERM_ID;
use jet_margin_sdk::jet_margin_pool::ID as MARGIN_POOL_ID;
use jet_solana_client::rpc::native::RpcConnection;
use lookup_table_registry_client::{instructions::InstructionBuilder, Entry, Registry};
use solana_sdk::transaction::Transaction;

use crate::{
    addresses::ProgramAddresses,
    client::{Client, Plan, TransactionEntry},
};

/// The registry is created separately due to some constraints in the program.
/// Lookup tables can't be created in the same transaction as the registry.
pub async fn create_registry(client: &Client, builder: &InstructionBuilder) -> Result<Plan> {
    let registry_address = builder.registry_address();
    if client.account_exists(&registry_address).await? {
        bail!("Registry already exists, cannot create it");
    }
    // Create a registry account
    let create_registry_ix = builder.init_registry().await?;
    let mut plan = Plan {
        entries: vec![],
        unordered: false,
    };
    plan.entries.push(TransactionEntry {
        steps: vec![format!(
            "Create a registry account for {}",
            builder.authority
        )],
        transaction: Transaction::new_with_payer(
            &[create_registry_ix],
            client.config.signer().as_ref(),
        ),
        signers: vec![],
    });

    Ok(plan)
}

pub async fn update_registry(
    client: &Client,
    builder: &InstructionBuilder,
    airspace: Pubkey,
) -> Result<Plan> {
    // get the registry account
    // populate the accounts to add by program
    // find lookup tables that contain the relevant programs
    // filter out the addresses that already exist
    // build instructions to add new ones
    // if lookup tables would be full, create new ones
    let mut plan = Plan {
        entries: vec![],
        unordered: false,
    };

    let registry_address = builder.registry_address();
    if !client.account_exists(&registry_address).await? {
        bail!("Registry does not exist, please create it first.");
    }

    // Get the accounts to add

    let rpc = RpcConnection::from(client.config.rpc_client());
    let mut program_addresses = ProgramAddresses::fetch(&rpc, airspace).await?;

    let mut registry = Registry::fetch(&client.rpc(), &builder.authority).await?;

    // Remove addresses that already exist
    for table in &registry.tables {
        // Rely on the program being included in the accounts
        if table.addresses.contains(&FIXED_TERM_ID) {
            // We could use intersection(), but it loops over the addresses,
            // so we directly loop and remove existing entries
            for address in &table.addresses {
                program_addresses.fixed_term.remove(address);
            }
        }
        if table.addresses.contains(&MARGIN_POOL_ID) {
            for address in &table.addresses {
                program_addresses.margin_pool.remove(address);
            }
        }
    }

    add_to_registry(
        client,
        builder,
        &mut plan,
        &mut program_addresses.fixed_term,
        &mut registry,
        FIXED_TERM_ID,
        airspace,
    )
    .await;
    add_to_registry(
        client,
        builder,
        &mut plan,
        &mut program_addresses.margin_pool,
        &mut registry,
        MARGIN_POOL_ID,
        airspace,
    )
    .await;

    Ok(plan)
}

pub async fn remove_lookup_table(
    client: &Client,
    builder: &InstructionBuilder,
    address: Pubkey,
) -> Result<Plan> {
    let registry = Registry::fetch(&client.rpc(), &builder.authority).await?;
    registry
        .tables
        .iter()
        .find(|entry| entry.lookup_address == address)
        .context("Lookup table does not exist or does not belong to authority")?;
    let ix = builder.remove_lookup_table(address).await;

    let plan = Plan {
        entries: vec![TransactionEntry {
            steps: vec![format!("Remove lookup table")],
            transaction: Transaction::new_with_payer(&[ix], client.config.signer().as_ref()),
            signers: vec![],
        }],
        unordered: false,
    };

    Ok(plan)
}

pub async fn close_registry(_client: &Client, _builder: &InstructionBuilder) -> Result<Plan> {
    todo!()
}

async fn add_to_registry(
    client: &Client,
    builder: &InstructionBuilder,
    plan: &mut Plan,
    program_addresses: &mut HashSet<Pubkey>,
    registry: &mut Registry,
    program: Pubkey,
    airspace: Pubkey,
) {
    // With the remaining addresses from both programs, find or create a lookup table
    while !program_addresses.is_empty() {
        // Find an address that has the fixed term program + airspace, append to it
        let eligible_entry = registry.tables.iter_mut().find(|entry| {
            entry.addresses.contains(&program)
                && entry.addresses.contains(&airspace)
                && entry.addresses.len() < 255
        });
        // If an entry is found, populate it and update it
        match eligible_entry {
            Some(entry) => {
                // We can safely add up to 28 entries at a time
                let addr_to_add = (255 - entry.addresses.len())
                    .min(program_addresses.len())
                    .min(28);
                // Any efficient way of removing n entries?
                let addresses = program_addresses
                    .iter()
                    .take(addr_to_add)
                    .cloned()
                    .collect::<Vec<_>>();
                let ix = builder.append_to_lookup_table(entry.lookup_address, &addresses, 0);
                for address in addresses {
                    program_addresses.remove(&address);
                    entry.addresses.push(address);
                }
                plan.entries.push(TransactionEntry {
                    steps: vec![format!(
                        "Add {addr_to_add} addresses to registry entry {}",
                        entry.lookup_address
                    )],
                    transaction: Transaction::new_with_payer(
                        &[ix],
                        client.config.signer().as_ref(),
                    ),
                    signers: vec![],
                })
            }
            None => {
                // introduce a small delay to prevent multiple lookup accounts
                // with the same slot being created.
                tokio::time::sleep(Duration::from_secs(3)).await;
                let (new_ix, new_lookup, _) = builder.create_lookup_table(0).await;
                let append_ix = builder.append_to_lookup_table(new_lookup, &[program, airspace], 0);
                plan.entries.push(TransactionEntry {
                    steps: vec![format!("Add a new lookup address {new_lookup}")],
                    transaction: Transaction::new_with_payer(
                        &[new_ix, append_ix],
                        client.config.signer().as_ref(),
                    ),
                    signers: vec![],
                });
                registry.tables.push(Entry {
                    addresses: vec![program, airspace],
                    discriminator: 0,
                    lookup_address: new_lookup,
                });
            }
        }
    }
}
