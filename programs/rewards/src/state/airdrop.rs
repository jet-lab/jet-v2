use anchor_lang::prelude::*;

#[cfg(feature = "cli")]
use serde::ser::{Serialize, SerializeStruct, Serializer};

use jet_program_common::pod::PodBytes;
use jet_program_proc_macros::assert_size;

use crate::ErrorCode;

#[account(zero_copy)]
pub struct Airdrop {
    /// The address of this account
    pub address: Pubkey,

    /// The token account containing the tokens to be distributed
    /// as the airdrop reward
    pub reward_vault: Pubkey,

    /// The address allowed to make changes to the airdrop metadata
    /// before finalizing.
    pub authority: Pubkey,

    /// The time at which this airdrop expires, and can no longer be claimed
    pub expire_at: i64,

    /// The stake pool that rewards are staked into when claimed
    pub stake_pool: Pubkey,

    /// Settings for airdrops
    pub flags: u64,

    /// A short descriptive text for the airdrop
    pub short_desc: [u8; 32],

    /// A longer descriptive text for the airdrop
    pub long_desc: PodBytes<255>,

    /// The bump seed for the reward vault
    pub vault_bump: [u8; 1],

    /// Storage space for the list of airdrop recipients
    pub target_info: PodBytes<400024>,
}

impl Airdrop {
    pub fn add_recipients(
        &mut self,
        start_idx: u64,
        to_add: impl Iterator<Item = (Pubkey, u64)>,
    ) -> Result<()> {
        let target = self.target_info_mut();

        if target.recipients_total != start_idx {
            return Err(ErrorCode::AddOutOfOrder.into());
        }

        if target.finalized > 0 {
            return Err(ErrorCode::AirdropFinal.into());
        }

        for (recipient, amount) in to_add {
            target.recipients[target.recipients_total as usize] =
                AirdropTarget { recipient, amount };

            target.recipients_total += 1;
            target.reward_total = target.reward_total.checked_add(amount).unwrap();
        }

        // Make sure the list of recipients is SORTED
        let range = match start_idx {
            0 => &target.recipients[..target.recipients_total as usize],
            i => &target.recipients[i as usize - 1..target.recipients_total as usize],
        };
        let is_sorted = range.windows(2).all(|i| i[0].recipient <= i[1].recipient);

        if !is_sorted {
            return Err(ErrorCode::RecipientsNotSorted.into());
        }

        Ok(())
    }

    pub fn has_recipient(&self, recipient: &Pubkey) -> bool {
        self.target_info().get_recipient(recipient).is_ok()
    }

    pub fn finalize(&mut self, vault_balance: u64) -> Result<()> {
        let target = self.target_info_mut();

        if vault_balance < target.reward_total {
            return Err(ErrorCode::AirdropInsufficientRewardBalance.into());
        }

        target.finalized = 1;
        Ok(())
    }

    pub fn claim(&mut self, recipient: &Pubkey) -> Result<u64> {
        let target = self.target_info_mut();

        if target.finalized != 1 {
            msg!("cannot claim from an unfinalized airdrop");
            return Err(ErrorCode::AirdropNotFinal.into());
        }

        let entry = target.get_recipient_mut(recipient)?;
        let amount = entry.amount;

        entry.amount = 0;
        target.reward_total = target.reward_total.checked_sub(amount).unwrap();

        Ok(amount)
    }

    pub fn signer_seeds(&self) -> [&[u8]; 3] {
        [self.address.as_ref(), b"vault".as_ref(), &self.vault_bump]
    }

    pub fn flags(&self) -> AirdropFlags {
        AirdropFlags::from_bits(self.flags).unwrap()
    }

    fn target_info_mut(&mut self) -> &mut AirdropTargetInfo {
        bytemuck::from_bytes_mut(self.target_info.as_mut())
    }

    pub fn target_info(&self) -> &AirdropTargetInfo {
        bytemuck::from_bytes(self.target_info.as_ref())
    }
}

impl std::fmt::Debug for Airdrop {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Airdrop")
            .field("address", &self.address)
            .field("reward_vault", &self.reward_vault)
            .field("authority", &self.authority)
            .field("stake_pool", &self.stake_pool)
            .field("expire_at", &(self.expire_at))
            .field("flags", &self.flags)
            .finish()
    }
}

#[cfg(feature = "cli")]
impl Serialize for Airdrop {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Airdrop", 7)?;
        s.serialize_field("rewardsVault", &self.reward_vault.to_string())?;
        s.serialize_field("authority", &self.authority.to_string())?;
        s.serialize_field("expireAt", &self.expire_at)?;
        s.serialize_field("stakePool", &self.stake_pool.to_string())?;
        s.serialize_field("flags", &self.flags)?;
        s.serialize_field(
            "shortDescription",
            &String::from_utf8(self.short_desc.to_vec())
                .unwrap()
                .replace("\u{0}", ""),
        )?;
        s.serialize_field(
            "longDescription",
            &String::from_utf8(self.long_desc.to_vec())
                .unwrap()
                .replace("\u{0}", ""),
        )?;
        s.end()
    }
}

#[repr(C)]
#[assert_size(400024)]
#[derive(Clone, Copy)]
pub struct AirdropTargetInfo {
    /// The total amount of reward tokens that are claimable by recipients
    pub reward_total: u64,

    /// The total number of airdrop recipients
    pub recipients_total: u64,

    /// Marker to indicate when the airdrop has been finalized
    /// from further edits
    pub finalized: u64,

    /// List of airdrop recipients that can claim tokens
    pub recipients: [AirdropTarget; 10000],
}

impl AirdropTargetInfo {
    fn get_recipient_mut(&mut self, recipient: &Pubkey) -> Result<&mut AirdropTarget> {
        let recipients = &mut self.recipients[..self.recipients_total as usize];

        let found = recipients
            .binary_search_by_key(recipient, |r: &AirdropTarget| r.recipient)
            .map_err(|_| ErrorCode::RecipientNotFound)?;

        Ok(&mut recipients[found])
    }

    fn get_recipient(&self, recipient: &Pubkey) -> Result<&AirdropTarget> {
        let recipients = &self.recipients[..self.recipients_total as usize];

        let found = recipients
            .binary_search_by_key(recipient, |r: &AirdropTarget| r.recipient)
            .map_err(|_| ErrorCode::RecipientNotFound)?;

        Ok(&recipients[found])
    }
}

#[repr(C)]
#[assert_size(40)]
#[derive(Clone, Copy)]
pub struct AirdropTarget {
    /// The amount of tokens that the target can claim
    pub amount: u64,

    /// The address allowed to claim the airdrop tokens
    pub recipient: Pubkey,
}

unsafe impl bytemuck::Pod for AirdropTargetInfo {}
unsafe impl bytemuck::Zeroable for AirdropTargetInfo {}

bitflags::bitflags! {
    pub struct AirdropFlags: u64 {
    }
}
