use std::{cell::RefMut, convert::TryFrom};

use anchor_lang::prelude::*;

use bytemuck::{Pod, Zeroable};

use jet_program_common::map::Map;

/// The v2 state for an airdrop
pub struct AirdropV2<'a> {
    header: &'a AirdropMetadata,
    recipients: Map<'a, Pubkey, u64>,
}

impl<'a> AirdropV2<'a> {
    pub fn from_buffer(data: &'a mut [u8]) -> Result<Self> {
        let header_len = 8 + std::mem::size_of::<AirdropMetadata>();

        if data.len() < header_len {
            return Err(ErrorCode::AccountDidNotDeserialize.into());
        }

        let (header_buf, remaining_buf) = data.split_at_mut(header_len);
        let header = bytemuck::from_bytes(&header_buf[8..]);
        let recipients =
            Map::from_buffer(&mut remaining_buf[header_len..]).expect("account is corrupt");

        Ok(Self { header, recipients })
    }
}

impl<'a> std::ops::Deref for AirdropV2<'a> {
    type Target = AirdropMetadata;

    fn deref(&self) -> &Self::Target {
        self.header
    }
}

#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
#[account(zero_copy)]
pub struct AirdropMetadata {
    /// The address allowed to make changes to the airdrop metadata before finalizing.
    pub authority: Pubkey,

    /// The token account containing the tokens to be distributed as the airdrop reward
    pub reward_vault: Pubkey,

    /// The stake pool that rewards are staked into when claimed
    pub stake_pool: Pubkey,

    /// The time at which this airdrop expires, and can no longer be claimed
    pub expire_at: i64,

    /// Unused
    pub reserved: u64,

    /// A short descriptive text for the airdrop
    pub short_desc: [u8; 31],

    /// The bump seed for the reward vault
    pub vault_bump: [u8; 1],
}
