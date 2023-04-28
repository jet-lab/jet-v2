use std::str::FromStr;

use solana_sdk::pubkey::Pubkey;

use jet_instructions::test_service::{
    derive_spl_swap_pool, saber_swap_pool_create, spl_swap_pool_create,
};
use jet_solana_client::{network::NetworkKind, NetworkUserInterface};

use crate::{
    builder::SetupPhase,
    config::EnvironmentConfig,
    programs::{ORCA_V2, ORCA_V2_DEVNET, SABER},
};

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
            p => {
                log::warn!("ignoring unknown swap program {} {p}", pool.program);
                continue;
            }
        }
    }

    Ok(())
}
