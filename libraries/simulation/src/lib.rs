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

use rand::rngs::mock::StepRng;
use solana_sdk::signature::Keypair;
use std::mem::MaybeUninit;
use std::sync::{Mutex, Once};

mod runtime;
pub mod tokens;

#[cfg(feature = "margin")]
pub mod margin;

/// Token swap
pub mod swap;

pub use runtime::{EntryFn, TestRuntime};

pub async fn send_and_confirm(
    rpc: &std::sync::Arc<dyn jet_solana_rpc_api::SolanaRpcClient>,
    instructions: &[solana_sdk::instruction::Instruction],
    signers: &[&solana_sdk::signature::Keypair],
) -> Result<solana_sdk::signature::Signature, anyhow::Error> {
    use solana_sdk::signature::Signer;

    let blockhash = rpc.get_latest_blockhash().await?;
    let mut all_signers = vec![rpc.payer()];

    all_signers.extend(signers);

    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        instructions,
        Some(&rpc.payer().pubkey()),
        &all_signers,
        blockhash,
    );

    rpc.send_and_confirm_transaction(&tx).await
}

/// Generate a new wallet keypair with some initial funding
pub async fn create_wallet(
    rpc: &std::sync::Arc<dyn jet_solana_rpc_api::SolanaRpcClient>,
    lamports: u64,
) -> Result<solana_sdk::signature::Keypair, anyhow::Error> {
    let wallet = solana_sdk::signature::Keypair::new();
    let tx = solana_sdk::system_transaction::create_account(
        rpc.payer(),
        &wallet,
        rpc.get_latest_blockhash().await?,
        lamports,
        0,
        &solana_sdk::system_program::ID,
    );

    rpc.send_and_confirm_transaction(&tx).await?;

    Ok(wallet)
}

#[macro_export]
macro_rules! assert_program_error_code {
    ($code:expr, $result:expr) => {{
        use solana_sdk::program_error::ProgramError;

        assert!($result.is_err(), "result is not an error");
        let err_obj = $result.unwrap_err();
        let actual_err = err_obj
            .downcast_ref::<ProgramError>()
            .expect("not a program error");

        let expect_err = &ProgramError::Custom($code);

        assert_eq!(
            expect_err, actual_err,
            "expected error {} but got {}",
            expect_err, actual_err
        )
    }};
}

#[macro_export]
macro_rules! assert_program_error {
    ($error:expr, $result:expr) => {{
        use solana_sdk::program_error::ProgramError;

        let result_value = $result;
        assert!(result_value.is_err(), "result is not an error");
        let err_obj = result_value.unwrap_err();
        let actual_err = err_obj
            .downcast_ref::<ProgramError>()
            .expect("not a program error");

        let expect_err = &ProgramError::Custom($error as u32 + 6000);

        assert_eq!(
            expect_err, actual_err,
            "expected error {} but got {}",
            expect_err, actual_err
        )
    }};
}

pub fn generate_keypair() -> Keypair {
    static MOCK_RNG_INIT: Once = Once::new();
    static mut MOCK_RNG: MaybeUninit<Mutex<MockRng>> = MaybeUninit::uninit();

    unsafe {
        MOCK_RNG_INIT.call_once(|| {
            MOCK_RNG.write(Mutex::new(MockRng(StepRng::new(1, 1))));
        });

        Keypair::generate(&mut *MOCK_RNG.assume_init_ref().lock().unwrap())
    }
}

struct MockRng(StepRng);

impl rand::CryptoRng for MockRng {}

impl rand::RngCore for MockRng {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.0.try_fill_bytes(dest)
    }
}
