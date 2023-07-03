use jet_environment::lookup_tables::resolve_lookup_tables;
use jet_instructions::test_service::{openbook_market_cancel_orders, openbook_market_make};
use solana_sdk::{account_info::AccountInfo, pubkey::Pubkey, signature::Keypair, signer::Signer};

use jet_solana_client::{rpc::{ClientError, SolanaRpc, SolanaRpcExtra}, transaction::create_signed_transaction};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account_idempotent,
};

/// Add maker liquidity to an openbook market
pub async fn market_make(
    wallet: &Keypair,
    client: &(dyn SolanaRpc + 'static),
    lookup_authority: Pubkey,
    market_address: Pubkey,
    dex_program: Pubkey,
) -> Result<(), ClientError> {
    let Some(mut market_account) = client.get_account(&market_address).await? else {
        return Err(ClientError::AccountNotFound(market_address));
    };

    let account_info = AccountInfo::from((&market_address, &mut market_account));
    let Ok(market_info) = anchor_spl::dex::serum_dex::state::MarketStateV2::load(&account_info, &dex_program) else {
        return Err(ClientError::Other(format!("failed to deserialize market state at {market_address}")));
    };

    let mut instructions = vec![];

    let token_base = Pubkey::new_from_array(bytemuck::cast(market_info.coin_mint));
    let token_quote = Pubkey::new_from_array(bytemuck::cast(market_info.pc_mint));
    let bids = Pubkey::new_from_array(bytemuck::cast(market_info.bids));
    let asks = Pubkey::new_from_array(bytemuck::cast(market_info.asks));
    let event_queue = Pubkey::new_from_array(bytemuck::cast(market_info.event_q));
    let request_queue = Pubkey::new_from_array(bytemuck::cast(market_info.req_q));
    let scratch_base = get_associated_token_address(&wallet.pubkey(), &token_base);
    let scratch_quote = get_associated_token_address(&wallet.pubkey(), &token_quote);

    if !client.account_exists(&scratch_base).await? {
        instructions.push(create_associated_token_account_idempotent(
            &wallet.pubkey(),
            &wallet.pubkey(),
            &token_base,
            &spl_token::ID,
        ));
    }

    if !client.account_exists(&scratch_quote).await? {
        instructions.push(create_associated_token_account_idempotent(
            &wallet.pubkey(),
            &wallet.pubkey(),
            &token_quote,
            &spl_token::ID,
        ));
    }

    instructions.push(openbook_market_cancel_orders(
        &dex_program,
        &token_base,
        &token_quote,
        &scratch_base,
        &scratch_quote,
        &wallet.pubkey(),
        &bids,
        &asks,
        &event_queue,
    ));

    instructions.push(openbook_market_make(
        &dex_program,
        &token_base,
        &token_quote,
        &scratch_base,
        &scratch_quote,
        &wallet.pubkey(),
        &bids,
        &asks,
        &request_queue,
        &event_queue,
    ));

    let lookup_tables = resolve_lookup_tables(client, &lookup_authority).await?;
    let recent_blockhash = client.get_latest_blockhash().await?;
    let tx = create_signed_transaction(&instructions, wallet, &lookup_tables, recent_blockhash).unwrap();

    client.send_and_confirm_transaction(&tx).await?;

    Ok(())
}
