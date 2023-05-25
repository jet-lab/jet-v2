use std::str::FromStr;

use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

use jet_instructions::{
    orca::{derive_whirlpool, whirlpool_initialize_fee_tier, WhirlpoolIxBuilder},
    test_service::{
        derive_spl_swap_pool, derive_whirlpool_config, orca_whirlpool_create_config,
        saber_swap_pool_create, spl_swap_pool_create,
    },
};
use jet_program_common::programs::{ORCA_V2, ORCA_V2_DEVNET, ORCA_WHIRLPOOL, SABER};
use jet_solana_client::{
    network::NetworkKind, transaction::TransactionBuilder, NetworkUserInterface,
};

use crate::{builder::SetupPhase, config::EnvironmentConfig};

use super::{resolve_token_mint, Builder, BuilderError};

pub const DEFAULT_TICK_SPACING: u16 = 64;

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

    if name == "orca-whirlpool" {
        return Ok(ORCA_WHIRLPOOL);
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
                create_spl_swap_pool(builder, swap_program, token_a, token_b).await?
            }
            p if p == "saber-swap" => {
                create_saber_swap_pool(builder, swap_program, token_a, token_b).await?
            }
            p if p == "orca-whirlpool" => create_orca_whirlpool(builder, token_a, token_b).await?,
            p => {
                log::warn!("ignoring unknown swap program {} {p}", pool.program);
                continue;
            }
        }
    }

    Ok(())
}

async fn create_spl_swap_pool<I: NetworkUserInterface>(
    builder: &mut Builder<I>,
    swap_program: Pubkey,
    token_a: Pubkey,
    token_b: Pubkey,
) -> Result<(), BuilderError> {
    let swap_info = derive_spl_swap_pool(&swap_program, &token_a, &token_b);

    if builder.account_exists(&swap_info.state).await? {
        return Ok(());
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
    );

    Ok(())
}

async fn create_saber_swap_pool<I: NetworkUserInterface>(
    builder: &mut Builder<I>,
    swap_program: Pubkey,
    token_a: Pubkey,
    token_b: Pubkey,
) -> Result<(), BuilderError> {
    let swap_info = derive_spl_swap_pool(&swap_program, &token_a, &token_b);

    if builder.account_exists(&swap_info.state).await? {
        return Ok(());
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
    );

    Ok(())
}

async fn create_orca_whirlpool<I: NetworkUserInterface>(
    builder: &mut Builder<I>,
    token_a: Pubkey,
    token_b: Pubkey,
) -> Result<(), BuilderError> {
    const DEFAULT_FEE_RATE: u16 = 300;

    let whirlpool_config_addr = derive_whirlpool_config();

    if !builder.account_exists(&whirlpool_config_addr).await? {
        builder.setup(
            SetupPhase::TokenMints,
            [
                orca_whirlpool_create_config(&builder.payer(), &builder.payer(), DEFAULT_FEE_RATE),
                whirlpool_initialize_fee_tier(
                    &builder.payer(),
                    &builder.payer(),
                    &whirlpool_config_addr,
                    DEFAULT_TICK_SPACING,
                    DEFAULT_FEE_RATE,
                ),
            ],
        );
    }

    let (token_a, token_b) = (
        std::cmp::min(token_a, token_b),
        std::cmp::max(token_a, token_b),
    );

    let (whirlpool_addr, _) = derive_whirlpool(
        &whirlpool_config_addr,
        &token_a,
        &token_b,
        DEFAULT_TICK_SPACING,
    );

    if builder.account_exists(&whirlpool_addr).await? {
        log::debug!(
            "whirlpool {} for {}/{} already exists",
            whirlpool_addr,
            token_a,
            token_b
        );
        return Ok(());
    }

    let vault_a = Keypair::new();
    let vault_b = Keypair::new();

    let ix_builder = WhirlpoolIxBuilder::new(
        builder.payer(),
        whirlpool_config_addr,
        token_a,
        token_b,
        vault_a.pubkey(),
        vault_b.pubkey(),
        DEFAULT_TICK_SPACING,
    );

    builder.setup(
        SetupPhase::Swaps,
        [TransactionBuilder {
            instructions: vec![ix_builder.initialize_pool(1 << 64)],
            signers: vec![vault_a, vault_b],
        }],
    );

    Ok(())
}
