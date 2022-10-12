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

use bytemuck::Pod;
use std::{cell::RefMut, io::Write};

pub fn load_pyth_account<'a, T: Pod>(acc_info: &'a AccountInfo) -> Result<RefMut<'a, T>> {
    Ok(RefMut::map(acc_info.try_borrow_mut_data()?, |data| {
        bytemuck::from_bytes_mut(bytemuck::cast_slice_mut::<u8, u8>(
            bytemuck::try_cast_slice_mut(&mut data[0..std::mem::size_of::<T>()]).unwrap(),
        ))
    }))
}

pub fn write_pyth_product_attributes(mut storage: &mut [u8], attributes: &[(&str, &str)]) {
    for (key, value) in attributes {
        msg!("product {} = {}", key, value); // ?
        storage.write_all(&[key.len() as u8]).unwrap();
        storage.write_all(key.as_bytes()).unwrap();
        storage.write_all(&[value.len() as u8]).unwrap();
        storage.write_all(value.as_bytes()).unwrap();
    }
}
