use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::{
    client::{Client, Plan},
    config::{ConfigType, TokenDefinition},
};

use super::margin_pool::ConfigurePoolCliOptions;

pub async fn process_apply(client: &Client, config_path: PathBuf) -> Result<Plan> {
    let path_metadata = config_path.metadata()?;

    if path_metadata.is_dir() {
        process_apply_directory(client, config_path).await
    } else {
        process_apply_file(client, config_path).await
    }
}

async fn process_apply_directory(client: &Client, directory: PathBuf) -> Result<Plan> {
    let mut plan = Plan::default();
    let mut dir_contents = tokio::fs::read_dir(directory).await?;

    while let Some(entry) = dir_contents.next_entry().await? {
        if !entry.metadata().await?.is_file() {
            continue;
        }

        plan.entries.extend(
            process_apply_file(client, entry.path())
                .await
                .with_context(|| format!("while processing file {:?}", entry.path()))?
                .entries,
        );
    }

    Ok(plan)
}

async fn process_apply_file(client: &Client, config_file: PathBuf) -> Result<Plan> {
    let config = crate::config::read_config_file(config_file).await?;

    match config {
        ConfigType::Token(token_def) => process_apply_token_def(client, token_def).await,
        _ => Ok(Plan::default()),
    }
}

async fn process_apply_token_def(client: &Client, token_def: TokenDefinition) -> Result<Plan> {
    let mut plan = Plan::default();

    plan.entries.extend(
        super::margin_pool::process_create_pool(client, token_def.config.mint)
            .await?
            .entries,
    );
    plan.entries.extend(
        super::margin_pool::process_configure_pool(
            client,
            ConfigurePoolCliOptions {
                token_config: token_def.config,
                margin_pool: token_def.margin_pool,
            },
        )
        .await?
        .entries,
    );

    Ok(plan)
}
