use anchor_lang::prelude::*;

use super::distribution::TokenDistribution;

#[account]
#[derive(Default)]
pub struct Award {
    /// The authority allowed to revoke/close the award
    pub authority: Pubkey,

    /// The seed for the address
    pub seed: [u8; 30],

    /// The length of the seed
    pub seed_len: u8,

    /// The bump seed for the address
    pub bump_seed: [u8; 1],

    /// The stake account the award is deposited to
    pub stake_account: Pubkey,

    /// The token account storing the unvested balance
    pub vault: Pubkey,

    /// The details on the token distribution
    pub token_distribution: TokenDistribution,
}

impl Award {
    pub fn space() -> usize {
        32 + 30 + 1 + 1 + 32 + 32 + TokenDistribution::space()
    }

    pub fn signer_seeds(&self) -> [&[u8]; 4] {
        [
            b"award".as_ref(),
            self.stake_account.as_ref(),
            &self.seed[..self.seed_len as usize],
            self.bump_seed.as_ref(),
        ]
    }
}

impl std::ops::Deref for Award {
    type Target = TokenDistribution;

    fn deref(&self) -> &Self::Target {
        &self.token_distribution
    }
}

impl std::ops::DerefMut for Award {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.token_distribution
    }
}
