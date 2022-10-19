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

use anchor_lang::prelude::ProgramError;
use anchor_lang::ToAccountMetas;
use anchor_spl::token::accessor::amount;
use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint, entrypoint::ProgramResult,
    instruction::Instruction, program::invoke,
};

use jet_static_program_registry::macro_imports::Pubkey;
use jet_static_program_registry::{
    orca_swap_v1, orca_swap_v2, related_programs, spl_token_swap_v2,
};

declare_id!("H6YDgTPCT2AByCKYpLFnAsYbH9X8rSbv6H9fZPJ3gaVJ");

// register permitted swap programs
related_programs! {
    SwapProgram {[
        spl_token_swap_v2::Spl2,
        orca_swap_v1::OrcaV1,
        orca_swap_v2::OrcaV2,
    ]}
}

entrypoint!(process_instruction);

/// Takes an spl token swap instruction and changes the amount_in parameter to
/// use the full balance of the input token account.
///
/// The first account provided to this instruction must be the spl token swap
/// program. Otherwise the api is identical to the spl token swap.
fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let swap_program = &accounts[0];

    let data = use_client!(*swap_program.key, {
        use client::instruction::*;
        match SwapInstruction::unpack(instruction_data)? {
            SwapInstruction::Swap(swap) => Ok(SwapInstruction::Swap(Swap {
                amount_in: amount(&accounts[3])?,
                ..swap
            })
            .pack()),
            _ => {
                msg!("instruction unsupported");
                Err(ProgramError::Custom(471392))
            }
        }
    })??;

    invoke(
        &Instruction {
            program_id: *swap_program.key,
            accounts: accounts[1..].to_vec().to_account_metas(None),
            data,
        },
        accounts,
    )
}
