use jet_instructions::{
    airspace::derive_airspace,
    margin_orca::MARGIN_ORCA_PROGRAM,
    test_service::{derive_pyth_price, derive_token_mint},
};
use jet_margin::{TokenConfigUpdate, TokenKind};
use jet_solana_client::network::NetworkKind;

use crate::config::AirspaceConfig;

use super::{Builder, BuilderError, LookupScope};

pub(crate) async fn configure_whirlpools(
    builder: &mut Builder,
    airspace_config: &AirspaceConfig,
) -> Result<(), BuilderError> {
    let airspace = derive_airspace(&airspace_config.name);

    let margin_config_ix = builder.margin_config_ix(&airspace);

    let mut configure_whirlpool_ixns = vec![];
    let mut lookup_addresses = vec![];

    for whirlpool_input in &airspace_config.whirlpools {
        // Find the tokens of the whirlpool
        let token_a = airspace_config
            .tokens
            .iter()
            .find(|token| {
                let mint = token.mint.unwrap_or_else(|| derive_token_mint(&token.name));
                mint == whirlpool_input.base
            })
            .ok_or(BuilderError::UnknownToken(whirlpool_input.base.to_string()))?;
        let token_b = airspace_config
            .tokens
            .iter()
            .find(|token| {
                let mint = token.mint.unwrap_or_else(|| derive_token_mint(&token.name));
                mint == whirlpool_input.quote
            })
            .ok_or(BuilderError::UnknownToken(
                whirlpool_input.quote.to_string(),
            ))?;

        let can_derive_oracle = builder.network != NetworkKind::Mainnet;

        let whirlpool_ix = builder.margin_orca_ix(
            &airspace,
            &whirlpool_input.base,
            &whirlpool_input.quote,
            &if can_derive_oracle {
                let mint = derive_token_mint(&token_a.name);
                derive_pyth_price(&mint)
            } else {
                token_a.pyth_price.unwrap()
            },
            &if can_derive_oracle {
                let mint = derive_token_mint(&token_b.name);
                derive_pyth_price(&mint)
            } else {
                token_b.pyth_price.unwrap()
            },
        );

        log::info!(
            "configure Orca adapter for whirlpool with token pair {}/{}",
            token_a.name,
            token_b.name
        );

        configure_whirlpool_ixns
            .push(whirlpool_ix.create(builder.payer(), builder.proposal_authority()));

        // Register a token for the configured whirlpool
        configure_whirlpool_ixns.push(margin_config_ix.configure_token(
            whirlpool_ix.margin_position_mint,
            Some(TokenConfigUpdate {
                underlying_mint: whirlpool_ix.margin_position_mint,
                admin: jet_margin::TokenAdmin::Adapter(MARGIN_ORCA_PROGRAM),
                token_kind: TokenKind::AdapterCollateral,
                value_modifier: token_a.collateral_weight.min(token_b.collateral_weight),
                max_staleness: 0,
            }),
        ));

        lookup_addresses
            .extend_from_slice(&[whirlpool_ix.address, whirlpool_ix.margin_position_mint]);
    }

    if !configure_whirlpool_ixns.is_empty() {
        builder.propose(configure_whirlpool_ixns);
    }

    builder.register_lookups(LookupScope::Swaps, lookup_addresses);

    Ok(())
}
