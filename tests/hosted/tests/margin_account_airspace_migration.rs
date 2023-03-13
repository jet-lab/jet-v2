use std::sync::Arc;

use anchor_lang::AccountDeserialize;
use solana_sdk::{
    account::Account,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

use jet_instructions::margin::MarginIxBuilder;
use jet_program_common::DEFAULT_AIRSPACE;
use jet_simulation::SolanaRpcClient;

#[cfg(not(feature = "localnet"))]
#[tokio::test]
async fn can_migrate() {
    use jet_margin::MarginAccount;

    let runtime = jet_simulation::create_test_runtime!(jet_margin);
    let existing_account = include_bytes!("data/pre-airspace-account");
    let account_addr = Pubkey::new_unique();
    let payer = Keypair::new();
    let (payer, rpc): (Pubkey, Arc<dyn SolanaRpcClient>) =
        (payer.pubkey(), Arc::new(runtime.rpc(payer)));

    runtime.set_account(
        &account_addr,
        &Account {
            data: existing_account.to_vec(),
            lamports: 53397120,
            owner: jet_margin::ID,
            ..Account::default()
        },
    );

    rpc.airdrop(&payer, LAMPORTS_PER_SOL).await.unwrap();

    let margin_ix = MarginIxBuilder::new_for_address(DEFAULT_AIRSPACE, account_addr, payer);
    jet_simulation::send_and_confirm(&rpc, &[margin_ix.configure_account_airspace()], &[])
        .await
        .unwrap();

    let account = MarginAccount::try_deserialize(
        &mut &rpc.get_account(&account_addr).await.unwrap().unwrap().data[..],
    )
    .unwrap();

    assert_eq!(account.airspace, DEFAULT_AIRSPACE);
}
