//! Instructions that are invoked by a market authority.

use anchor_lang::{prelude::Pubkey, InstructionData, ToAccountMetas};
use jet_fixed_term::{
    control::instructions::{InitializeMarketParams, InitializeOrderbookParams},
    orderbook::state::{event_queue_len, orderbook_slab_len},
};
use solana_sdk::instruction::Instruction;
use spl_associated_token_account::{
    get_associated_token_address as ata, instruction::create_associated_token_account,
};

use crate::{
    airspace::derive_governor_id, fixed_term::MarketAdmin, test_service::if_not_initialized,
};

use super::super::{derive::*, OrderbookAddresses};

pub fn initialize_market(
    params: InitializeMarketParams,
    underlying_token_mint: Pubkey,
    airspace: Pubkey,
    authority: Pubkey,
    underlying_oracle: Pubkey,
    ticket_oracle: Pubkey,
    fee_destination: Pubkey,
    payer: Pubkey,
) -> Instruction {
    let market = market(&airspace, &underlying_token_mint, params.seed);
    let data = jet_fixed_term::instruction::InitializeMarket { params }.data();
    let accounts = jet_fixed_term::accounts::InitializeMarket {
        underlying_token_vault: underlying_token_vault(&market),
        ticket_mint: ticket_mint(&market),
        claims: claims_mint(&market),
        collateral: ticket_collateral_mint(&market),
        market,
        underlying_token_mint,
        airspace,
        authority,
        underlying_oracle,
        ticket_oracle,
        fee_destination,
        payer,
        rent: solana_sdk::sysvar::rent::ID,
        token_program: spl_token::ID,
        system_program: solana_sdk::system_program::ID,
    }
    .to_account_metas(None);
    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}

pub fn initialize_orderbook(
    min_base_order_size: u64,
    orderbook: OrderbookAddresses,
    market_admin: MarketAdmin,
    payer: Pubkey,
) -> Instruction {
    let data = jet_fixed_term::instruction::InitializeOrderbook {
        params: InitializeOrderbookParams {
            min_base_order_size,
        },
    }
    .data();
    let accounts = jet_fixed_term::accounts::InitializeOrderbook {
        orderbook_market_state: orderbook_market_state(&market_admin.market),
        market: market_admin.market,
        authority: market_admin.authority,
        airspace: market_admin.airspace,
        event_queue: orderbook.event_queue,
        bids: orderbook.bids,
        asks: orderbook.asks,
        payer,
        system_program: solana_sdk::system_program::ID,
    }
    .to_account_metas(None);
    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}

pub fn init_event_queue(queue: &Pubkey, capacity: usize, rent: u64, payer: &Pubkey) -> Instruction {
    solana_sdk::system_instruction::create_account(
        payer,
        queue,
        rent,
        event_queue_len(capacity) as u64,
        &jet_fixed_term::ID,
    )
}

pub fn init_orderbook_slab(
    slab: &Pubkey,
    capacity: usize,
    rent: u64,
    payer: &Pubkey,
) -> Instruction {
    solana_sdk::system_instruction::create_account(
        payer,
        slab,
        rent,
        orderbook_slab_len(capacity) as u64,
        &jet_fixed_term::ID,
    )
}

/// initializes the associated token account for the underlying mint owned
/// by the authority of the market. this only returns an instruction if
/// you've opted to use the default fee_destination, which is the ata for
/// the authority. otherwise this returns nothing
pub fn init_default_fee_destination(
    fee_destination: &Pubkey,
    authority: &Pubkey,
    underlying_mint: &Pubkey,
    payer: &Pubkey,
) -> Option<Instruction> {
    let ata = ata(&authority, &underlying_mint);
    if fee_destination == &ata {
        Some(if_not_initialized(
            ata,
            create_associated_token_account(payer, authority, underlying_mint, &spl_token::id()),
        ))
    } else {
        None
    }
}

pub fn pause_order_matching(
    market_admin: MarketAdmin,
    orderbook_market_state: Pubkey,
) -> Instruction {
    let data = jet_fixed_term::instruction::PauseOrderMatching {}.data();
    let accounts = jet_fixed_term::accounts::PauseOrderMatching {
        market: market_admin.market,
        authority: market_admin.authority,
        airspace: market_admin.airspace,
        orderbook_market_state,
    }
    .to_account_metas(None);

    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}

pub fn resume_order_matching(
    market_admin: MarketAdmin,
    orderbook: OrderbookAddresses,
) -> Instruction {
    let orderbook_market_state = orderbook_market_state(&market_admin.market);
    let data = jet_fixed_term::instruction::ResumeOrderMatching {}.data();
    let accounts = jet_fixed_term::accounts::ResumeOrderMatching {
        market: market_admin.market,
        authority: market_admin.authority,
        airspace: market_admin.airspace,
        orderbook_market_state,
        event_queue: orderbook.event_queue,
        bids: orderbook.bids,
        asks: orderbook.asks,
    }
    .to_account_metas(None);

    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}

pub fn pause_ticket_redemption(market_admin: MarketAdmin) -> Instruction {
    modify_market([true as u8].into(), 8 + 32 * 14 + 2, market_admin)
}

pub fn resume_ticket_redemption(market_admin: MarketAdmin) -> Instruction {
    modify_market([false as u8].into(), 8 + 32 * 14 + 2, market_admin)
}

pub fn modify_market(data: Vec<u8>, offset: u32, market_admin: MarketAdmin) -> Instruction {
    let data = jet_fixed_term::instruction::ModifyMarket { data, offset }.data();
    let accounts = jet_fixed_term::accounts::ModifyMarket {
        market: market_admin.market,
        authority: market_admin.authority,
        airspace: market_admin.airspace,
    }
    .to_account_metas(None);
    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}

pub fn authorize_crank(crank: Pubkey, market_admin: MarketAdmin, payer: Pubkey) -> Instruction {
    let data = jet_fixed_term::instruction::AuthorizeCrank {}.data();
    let accounts = jet_fixed_term::accounts::AuthorizeCrank {
        crank_authorization: crank_authorization(&market_admin.market, &crank),
        crank,
        market: market_admin.market,
        authority: market_admin.authority,
        airspace: market_admin.airspace,
        payer,
        system_program: solana_sdk::system_program::ID,
    }
    .to_account_metas(None);
    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}

pub fn recover_uninitialized(
    governor: Pubkey,
    uninitialized: Pubkey,
    recipient: Pubkey,
) -> Instruction {
    let data = jet_fixed_term::instruction::RecoverUninitialized {}.data();
    let accounts = jet_fixed_term::accounts::RecoverUninitialized {
        governor,
        governor_id: derive_governor_id(),
        uninitialized,
        recipient,
    }
    .to_account_metas(None);

    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}
