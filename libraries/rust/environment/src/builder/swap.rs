use std::str::FromStr;

use solana_sdk::pubkey::Pubkey;

use jet_instructions::test_service::spl_swap_pool_create;
use jet_solana_client::{network::NetworkKind, NetworkUserInterface};

use crate::{
    config::EnvironmentConfig,
    programs::{ORCA_V2, ORCA_V2_DEVNET},
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

    Err(BuilderError::UnknownSwapProgram(name.to_string()))
}

pub async fn create_swap_pools<'a, I: NetworkUserInterface>(
    builder: &mut Builder<I>,
    config: &EnvironmentConfig,
) -> Result<(), BuilderError> {
    for pool in &config.exchanges {
        if pool.program != "spl-swap" {
            log::warn!("ignoring unknown swap program {}", pool.program);
            continue;
        }

        log::info!("create SPL swap pool for {}/{}", pool.base, pool.quote);
        let token_a = resolve_token_mint(config, &pool.base)?;
        let token_b = resolve_token_mint(config, &pool.quote)?;

        let swap_program = resolve_swap_program(builder.network, &pool.program)?;

        builder.setup([spl_swap_pool_create(
            &swap_program,
            &builder.payer(),
            &token_a,
            &token_b,
            8,
            500,
        )])
    }

    Ok(())
}
