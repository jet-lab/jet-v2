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

#[event]
pub struct AirspaceCreated {
    pub airspace: Pubkey,
    pub seed: String,
    pub authority: Pubkey,
    pub is_restricted: bool,
}

#[event]
pub struct AirspaceAuthoritySet {
    pub airspace: Pubkey,
    pub authority: Pubkey,
}

#[event]
pub struct AirspaceIssuerIdCreated {
    pub airspace: Pubkey,
    pub issuer: Pubkey,
}

#[event]
pub struct AirspaceIssuerIdRevoked {
    pub airspace: Pubkey,
    pub issuer: Pubkey,
}

#[event]
pub struct AirspacePermitCreated {
    pub airspace: Pubkey,
    pub issuer: Pubkey,
    pub owner: Pubkey,
}

#[event]
pub struct AirspacePermitRevoked {
    pub airspace: Pubkey,
    pub permit: Pubkey,
}
