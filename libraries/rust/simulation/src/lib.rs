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
use lazy_static::__Deref;
use solana_client::client_error::ClientError;
use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use rand::rngs::mock::StepRng;
use solana_sdk::{
    account_info::AccountInfo, instruction::InstructionError, program_error::ProgramError,
    pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::TransactionError,
};

mod log;
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

pub trait Keygen: Send + Sync {
    fn generate_key(&self) -> Keypair;
}
impl Keygen for Arc<dyn Keygen> {
    fn generate_key(&self) -> Keypair {
        self.deref().generate_key()
    }
}

#[derive(Clone)]
pub struct DeterministicKeygen(Arc<Mutex<RefCell<MockRng>>>);
impl DeterministicKeygen {
    pub fn new(seed: &str) -> Self {
        Self(Arc::new(Mutex::new(RefCell::new(MockRng(StepRng::new(
            hash(seed),
            1,
        ))))))
    }
}

impl Keygen for DeterministicKeygen {
    fn generate_key(&self) -> Keypair {
        Keypair::generate(&mut *self.0.lock().unwrap().borrow_mut())
    }
}

#[derive(Clone)]
pub struct RandomKeygen;
impl Keygen for RandomKeygen {
    fn generate_key(&self) -> Keypair {
        Keypair::new()
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

pub fn hash<T: std::hash::Hash + ?Sized>(item: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    item.hash(&mut hasher);
    std::hash::Hasher::finish(&hasher)
}
