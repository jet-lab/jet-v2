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

use anyhow::Error;
use solana_client::client_error::ClientError;
use std::{
    mem::MaybeUninit,
    sync::{Mutex, Once},
};

use rand::rngs::mock::StepRng;
use solana_sdk::{
    account_info::AccountInfo,
    instruction::InstructionError,
    program_error::ProgramError,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, TransactionError},
};

#[doc(hidden)]
pub mod runtime;

pub mod solana_rpc_api;

pub use runtime::{Entrypoint, TestRuntime};
pub use solana_rpc_api::{RpcConnection, SolanaRpcClient};

pub type EntryFn =
    Box<dyn Fn(&Pubkey, &[AccountInfo], &[u8]) -> Result<(), ProgramError> + Send + Sync>;

pub async fn send_and_confirm(
    rpc: &std::sync::Arc<dyn SolanaRpcClient>,
    instructions: &[solana_sdk::instruction::Instruction],
    signers: &[&solana_sdk::signature::Keypair],
) -> Result<solana_sdk::signature::Signature, anyhow::Error> {
    let blockhash = rpc.get_latest_blockhash().await?;
    let mut signing_keypairs = vec![rpc.payer()];
    signing_keypairs.extend(signers.iter().map(|k| &**k));

    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        instructions,
        Some(&rpc.payer().pubkey()),
        &signing_keypairs,
        blockhash,
    );

    rpc.send_and_confirm_transaction(&tx).await
}

/// Generate a new wallet keypair with some initial funding
pub async fn create_wallet(
    rpc: &std::sync::Arc<dyn SolanaRpcClient>,
    lamports: u64,
) -> Result<solana_sdk::signature::Keypair, anyhow::Error> {
    let wallet = solana_sdk::signature::Keypair::new();
    let blockhash = rpc.get_latest_blockhash().await?;

    let payer = rpc.payer();
    let signers = vec![payer, &wallet];

    let tx = Transaction::new_signed_with_payer(
        &[solana_sdk::system_instruction::create_account(
            &payer.pubkey(),
            &wallet.pubkey(),
            lamports,
            0,
            &solana_sdk::system_program::ID,
        )],
        Some(&payer.pubkey()),
        &signers,
        blockhash,
    );

    rpc.send_and_confirm_transaction(&tx).await?;

    Ok(wallet)
}

/// Asserts that an error is a custom solana error with the expected code number
pub fn assert_custom_program_error<
    T: std::fmt::Debug,
    E: Into<u32> + Clone + std::fmt::Debug,
    A: Into<anyhow::Error>,
>(
    expected_error: E,
    actual_result: Result<T, A>,
) {
    let expected_num = expected_error.clone().into();
    let actual_err: Error = actual_result.expect_err("result is not an error").into();

    let actual_num = match (
        actual_err
            .downcast_ref::<ClientError>()
            .and_then(ClientError::get_transaction_error),
        actual_err.downcast_ref::<ProgramError>(),
    ) {
        (Some(TransactionError::InstructionError(_, InstructionError::Custom(n))), _) => n,
        (_, Some(ProgramError::Custom(n))) => *n,
        _ => panic!("not a custom program error: {:?}", actual_err),
    };

    assert_eq!(
        expected_num, actual_num,
        "expected error {:?} as code {} but got {}",
        expected_error, expected_num, actual_err
    )
}

#[deprecated(note = "use `assert_custom_program_error`")]
#[macro_export]
macro_rules! assert_program_error_code {
    ($code:expr, $result:expr) => {{
        let expected: u32 = $code;
        $crate::assert_custom_program_error(expected, $result)
    }};
}

#[deprecated(note = "use `assert_custom_program_error`")]
#[macro_export]
macro_rules! assert_program_error {
    ($error:expr, $result:expr) => {{
        $crate::assert_custom_program_error($error, $result)
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
