use anchor_lang::{InstructionData, ToAccountMetas};
use bonds_metadata::jet_bonds_metadata;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

pub fn authorize_crank_signer_instruction(
    crank_signer: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let data = jet_bonds_metadata::instruction::AuthorizeCrankSigner {}.data();
    let metadata_account = Pubkey::find_program_address(
        &[
            jet_bonds_metadata::seeds::CRANK_SIGNER,
            crank_signer.as_ref(),
        ],
        &jet_bonds_metadata::ID,
    )
    .0;

    let accounts = jet_bonds_metadata::accounts::AuthorizeCrankSigner {
        crank_signer: *crank_signer,
        metadata_account,
        authority: *authority,
        payer: *payer,
        system_program: solana_sdk::system_program::ID,
    }
    .to_account_metas(None);
    Instruction::new_with_bytes(jet_bonds_metadata::ID, &data, accounts)
}
