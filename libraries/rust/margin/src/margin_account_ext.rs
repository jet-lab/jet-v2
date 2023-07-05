use anchor_lang::prelude::Pubkey;
use anyhow::Context;
use jet_instructions::margin::derive_margin_account_from_state;
use jet_margin::MarginAccount;

/// Simplifies MarginAccount reads with helper methods for common patterns.
pub trait MarginAccountExt {
    /// Returns the address of the margin account
    fn address(&self) -> Pubkey;

    /// Returns the balance in the margin account of this position.
    /// Returns zero if the position does not exist.
    fn balance(&self, position_token_mint: &Pubkey) -> u64;

    /// Returns the position token account address for a particular position
    /// token mint, or an error if it does not exist.
    fn position_address(&self, position_token_mint: &Pubkey) -> anyhow::Result<Pubkey>;
}

impl MarginAccountExt for MarginAccount {
    fn address(&self) -> Pubkey {
        derive_margin_account_from_state(self)
    }

    fn balance(&self, position_token_mint: &Pubkey) -> u64 {
        self.get_position(position_token_mint)
            .map(|p| p.balance)
            .unwrap_or(0)
    }

    fn position_address(&self, position_token_mint: &Pubkey) -> anyhow::Result<Pubkey> {
        Ok(self
            .get_position(position_token_mint)
            .with_context(|| {
                format!(
                    "Cannot find position of token {position_token_mint} for margin account {}",
                    self.address()
                )
            })?
            .address)
    }
}
