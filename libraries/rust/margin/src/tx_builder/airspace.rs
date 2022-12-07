// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright (C) 2022 JET PROTOCOL HOLDINGS, LLC.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use solana_sdk::pubkey::Pubkey;

use crate::{
    fixed_term::FixedTermIxBuilder,
    ix_builder::{
        derive_airspace, derive_governor_id, get_control_authority_address,
        test_service::if_not_initialized, AirspaceIxBuilder, ControlIxBuilder,
        MarginConfigIxBuilder, MarginPoolConfiguration,
    },
    solana::transaction::TransactionBuilder,
};
use jet_margin::{TokenAdmin, TokenConfigUpdate, TokenKind, TokenOracle};

/// Utility for constructing transactions for administrative functions on protocol
/// resources within an airspace.
pub struct AirspaceAdmin {
    /// The airspace this interacts with
    pub airspace: Pubkey,
    authority: Pubkey,
    payer: Pubkey,
    as_ix: AirspaceIxBuilder,
}

impl AirspaceAdmin {
    /// Create new builder with payer as authority, for a given airspace based on its seed
    pub fn new(airspace_seed: &str, payer: Pubkey, authority: Pubkey) -> Self {
        Self {
            payer,
            authority,
            airspace: derive_airspace(airspace_seed),
            as_ix: AirspaceIxBuilder::new(airspace_seed, payer, authority),
        }
    }

    /// Create this airspace
    pub fn create_airspace(&self, is_restricted: bool) -> TransactionBuilder {
        vec![self.as_ix.create(is_restricted)].into()
    }

    /// Create a permit for a user to be allowed to use this airspace
    pub fn issue_user_permit(&self, user: Pubkey) -> TransactionBuilder {
        vec![self.as_ix.permit_create(user)].into()
    }

    /// Revoke a previously issued permit for a user, preventing them from continuing to
    /// use airspace resources.
    pub fn revoke_user_permit(&self, user: Pubkey, issuer: Pubkey) -> TransactionBuilder {
        vec![self.as_ix.permit_revoke(user, issuer)].into()
    }

    /// Create a new margin pool for a given token
    pub fn create_margin_pool(&self, token_mint: Pubkey) -> TransactionBuilder {
        let ctrl_ix_builder = ControlIxBuilder::new_for_authority(self.authority, self.payer);
        vec![ctrl_ix_builder.create_margin_pool(&token_mint)].into()
    }

    /// Configure a margin pool for the given token.
    pub fn configure_margin_pool(
        &self,
        token_mint: Pubkey,
        config: &MarginPoolConfiguration,
    ) -> TransactionBuilder {
        let mut instructions = vec![];
        let margin_config_ix_builder = MarginConfigIxBuilder::new(self.airspace, self.payer);

        // FIXME: remove control legacy
        let ctrl_ix_builder = ControlIxBuilder::new_for_authority(self.authority, self.payer);

        instructions.push(ctrl_ix_builder.configure_margin_pool(&token_mint, config));

        if let Some(metadata) = &config.metadata {
            let mut deposit_note_config_update = TokenConfigUpdate {
                admin: TokenAdmin::Adapter(jet_margin_pool::ID),
                underlying_mint: token_mint,
                token_kind: metadata.token_kind.into(),
                value_modifier: metadata.collateral_weight,
                max_staleness: 0,
            };

            let mut loan_note_config_update = TokenConfigUpdate {
                admin: TokenAdmin::Adapter(jet_margin_pool::ID),
                underlying_mint: token_mint,
                token_kind: TokenKind::Claim,
                value_modifier: metadata.max_leverage,
                max_staleness: 0,
            };

            if let Some(metadata) = &config.metadata {
                deposit_note_config_update.token_kind = metadata.token_kind.into();
                deposit_note_config_update.value_modifier = metadata.collateral_weight;
                loan_note_config_update.value_modifier = metadata.max_leverage;
            }

            instructions.push(
                margin_config_ix_builder
                    .configure_token(token_mint, Some(deposit_note_config_update)),
            );
            instructions.push(
                margin_config_ix_builder.configure_token(token_mint, Some(loan_note_config_update)),
            );
        }

        instructions.into()
    }

    /// Configure deposits for a given token (when placed directly into a margin account)
    pub fn configure_margin_token_deposits(
        &self,
        token_mint: Pubkey,
        config: Option<TokenDepositsConfig>,
    ) -> TransactionBuilder {
        let margin_config_ix = MarginConfigIxBuilder::new(self.airspace, self.payer);
        let config_update = config.map(|config| TokenConfigUpdate {
            underlying_mint: token_mint,
            token_kind: TokenKind::Collateral,
            value_modifier: config.collateral_weight,
            max_staleness: 0,
            admin: TokenAdmin::Margin {
                oracle: config.oracle,
            },
        });

        vec![margin_config_ix.configure_token(token_mint, config_update)].into()
    }

    /// Configure an adapter that can be invoked through a margin account
    pub fn configure_margin_adapter(
        &self,
        adapter_program_id: Pubkey,
        is_adapter: bool,
    ) -> TransactionBuilder {
        let margin_config_ix = MarginConfigIxBuilder::new(self.airspace, self.payer);

        vec![margin_config_ix.configure_adapter(adapter_program_id, is_adapter)].into()
    }

    /// Configure an adapter that can be invoked through a margin account
    pub fn configure_margin_liquidator(
        &self,
        liquidator: Pubkey,
        is_liquidator: bool,
    ) -> TransactionBuilder {
        let margin_config_ix = MarginConfigIxBuilder::new(self.airspace, self.payer);

        // FIXME: remove control legacy
        let ctrl_ix = ControlIxBuilder::new(self.payer);

        vec![
            ctrl_ix.set_liquidator(&liquidator, is_liquidator),
            margin_config_ix.configure_liquidator(liquidator, is_liquidator),
        ]
        .into()
    }

    /// Register a fixed term market for use with margin accounts
    pub fn register_fixed_term_market(
        &self,
        token_mint: Pubkey,
        seed: [u8; 32],
        collateral_weight: u16,
        max_leverage: u16,
    ) -> TransactionBuilder {
        let margin_config_ix = MarginConfigIxBuilder::new(self.airspace, self.payer);
        let market = FixedTermIxBuilder::market_key(&self.airspace, &token_mint, seed);
        let claims_mint = FixedTermIxBuilder::claims_mint(&market);
        let collateral_mint = FixedTermIxBuilder::collateral_mint(&market);

        let claims_update = TokenConfigUpdate {
            admin: TokenAdmin::Adapter(jet_market::ID),
            underlying_mint: token_mint,
            token_kind: TokenKind::Claim,
            value_modifier: max_leverage,
            max_staleness: 0,
        };

        let collateral_update = TokenConfigUpdate {
            admin: TokenAdmin::Adapter(jet_market::ID),
            underlying_mint: token_mint,
            token_kind: TokenKind::AdapterCollateral,
            value_modifier: collateral_weight,
            max_staleness: 0,
        };

        let claims_update_ix = margin_config_ix.configure_token(claims_mint, Some(claims_update));
        let collateral_update_ix =
            margin_config_ix.configure_token(collateral_mint, Some(collateral_update));

        TransactionBuilder {
            instructions: vec![claims_update_ix, collateral_update_ix],
            signers: vec![],
        }
    }
}

/// Configuration for token deposits into margin accounts
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct TokenDepositsConfig {
    /// The oracle for the token
    pub oracle: TokenOracle,

    /// Adjust the collateral value of deposits in the associated token
    pub collateral_weight: u16,
}

/// Instructions required to initialize global state for the protocol. Sets up the minimum state
/// necessary to configure resources within the protocol.
///
/// This primarily sets up the root permissions for the protocol. Must be signed by the default
/// governing address for the protocol. When built with the `testing` feature, the first signer
/// to submit these instructions becomes set as the governor address.
pub fn global_initialize_instructions(payer: Pubkey) -> Vec<TransactionBuilder> {
    let as_ix = AirspaceIxBuilder::new("", payer, payer);
    let ctrl_ix = ControlIxBuilder::new_for_authority(payer, payer);

    vec![
        if_not_initialized(get_control_authority_address(), ctrl_ix.create_authority()).into(),
        if_not_initialized(derive_governor_id(), as_ix.create_governor_id()).into(),
    ]
}

#[cfg(test)]
mod tests {
    use anchor_lang::AnchorDeserialize;
    use jet_margin::instruction::ConfigureToken;

    use super::*;

    #[test]
    fn check_value_modifiers() {
        let collateral_weight = 1_u16;
        let max_leverage = 2_u16;
        let foo = Pubkey::default();

        let am = AirspaceAdmin::new("test-airspace", foo, foo);
        let txb = am.register_fixed_term_market(foo, [0; 32], collateral_weight, max_leverage);

        let mut data = &txb.instructions[0].data[8..];
        let dec = ConfigureToken::deserialize(&mut data).unwrap();
        assert_eq!(dec.update.unwrap().value_modifier, max_leverage);

        let mut data = &txb.instructions[1].data[8..];
        let dec = ConfigureToken::deserialize(&mut data).unwrap();
        assert_eq!(dec.update.unwrap().value_modifier, collateral_weight);
    }
}
