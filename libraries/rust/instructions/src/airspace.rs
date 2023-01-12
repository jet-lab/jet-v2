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

use anchor_lang::{InstructionData, ToAccountMetas};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, system_program};

use jet_airspace::seeds::{AIRSPACE, AIRSPACE_PERMIT, AIRSPACE_PERMIT_ISSUER, GOVERNOR_ID};

pub use jet_airspace::ID as AIRSPACE_PROGRAM;

/// A builder for [`jet_airspace::instruction`] instructions.
pub struct AirspaceIxBuilder {
    /// The user address that will pay for the transactions
    payer: Pubkey,

    /// The address with authority to request changes
    authority: Pubkey,

    /// The address of the relevant airspace
    address: Pubkey,

    /// The seed value used to generate the address
    seed: String,
}

impl AirspaceIxBuilder {
    /// Create a new instruction builder referencing an airspace by using a seed
    pub fn new(seed: &str, payer: Pubkey, authority: Pubkey) -> Self {
        let address = derive_airspace(seed);

        Self {
            payer,
            address,
            authority,
            seed: seed.to_owned(),
        }
    }

    /// making the field public would allow invalid states because it can
    /// diverge from the seed.
    pub fn address(&self) -> Pubkey {
        self.address
    }

    /// Create the governor identity account
    pub fn create_governor_id(&self) -> Instruction {
        let accounts = jet_airspace::accounts::CreateGovernorId {
            payer: self.payer,
            governor_id: derive_governor_id(),
            system_program: system_program::ID,
        }
        .to_account_metas(None);

        Instruction {
            accounts,
            program_id: jet_airspace::ID,
            data: jet_airspace::instruction::CreateGovernorId {}.data(),
        }
    }

    /// Set the protocol governor address
    ///
    /// # Params
    ///
    /// `new_governor` - The new governor address
    pub fn set_governor(&self, new_governor: Pubkey) -> Instruction {
        let accounts = jet_airspace::accounts::SetGovernor {
            governor: self.authority,
            governor_id: derive_governor_id(),
            system_program: system_program::ID,
        }
        .to_account_metas(None);

        Instruction {
            accounts,
            program_id: jet_airspace::ID,
            data: jet_airspace::instruction::SetGovernor { new_governor }.data(),
        }
    }

    /// Create the airspace
    ///
    /// # Params
    ///
    /// `is_restricted` - If true, the airspace requires specific issuers to enable user access
    pub fn create(&self, is_restricted: bool) -> Instruction {
        let accounts = jet_airspace::accounts::AirspaceCreate {
            payer: self.payer,
            airspace: self.address,
            system_program: system_program::ID,
        }
        .to_account_metas(None);

        Instruction {
            accounts,
            program_id: jet_airspace::ID,
            data: jet_airspace::instruction::AirspaceCreate {
                seed: self.seed.clone(),
                is_restricted,
                authority: self.authority,
            }
            .data(),
        }
    }

    /// Change the authority for the airspace
    ///
    /// # Params
    ///
    /// `new_authority` - The new address
    pub fn set_authority(&self, new_authority: Pubkey) -> Instruction {
        let accounts = jet_airspace::accounts::AirspaceSetAuthority {
            airspace: self.address,
            authority: self.authority,
        }
        .to_account_metas(None);

        Instruction {
            accounts,
            program_id: jet_airspace::ID,
            data: jet_airspace::instruction::AirspaceSetAuthority { new_authority }.data(),
        }
    }

    /// Register an address as being allowed to issue new permits for users
    ///
    /// # Params
    ///
    /// `issuer` - The address authorized to issue permits
    pub fn permit_issuer_create(&self, issuer: Pubkey) -> Instruction {
        let accounts = jet_airspace::accounts::AirspacePermitIssuerCreate {
            airspace: self.address,
            authority: self.authority,
            payer: self.payer,
            issuer_id: self.derive_issuer_id(&issuer),
            system_program: system_program::ID,
        }
        .to_account_metas(None);

        Instruction {
            accounts,
            program_id: jet_airspace::ID,
            data: jet_airspace::instruction::AirspacePermitIssuerCreate { issuer }.data(),
        }
    }

    /// Revoke an issuer from issuing new permits
    ///
    /// # Params
    ///
    /// `issuer` - The address no longer authorized to issue permits
    pub fn permit_issuer_revoke(&self, issuer: Pubkey) -> Instruction {
        let accounts = jet_airspace::accounts::AirspacePermitIssuerRevoke {
            airspace: self.address,
            authority: self.authority,
            receiver: self.payer,
            issuer_id: self.derive_issuer_id(&issuer),
        }
        .to_account_metas(None);

        Instruction {
            accounts,
            program_id: jet_airspace::ID,
            data: jet_airspace::instruction::AirspacePermitIssuerCreate { issuer }.data(),
        }
    }

    /// Issue a permit for an address, allowing it to use the airspace
    ///
    /// # Params
    ///
    /// `user` - The address authorized to use the airspace
    pub fn permit_create(&self, user: Pubkey) -> Instruction {
        let accounts = jet_airspace::accounts::AirspacePermitCreate {
            airspace: self.address,
            authority: self.authority,
            payer: self.payer,
            permit: self.derive_permit(&user),
            issuer_id: self.derive_issuer_id(&self.authority),
            system_program: system_program::ID,
        }
        .to_account_metas(None);

        Instruction {
            accounts,
            program_id: jet_airspace::ID,
            data: jet_airspace::instruction::AirspacePermitCreate { owner: user }.data(),
        }
    }

    /// Revoke a previously issued permit for an address
    ///
    /// # Params
    ///
    /// `user` - The address previously authorized to use the airspace
    /// `issuer` - The address that originally issued the permit
    pub fn permit_revoke(&self, user: Pubkey, issuer: Pubkey) -> Instruction {
        let accounts = jet_airspace::accounts::AirspacePermitRevoke {
            airspace: self.address,
            authority: self.authority,
            receiver: self.payer,
            permit: self.derive_permit(&user),
            issuer_id: self.derive_issuer_id(&issuer),
        }
        .to_account_metas(None);

        Instruction {
            accounts,
            program_id: jet_airspace::ID,
            data: jet_airspace::instruction::AirspacePermitCreate { owner: user }.data(),
        }
    }

    /// Derive the address for the account identifying permit issuers
    pub fn derive_issuer_id(&self, issuer: &Pubkey) -> Pubkey {
        derive_issuer_id(&self.address, issuer)
    }

    /// Derive the address for a user's permit to use the airspace
    pub fn derive_permit(&self, user: &Pubkey) -> Pubkey {
        derive_permit(&self.address, user)
    }
}

/// Derive the governor id account address
pub fn derive_governor_id() -> Pubkey {
    Pubkey::find_program_address(&[GOVERNOR_ID], &jet_airspace::ID).0
}

/// Derive the airspace address for a given seed
pub fn derive_airspace(seed: &str) -> Pubkey {
    Pubkey::find_program_address(&[AIRSPACE, seed.as_bytes()], &jet_airspace::ID).0
}

/// Derive the address for the account identifying permit issuers
pub fn derive_issuer_id(airspace: &Pubkey, issuer: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[AIRSPACE_PERMIT_ISSUER, airspace.as_ref(), issuer.as_ref()],
        &jet_airspace::ID,
    )
    .0
}

/// Derive the address for a user's permit to use the airspace
pub fn derive_permit(airspace: &Pubkey, user: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[AIRSPACE_PERMIT, airspace.as_ref(), user.as_ref()],
        &jet_airspace::ID,
    )
    .0
}
