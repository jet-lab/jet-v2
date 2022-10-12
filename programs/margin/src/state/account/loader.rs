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

use anchor_lang::prelude::*;
use anchor_lang::{error::ErrorCode as AnchorErrorCode, AccountsClose, Discriminator};

use crate::MarginAccount;

/// An anchor-compatible account loader for the margin account
#[derive(Debug, Clone)]
pub struct MarginAccountLoader<'info> {
    acc_info: AccountInfo<'info>,
}

impl<'info> MarginAccountLoader<'info> {
    fn new(acc_info: AccountInfo<'info>) -> Self {
        Self { acc_info }
    }

    pub fn try_from(acc_info: &AccountInfo<'info>) -> Result<Self> {
        if acc_info.owner != &MarginAccount::owner() {
            return Err(Error::from(AnchorErrorCode::AccountOwnedByWrongProgram)
                .with_pubkeys((*acc_info.owner, MarginAccount::owner())));
        }

        let data: &[u8] = &acc_info.try_borrow_data()?;
        if data.len() < MarginAccount::discriminator().len() {
            return Err(ErrorCode::AccountDiscriminatorNotFound.into());
        }

        // Discriminator must match.
        let disc_bytes = &data[..8];
        if disc_bytes != &MarginAccount::discriminator() {
            return Err(ErrorCode::AccountDiscriminatorMismatch.into());
        }

        Ok(Self::new(acc_info.clone()))
    }
}

impl<'info> Accounts<'info> for MarginAccountLoader<'info> {
    fn try_accounts(
        _program_id: &Pubkey,
        accounts: &mut &[AccountInfo<'info>],
        _ix_data: &[u8],
        _bumps: &mut std::collections::BTreeMap<String, u8>,
        _reallocs: &mut std::collections::BTreeSet<Pubkey>,
    ) -> Result<Self> {
        if accounts.is_empty() {
            return Err(AnchorErrorCode::AccountNotEnoughKeys.into());
        }

        let account = &accounts[0];
        *accounts = &accounts[1..];

        Self::try_from(account)
    }
}

impl<'info> AccountsExit<'info> for MarginAccountLoader<'info> {
    fn exit(&self, program_id: &Pubkey) -> Result<()> {
        AccountLoader::<MarginAccount>::try_from_unchecked(&Pubkey::default(), &self.acc_info)
            .unwrap()
            .exit(program_id)
    }
}

impl<'info> AccountsClose<'info> for MarginAccountLoader<'info> {
    fn close(&self, sol_destination: AccountInfo<'info>) -> Result<()> {
        AccountLoader::<MarginAccount>::try_from_unchecked(&Pubkey::default(), &self.acc_info)
            .unwrap()
            .close(sol_destination)
    }
}

impl<'info> ToAccountMetas for MarginAccountLoader<'info> {
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        AccountLoader::<MarginAccount>::try_from_unchecked(&Pubkey::default(), &self.acc_info)
            .unwrap()
            .to_account_metas(is_signer)
    }
}

impl<'info> AsRef<AccountInfo<'info>> for MarginAccountLoader<'info> {
    fn as_ref(&self) -> &AccountInfo<'info> {
        &self.acc_info
    }
}

impl<'info> ToAccountInfos<'info> for MarginAccountLoader<'info> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        vec![self.acc_info.clone()]
    }
}

impl<'info> Key for MarginAccountLoader<'info> {
    fn key(&self) -> Pubkey {
        *self.acc_info.key
    }
}
