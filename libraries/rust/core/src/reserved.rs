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
use bytemuck::{Pod, Zeroable};

/// Utility for representing some reserved amount of space within a struct
/// where having a fixed size is desirable.
#[derive(Clone, Copy)]
pub struct Reserved<const S: usize>([u8; S]);

impl<const S: usize> Default for Reserved<S> {
    fn default() -> Self {
        Self([0u8; S])
    }
}

impl<const S: usize> std::fmt::Debug for Reserved<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "reserved[{}]", S)
    }
}

unsafe impl<const S: usize> Zeroable for Reserved<S> {}
unsafe impl<const S: usize> Pod for Reserved<S> {}
