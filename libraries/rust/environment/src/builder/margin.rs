use jet_instructions::margin::derive_token_config;
use solana_sdk::pubkey::Pubkey;

use jet_margin::{TokenConfig, TokenConfigUpdate};
use jet_solana_client::{NetworkUserInterface, NetworkUserInterfaceExt};

use super::{Builder, BuilderError};

pub async fn configure_margin_token<I: NetworkUserInterface>(
    builder: &mut Builder<I>,
    airspace: &Pubkey,
    mint: &Pubkey,
    config: Option<TokenConfigUpdate>,
) -> Result<(), BuilderError> {
    let existing_config = get_token_config(builder, airspace, mint).await?;

    let should_update = match (existing_config, &config) {
        (None, None) => false,
        (Some(existing), Some(update)) => existing != *update,
        _ => true,
    };

    if should_update {
        log::info!("updating margin token config for mint {mint}");

        let ix_builder = builder.margin_config_ix(airspace);
        builder.propose([ix_builder.configure_token(*mint, config)])
    }

    Ok(())
}

pub async fn get_token_config<I: NetworkUserInterface>(
    builder: &Builder<I>,
    airspace: &Pubkey,
    mint: &Pubkey,
) -> Result<Option<TokenConfig>, BuilderError> {
    let address = derive_token_config(airspace, mint);
    Ok(builder.interface.get_anchor_account(&address).await?)
}
