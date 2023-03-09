//! Instructions that are usually invoked by a crank.

use anchor_lang::{
    prelude::{AccountMeta, Pubkey},
    InstructionData, ToAccountMetas,
};
use solana_sdk::instruction::Instruction;
use spl_associated_token_account::get_associated_token_address as ata;

use super::super::derive::*;

pub fn consume_events(
    seed: &[u8],
    events: impl IntoIterator<Item = impl Into<Vec<Pubkey>>>,
    market: Pubkey,
    event_queue: Pubkey,
    crank: Pubkey,
    payer: Pubkey,
) -> Instruction {
    let mut accounts = jet_fixed_term::accounts::ConsumeEvents {
        market,
        ticket_mint: ticket_mint(&market),
        underlying_token_vault: underlying_token_vault(&market),
        fee_vault: fee_vault(&market),
        orderbook_market_state: orderbook_market_state(&market),
        event_queue,
        crank_authorization: crank_authorization(&market, &crank),
        crank,
        payer,
        system_program: solana_sdk::system_program::ID,
        token_program: spl_token::ID,
    }
    .to_account_metas(None);

    let events = events.into_iter().map(Into::into).collect::<Vec<_>>();

    accounts.extend(events.iter().flat_map(|event_accounts: &Vec<Pubkey>| {
        event_accounts.iter().map(|a| AccountMeta::new(*a, false))
    }));

    let data = jet_fixed_term::instruction::ConsumeEvents {
        num_events: events.len() as u32,
        seed_bytes: seed.to_vec(),
    }
    .data();

    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}

pub fn settle(market: Pubkey, underlying_mint: Pubkey, margin_account: Pubkey) -> Instruction {
    let margin_user = margin_user(&market, &margin_account);
    let ticket_mint = ticket_mint(&market);
    let accounts = jet_fixed_term::accounts::Settle {
        token_program: spl_token::ID,
        claims: user_claims(&margin_user),
        claims_mint: claims_mint(&market),
        ticket_collateral: user_ticket_collateral(&margin_user),
        ticket_collateral_mint: ticket_collateral_mint(&market),
        underlying_token_vault: underlying_token_vault(&market),
        underlying_settlement: ata(&margin_account, &underlying_mint),
        ticket_settlement: ata(&margin_account, &ticket_mint),
        market,
        margin_user,
        margin_account,
        ticket_mint,
    };
    Instruction::new_with_bytes(
        jet_fixed_term::ID,
        &jet_fixed_term::instruction::Settle {}.data(),
        accounts.to_account_metas(None),
    )
}
