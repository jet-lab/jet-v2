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

use crate::{Invocation, state::account::operations::MarginAccountOperations};

#[account(zero_copy)]
#[repr(C)]
#[cfg_attr(not(target_arch = "bpf"), repr(align(8)))]
pub struct MarginAccount {
    pub version: u8,
    pub bump_seed: [u8; 1],
    pub user_seed: [u8; 2],

    /// Data an adapter can use to check what the margin program thinks about the current invocation
    /// Must normally be zeroed, except during an invocation.
    pub invocation: Invocation,

    pub reserved0: [u8; 3],

    /// The owner of this account, which generally has to sign for any changes to it
    pub owner: Pubkey,

    /// The state of an active liquidation for this account
    pub liquidation: Pubkey,

    /// The active liquidator for this account
    pub liquidator: Pubkey,

    /// The storage for tracking account balances
    pub positions: [u8; 7432],
}

/* 
impl MarginAccountOperations for MarginAccount {
    fn signer_seeds(&self) -> Seeds {
        Seeds::new(&[
            self.owner.as_ref(),
            self.user_seed.as_ref(),
            self.bump_seed.as_ref()
        ]) 
    }

    fn register_position(
            &mut self,
            config: &crate::TokenConfig,
            address: Pubkey,
        ) -> Result<crate::AccountPosition, ErrorCode> {
        
    }
}
 */