use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct Distribution {
    /// The address of this distribution account
    pub address: Pubkey,

    /// The authority that can manage this distribution.
    pub authority: Pubkey,

    /// The account with the tokens to be distributed
    pub vault: Pubkey,

    /// The seed for the address
    pub seed: [u8; 30],

    /// The length of the seed string
    pub seed_len: u8,

    /// The bump seed for the address
    pub bump_seed: [u8; 1],

    /// The account the rewards are distributed into
    pub target_account: Pubkey,

    /// The details on the token distribution
    pub token_distribution: TokenDistribution,
}

impl Distribution {
    pub fn space() -> usize {
        32 * 3 + 30 + 1 + 1 + 32 + TokenDistribution::space()
    }

    pub fn signer_seeds(&self) -> [&[u8]; 3] {
        [
            b"distribution".as_ref(),
            &self.seed[..self.seed_len as usize],
            self.bump_seed.as_ref(),
        ]
    }
}

impl std::ops::Deref for Distribution {
    type Target = TokenDistribution;

    fn deref(&self) -> &Self::Target {
        &self.token_distribution
    }
}

impl std::ops::DerefMut for Distribution {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.token_distribution
    }
}

#[derive(Default, AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub enum DistributionKind {
    #[default]
    Linear,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Default)]
pub struct TokenDistribution {
    /// The total amount of tokens to be distributed
    pub target_amount: u64,

    /// The amount of tokens already distributed
    pub distributed: u64,

    /// The time after which rewards will start to be distributed
    pub begin_at: u64,

    /// The time the distribution will be complete by
    pub end_at: u64,

    /// The type of distribution
    pub kind: DistributionKind,
}

impl TokenDistribution {
    pub fn space() -> usize {
        8 * 4 + 1
    }

    pub fn distribute(&mut self, timestamp: u64) -> u64 {
        let distributed = self.distributed;
        self.distributed = self.distributed_amount(timestamp);

        self.distributed.checked_sub(distributed).unwrap()
    }

    pub fn distributed_amount(&self, timestamp: u64) -> u64 {
        match self.kind {
            DistributionKind::Linear => self.distributed_amount_linear(timestamp),
        }
    }

    fn distributed_amount_linear(&self, timestamp: u64) -> u64 {
        let range = std::cmp::max(1, self.end_at.checked_sub(self.begin_at).unwrap()) as u128;
        let remaining = self.end_at.saturating_sub(timestamp) as u128;
        let target_amount = self.target_amount as u128;

        let distributed = target_amount
            .checked_sub((remaining.checked_mul(target_amount).unwrap()) / range)
            .unwrap();
        assert!(distributed < std::u64::MAX as u128);

        distributed as u64
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sanity_test_linear_distribution() {
        let dist = TokenDistribution {
            distributed: 0,
            target_amount: 1000000000000,
            begin_at: 1642720446,
            end_at: 1645312446,
            kind: DistributionKind::Linear,
        };

        let now = 1642721054;
        assert_eq!(234567902, dist.distributed_amount(now));
    }

    #[test]
    fn linear_distribution_at_limit() {
        let dist = TokenDistribution {
            distributed: 0,
            target_amount: 1000000000000,
            begin_at: 0,
            end_at: 0,
            kind: DistributionKind::Linear,
        };

        let now = 1642721054;
        assert_eq!(dist.target_amount, dist.distributed_amount(now));
    }
}
