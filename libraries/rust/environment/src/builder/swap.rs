use std::str::FromStr;

use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction};

use jet_instructions::test_service::{
    derive_openbook_market, derive_spl_swap_pool, openbook_market_create, saber_swap_pool_create,
    spl_swap_pool_create,
};
use jet_solana_client::{
    network::NetworkKind, transaction::TransactionBuilder, NetworkUserInterface,
    NetworkUserInterfaceExt,
};

use crate::{builder::SetupPhase, config::EnvironmentConfig, programs::*};

use super::{resolve_token_mint, Builder, BuilderError};

pub fn resolve_swap_program(network: NetworkKind, name: &str) -> Result<Pubkey, BuilderError> {
    if let Ok(address) = Pubkey::from_str(name) {
        return Ok(address);
    }

    if name == "spl-swap" || name == "orca-spl-swap" {
        return Ok(match network {
            NetworkKind::Devnet => ORCA_V2_DEVNET,
            _ => ORCA_V2,
        });
    }
    if name == "saber-swap" {
        return Ok(SABER);
    }
    if name == "openbook" {
        return Ok(match network {
            NetworkKind::Devnet => OPENBOOK_DEVNET,
            _ => OPENBOOK,
        });
    }

    Err(BuilderError::UnknownSwapProgram(name.to_string()))
}

pub async fn create_swap_pools<'a, I: NetworkUserInterface>(
    builder: &mut Builder<I>,
    config: &EnvironmentConfig,
) -> Result<(), BuilderError> {
    if builder.network == NetworkKind::Mainnet {
        return Ok(());
    }

    for pool in &config.exchanges {
        let swap_program = resolve_swap_program(builder.network, &pool.program)?;
        let token_a = resolve_token_mint(config, &pool.base)?;
        let token_b = resolve_token_mint(config, &pool.quote)?;
        match &pool.program {
            p if p == "spl-swap" => {
                log::info!("create SPL swap pool for {}/{}", pool.base, pool.quote);

                let swap_info = derive_spl_swap_pool(&swap_program, &token_a, &token_b);

                if builder.account_exists(&swap_info.state).await? {
                    continue;
                }

                builder.setup(
                    SetupPhase::Swaps,
                    [spl_swap_pool_create(
                        &swap_program,
                        &builder.payer(),
                        &token_a,
                        &token_b,
                        8,
                        500,
                    )],
                )
            }
            p if p == "saber-swap" => {
                log::info!("create Saber swap pool for {}/{}", pool.base, pool.quote);

                let swap_info = derive_spl_swap_pool(&swap_program, &token_a, &token_b);

                if builder.account_exists(&swap_info.state).await? {
                    continue;
                }

                builder.setup(
                    SetupPhase::Swaps,
                    [saber_swap_pool_create(
                        &swap_program,
                        &builder.payer(),
                        &token_a,
                        &token_b,
                        8,
                        500,
                    )],
                )
            }
            p if p == "openbook" => {
                log::info!("Create Openbook market for {}/{}", pool.base, pool.quote);

                let market_info =
                    derive_openbook_market(&swap_program, &token_a, &token_b, &builder.payer());

                if builder.account_exists(&market_info.state).await? {
                    continue;
                }
                // Create bids, asks, event queue and request queue
                let state_accounts =
                    OpenbookStateAccounts::create(&builder.interface, &swap_program).await?;

                builder.setup(
                    SetupPhase::Swaps,
                    [openbook_market_create(
                        &swap_program,
                        &builder.payer(),
                        &token_a,
                        &token_b,
                        &state_accounts.bids,
                        &state_accounts.asks,
                        &state_accounts.event_queue,
                        &state_accounts.request_queue,
                        1000,
                    )],
                )
            }
            p => {
                log::warn!("ignoring unknown swap program {} {p}", pool.program);
                continue;
            }
        }
    }

    Ok(())
}

pub struct OpenbookStateAccounts {
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub event_queue: Pubkey,
    pub request_queue: Pubkey,
}

impl OpenbookStateAccounts {
    pub async fn create<I: NetworkUserInterface>(
        client: &I,
        dex_program: &Pubkey,
    ) -> Result<Self, BuilderError> {
        log::info!("Creating state accounts for an Openbook market");
        // Create large accounts that can't be created as PDAs
        let bid_ask_size = 65536 + 12;
        // let bid_ask_lamports = client
        //     .get_minimum_balance_for_rent_exemption(bid_ask_size)
        //     .await?;
        let bid_ask_lamports = 500_000_000;
        let bids = Keypair::new();
        let asks = Keypair::new();
        let bids_ix = system_instruction::create_account(
            &client.signer(),
            &bids.pubkey(),
            bid_ask_lamports,
            bid_ask_size as u64,
            dex_program,
        );
        let asks_ix = system_instruction::create_account(
            &client.signer(),
            &asks.pubkey(),
            bid_ask_lamports,
            bid_ask_size as u64,
            dex_program,
        );
        let event_queue_size = 262144 + 12;
        let request_queue_size = 5120 + 12;
        // let events_lamports = ctx
        //     .rpc()
        //     .get_minimum_balance_for_rent_exemption(event_queue_size)
        //     .await?;
        let events_lamports = 10_000_000_000;
        // let requests_lamports = ctx
        //     .rpc()
        //     .get_minimum_balance_for_rent_exemption(request_queue_size)
        //     .await?;
        let requests_lamports = 400_000_000;
        let events = Keypair::new();
        let requests = Keypair::new();
        let events_ix = system_instruction::create_account(
            &client.signer(),
            &events.pubkey(),
            events_lamports,
            event_queue_size as u64,
            dex_program,
        );
        let requests_ix = system_instruction::create_account(
            &client.signer(),
            &requests.pubkey(),
            requests_lamports,
            request_queue_size as u64,
            dex_program,
        );

        let accounts = Self {
            bids: bids.pubkey(),
            asks: asks.pubkey(),
            event_queue: events.pubkey(),
            request_queue: requests.pubkey(),
        };

        let transaction = TransactionBuilder {
            instructions: vec![bids_ix, asks_ix, events_ix, requests_ix],
            signers: vec![bids, asks, events, requests],
        };
        let (signatures, error) = client.send_condensed_ordered(&[transaction]).await;
        if let Some(error) = error {
            log::error!("Error creating Openbook state accounts: {}", error);
            return Err(error.into());
        }
        log::debug!("Created Openbook state accounts: {:?}", signatures);

        Ok(accounts)
    }
}
