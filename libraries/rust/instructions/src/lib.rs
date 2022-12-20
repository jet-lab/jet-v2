pub mod airspace;
pub mod control;
pub mod fixed_term;
pub mod margin;
pub mod margin_pool;
pub mod margin_swap;

/// Instruction builder for the protocol test service
pub mod test_service;

use solana_sdk::pubkey::Pubkey;

/// Get the address of a [jet_metadata] account.
///
/// Metadata addresses are PDAs of various metadata types. Refer to `jet_metadata` for
/// the different account types.
pub fn get_metadata_address(address: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[address.as_ref()], &jet_metadata::ID).0
}
