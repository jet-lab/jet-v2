use anchor_lang::prelude::Pubkey;

pub mod instructions;
pub mod transactions;

/// only for the bonds program
pub fn pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &jet_bonds::ID).0
}
