use solana_sdk::{
    hash::Hash, program_pack::Pack, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction,
};
use spl_token::{instruction::initialize_mint, state::Mint};

pub fn initialize_test_mint_transaction(
    mint_keypair: &Keypair,
    payer: &Keypair,
    decimals: u8,
    rent: u64,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = {
        let create_mint_account = {
            let space = Mint::LEN;
            system_instruction::create_account(
                &payer.pubkey(),
                &mint_keypair.pubkey(),
                rent,
                space as u64,
                &spl_token::ID,
            )
        };
        let initialize_mint = initialize_mint(
            &spl_token::ID,
            &mint_keypair.pubkey(),
            &mint_keypair.pubkey(),
            Some(&mint_keypair.pubkey()),
            decimals,
        )
        .unwrap();

        &[create_mint_account, initialize_mint]
    };
    let signing_keypairs = &[payer, mint_keypair];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}
