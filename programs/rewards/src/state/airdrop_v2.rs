use std::{cell::RefMut, io::Write};

use anchor_lang::prelude::*;

use bytemuck::Contiguous;

use jet_program_common::{map::Map, pod::PodBytes};

use crate::{error::ErrorCode, seeds, RewardResult};

type RecipientMap<'a> = Map<'a, Pubkey, u64>;

/// The v2 state for an airdrop
pub struct AirdropV2<'a> {
    header: RefMut<'a, AirdropMetadata>,
    recipients: RecipientMap<'a>,
}

impl<'a> AirdropV2<'a> {
    pub const MINIMUM_SIZE: usize = std::mem::size_of::<AirdropMetadata>()
        + RecipientMap::HEADER_SIZE
        + RecipientMap::ENTRY_SIZE;

    pub const fn extra_space_for_recipients(recipients: usize) -> usize {
        RecipientMap::ENTRY_SIZE * recipients
    }

    pub fn initialize(
        account: &'a AccountInfo,
        params: &AirdropCreateParams,
    ) -> RewardResult<Self> {
        let data = account.try_borrow_mut_data().unwrap();
        let mut airdrop = Self::from_account_data(data, true)?;

        *airdrop.header = AirdropMetadata {
            authority: params.authority,
            expire_at: params.expire_at,
            stake_pool: params.stake_pool,
            vault: params.vault,
            bump_seed: [params.bump_seed],
            seed: params.seed.to_le_bytes(),
            ..Default::default()
        };

        airdrop
            .header
            .short_desc
            .as_mut()
            .write_all(params.short_desc.as_bytes())
            .unwrap();

        Ok(airdrop)
    }

    /// Load the airdrop from an account
    pub fn from_account<'b: 'a>(account: &'a AccountInfo<'b>) -> RewardResult<Self> {
        if account.owner != &crate::ID {
            msg!("wrong owner for airdrop account");
            return Err(ErrorCode::AirdropInvalidFormat);
        }

        let header_len = 8 + std::mem::size_of::<AirdropMetadata>();
        let data = account.try_borrow_mut_data().unwrap();

        if data.len() < header_len {
            return Err(ErrorCode::AirdropInvalidFormat);
        }

        let data = account.try_borrow_mut_data().unwrap();
        Self::from_account_data(data, false)
    }

    /// Get the amount claimable by a recipient
    pub fn get_recipient_amount(&self, recipient: &Pubkey) -> Option<u64> {
        self.recipients.get(&recipient).copied()
    }

    /// Add recipients to the airdrop
    pub fn add_recipients(
        &mut self,
        recipients: impl IntoIterator<Item = AirdropRecipient>,
    ) -> RewardResult<()> {
        if self.header.status() != AirdropStatus::Draft {
            return Err(ErrorCode::AirdropNotDraft);
        }

        for entry in recipients {
            let removed = self
                .recipients
                .insert(entry.recipient, entry.amount)
                .expect("airdrop recipient map is full");

            if let Some(value) = removed {
                self.header.amount = self.header.amount.checked_sub(value).unwrap();
            }

            self.header.amount = self.header.amount.checked_add(entry.amount).unwrap();
        }

        Ok(())
    }

    /// Change the authority field on the airdrop
    pub fn change_authority(&mut self, new_authority: Pubkey) {
        self.header.authority = new_authority;
    }

    /// Finalize the airdrop's recipients list, moving it to `Review` status
    pub fn finalize_recipients(&mut self) {
        if self.header.status() == AirdropStatus::Draft {
            self.header.status = AirdropStatus::Review.into_integer();
        }
    }

    /// Finalize the airdrop, allowing claims
    pub fn finalize(&mut self) {
        self.header.status = AirdropStatus::Final.into_integer();
    }

    /// Claim tokens from the airdrop
    pub fn claim(&mut self, recipient: &Pubkey) -> RewardResult<u64> {
        if self.header.status() != AirdropStatus::Final {
            return Err(ErrorCode::AirdropNotFinal);
        }

        let claimed = self
            .recipients
            .remove(&recipient)
            .ok_or(ErrorCode::RecipientNotFound)?;

        self.header.amount = self.header.amount.checked_sub(claimed).unwrap();
        Ok(claimed)
    }

    fn from_account_data(data: RefMut<'a, &mut [u8]>, init: bool) -> RewardResult<Self> {
        let header_len = 8 + std::mem::size_of::<AirdropMetadata>();

        if data.len() < header_len {
            return Err(ErrorCode::AirdropInvalidFormat);
        }

        let (header_buf, remaining_buf) =
            RefMut::map_split(data, |data| data.split_at_mut(header_len));
        let header = RefMut::map(header_buf, |buf| bytemuck::from_bytes_mut(&mut buf[8..]));
        let recipients = match init {
            false => Map::from_buffer(remaining_buf),
            true => Ok(Map::initialize(remaining_buf)),
        };

        let recipients = match recipients {
            Err(_) => {
                msg!("recipient map is corrupt");
                return Err(ErrorCode::AirdropInvalidFormat);
            }

            Ok(r) => r,
        };

        Ok(Self { header, recipients })
    }
}

impl<'a> std::ops::Deref for AirdropV2<'a> {
    type Target = AirdropMetadata;

    fn deref(&self) -> &Self::Target {
        self.header.deref()
    }
}

#[repr(C)]
#[derive(Default, Debug)]
#[account(zero_copy)]
pub struct AirdropMetadata {
    /// The address allowed to make changes to the airdrop metadata before finalizing.
    pub authority: Pubkey,

    /// The token account containing the tokens to be distributed as the airdrop reward
    pub vault: Pubkey,

    /// The stake pool that rewards are staked into when claimed
    pub stake_pool: Pubkey,

    /// The time at which this airdrop expires, and can no longer be claimed
    pub expire_at: i64,

    /// The total amount of tokens to be distributed
    pub amount: u64,

    /// The seed value for the airdrop account
    pub seed: [u8; 8],

    /// A short descriptive text for the airdrop
    pub short_desc: PodBytes<46>,

    /// The bump seed for the account
    pub bump_seed: [u8; 1],

    /// The status of this airdrop, which determines the actions that can be performed on it
    status: u8,
}

impl AirdropMetadata {
    pub fn signer_seeds(&self) -> [&[u8]; 3] {
        [seeds::AIRDROP, &self.seed, &self.bump_seed]
    }

    pub fn status(&self) -> AirdropStatus {
        AirdropStatus::from_integer(self.status).expect("invalid airdrop status")
    }
}

#[repr(u8)]
#[derive(Contiguous, Debug, Clone, Copy, Eq, PartialEq)]
pub enum AirdropStatus {
    /// The airdrop is still being created
    Draft = 0,

    /// The airdrop recipient list is final and ready for review,
    Review = 1,

    /// The aidrop is finalized, and allows claims
    Final = 2,
}

#[derive(Debug, AnchorDeserialize, AnchorSerialize)]
pub struct AirdropCreateParams {
    /// The address allowed to make changes to the airdrop metadata before finalizing.
    pub authority: Pubkey,

    /// The token account containing the tokens to be distributed as the airdrop reward
    pub vault: Pubkey,

    /// The stake pool that rewards are staked into when claimed
    pub stake_pool: Pubkey,

    /// The time at which this airdrop expires, and can no longer be claimed
    pub expire_at: i64,

    /// The seed value for the airdrop account
    pub seed: u64,

    /// The bump seed value for the airdrop account
    pub bump_seed: u8,

    /// A short descriptive text for the airdrop
    pub short_desc: String,
}

#[derive(Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub struct AirdropRecipient {
    /// The amount to be received by the recipient
    pub amount: u64,

    /// The address of the recipient allowed to claim the airdrop amount
    pub recipient: Pubkey,
}
