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

/// Owned representation of key generating seeds
#[derive(Eq, PartialEq, Clone)]
pub struct Seeds {
    data: Vec<u8>,
    references: Vec<*const [u8]>,
}

impl Seeds {
    pub fn new(seeds: &[&[u8]]) -> Self {
        let capacity = seeds.iter().map(|s| s.len()).sum();
        let mut data = Vec::<u8>::with_capacity(capacity);
        let mut references = Vec::with_capacity(seeds.len());

        for seed in seeds {
            let start = data.len();
            data.extend_from_slice(seed);

            let end = data.len();
            let ptr = &data[start..end];

            references.push(ptr as *const [u8]);
        }

        Self { data, references }
    }
}

impl<'a> AsRef<[&'a [u8]]> for Seeds {
    fn as_ref(&self) -> &[&'a [u8]] {
        unsafe { std::mem::transmute(&self.references[..]) }
    }
}
