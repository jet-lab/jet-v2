use jet_instructions::{
    control::{MarginPoolConfiguration, TokenMetadataParams},
    margin::{TokenAdmin, TokenConfigUpdate, TokenKind},
    margin_pool::{derive_margin_pool, MarginPoolIxBuilder, MARGIN_POOL_PROGRAM},
};
use jet_margin_pool::MarginPool;
use jet_solana_client::{network::NetworkKind, NetworkUserInterface, NetworkUserInterfaceExt};

use super::{Builder, BuilderError, TokenContext};

pub(crate) async fn configure_for_token<I: NetworkUserInterface>(
    builder: &mut Builder<I>,
    token: &TokenContext,
) -> Result<(), BuilderError> {
    let margin_config_ix = builder.margin_config_ix(&token.airspace);
    let ctrl_ix = builder.control_ix();
    let pool_ix = MarginPoolIxBuilder::new(token.mint);

    let Some(pool_config) = &token.desc.margin_pool else {
            return Ok(());
        };

    let pool = builder
        .interface
        .get_anchor_account::<MarginPool>(&pool_ix.address)
        .await?;

    let mut configure_pool_ixns = vec![];

    if pool.is_none() {
        log::info!(
            "create margin pool for token {} at {}",
            &token.desc.name,
            derive_margin_pool(&token.airspace, &token.mint)
        );
        configure_pool_ixns.push(ctrl_ix.create_margin_pool(&token.mint));
    }

    let should_reconfigure = match pool {
        None => true,
        Some(pool) => pool.token_price_oracle != token.pyth_price || pool.config != *pool_config,
    };

    if should_reconfigure {
        log::info!(
            "configure margin pool for token {} at {}: {:#?}",
            &token.desc.name,
            derive_margin_pool(&token.airspace, &token.mint),
            pool_config
        );

        configure_pool_ixns.push(ctrl_ix.configure_margin_pool(
            &token.mint,
            &MarginPoolConfiguration {
                pyth_price: Some(token.pyth_price),
                pyth_product: Some(token.pyth_product),
                metadata: Some(TokenMetadataParams {
                    token_kind: jet_metadata::TokenKind::Collateral,
                    collateral_weight: token.desc.collateral_weight,
                    max_leverage: token.desc.max_leverage,
                }),
                parameters: Some(*pool_config),
            },
        ));
    }

    if !configure_pool_ixns.is_empty() {
        builder.propose(configure_pool_ixns);
    }

    if builder.network == NetworkKind::Mainnet {
        return Ok(());
    }

    let note_configs = builder
        .get_margin_token_configs(
            &token.airspace,
            &[pool_ix.deposit_note_mint, pool_ix.loan_note_mint],
        )
        .await?;

    let should_update_deposit = note_configs[0]
        .as_ref()
        .map(|c| c.value_modifier != token.desc.collateral_weight)
        .unwrap_or(true);
    let should_update_loan = note_configs[1]
        .as_ref()
        .map(|c| c.value_modifier != token.desc.max_leverage)
        .unwrap_or(true);

    if should_update_deposit {
        builder.propose([margin_config_ix.configure_token(
            pool_ix.deposit_note_mint,
            Some(TokenConfigUpdate {
                underlying_mint: token.mint,
                admin: TokenAdmin::Adapter(MARGIN_POOL_PROGRAM),
                token_kind: TokenKind::Collateral,
                value_modifier: token.desc.collateral_weight,
                max_staleness: 0,
            }),
        )]);
    }

    if should_update_loan {
        builder.propose([margin_config_ix.configure_token(
            pool_ix.loan_note_mint,
            Some(TokenConfigUpdate {
                underlying_mint: token.mint,
                admin: TokenAdmin::Adapter(MARGIN_POOL_PROGRAM),
                token_kind: TokenKind::Claim,
                value_modifier: token.desc.max_leverage,
                max_staleness: 0,
            }),
        )]);
    }

    Ok(())
}
