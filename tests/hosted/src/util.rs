use anchor_lang::prelude::ProgramError;
use solana_client::client_error::ClientError;
use solana_sdk::{instruction::InstructionError, transaction::TransactionError};

use jet_client_native::JetSimulationClientError;

/// Asserts that an error is a custom solana error with the expected code number
pub fn assert_program_error<
    T: std::fmt::Debug,
    E: Into<u32> + Clone + std::fmt::Debug,
    A: Into<anyhow::Error>,
>(
    expected_error: E,
    actual_result: Result<T, A>,
) {
    let expected_code = expected_error.clone().into();
    let actual_err: anyhow::Error = actual_result.expect_err("result is not an error").into();
    let mut actual_err_code = None;

    if let Some(JetSimulationClientError::Interface(if_error)) =
        actual_err.downcast_ref::<JetSimulationClientError>()
    {
        if let Some(client_err) = if_error.downcast_ref::<ClientError>() {
            eprintln!("{:?}", client_err);
            if let Some(TransactionError::InstructionError(_, InstructionError::Custom(n))) =
                client_err.get_transaction_error()
            {
                actual_err_code = Some(n);
            }
        }

        if let Some(ProgramError::Custom(n)) = if_error.downcast_ref::<ProgramError>() {
            actual_err_code = Some(*n);
        }
    }

    let Some(actual_err_code) = actual_err_code else {
        panic!("not a program error: {:#?}", actual_err);
    };

    assert_eq!(
        expected_code, actual_err_code,
        "expected error {:?} as code {} but got {}",
        expected_error, expected_code, actual_err
    )
}
