use std::path::Path;

use anchor_lang::{AccountDeserialize, Discriminator};
use anyhow::{Context, Result};
use jet_margin_sdk::{
    ix_builder::{get_control_authority_address, get_metadata_address, ControlIxBuilder},
    jet_metadata::{
        LiquidatorMetadata, MarginAdapterMetadata, PositionTokenMetadata, TokenMetadata,
    },
};
use solana_sdk::pubkey::Pubkey;

use crate::client::{Client, Plan};

macro_rules! match_account_type {
    ($data:expr, [$($type:ident,)*]) => {
        match &$data[..8] {
            $(
                d if d == $type::discriminator() => {
                    Some(format!("{:#?}", $type::try_deserialize(&mut &$data[..])?))
                }
            )*
            _ => None
        }
    };
}

pub async fn process_check_metadata(client: &Client, address: Pubkey) -> Result<Plan> {
    let md_address = get_metadata_address(&address);

    if !client.account_exists(&md_address).await? {
        println!("There is no metadata set for this address");
    } else {
        let md_data = client.rpc().get_account_data(&md_address).await?;
        let matched_type_value = match_account_type!(
            md_data,
            [
                PositionTokenMetadata,
                TokenMetadata,
                MarginAdapterMetadata,
                LiquidatorMetadata,
            ]
        );

        match matched_type_value {
            None => println!("there is metadata, but its type is unknown"),
            Some(dbg_value) => println!("the metadata is a: {dbg_value}"),
        }
    }

    Ok(Plan::new())
}

pub async fn process_create_authority(client: &Client) -> Result<Plan> {
    let authority_address = get_control_authority_address();
    let ix = ControlIxBuilder::new(client.signer()?);

    if client.account_exists(&authority_address).await? {
        println!("authority already exists");
        return Ok(Plan::new());
    }

    Ok(client
        .plan()?
        .instructions(
            [],
            ["create global authority account"],
            [ix.create_authority()],
        )
        .build())
}

pub async fn process_generate_app_config(
    client: &Client,
    config_dir: &Path,
    output_path: &Path,
) -> Result<Plan> {
    let app_config = crate::app_config::JetAppConfig::generate_from_config_dir(client, config_dir)
        .await
        .with_context(|| format!("while reading configuration in {config_dir:?}"))?;
    let app_config_json = serde_json::to_string_pretty(&app_config)
        .with_context(|| "while serializing config to JSON")?;

    tokio::fs::write(output_path, &app_config_json)
        .await
        .with_context(|| format!("while trying to write to file {output_path:?}"))?;

    Ok(Plan::new())
}
