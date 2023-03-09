//! Instructions invoked by an end user, agnostic of margin accounts.

use anchor_lang::{prelude::Pubkey, InstructionData, ToAccountMetas};
use jet_fixed_term::{
    accounts::OrderbookMut, orderbook::state::OrderParams,
    tickets::instructions::StakeTicketsParams,
};
use solana_sdk::instruction::Instruction;
use spl_associated_token_account::get_associated_token_address as ata;

use crate::fixed_term::derive::*;

/// can derive keys from `owner`, else needs vault addresses
pub fn convert_tokens(
    amount: u64,
    market: Pubkey,
    owner: Pubkey,
    underlying_token_source: Option<Pubkey>,
    ticket_destination: Option<Pubkey>,
    underlying_token_mint: Pubkey,
) -> Instruction {
    let ticket_mint = ticket_mint(&market);
    let data = jet_fixed_term::instruction::ExchangeTokens { amount }.data();
    let accounts = jet_fixed_term::accounts::ExchangeTokens {
        underlying_token_vault: underlying_token_vault(&market),
        market,
        ticket_mint,
        user_ticket_vault: ticket_destination.unwrap_or_else(|| ata(&owner, &ticket_mint)),
        user_underlying_token_vault: underlying_token_source
            .unwrap_or_else(|| ata(&owner, &underlying_token_mint)),
        user_authority: owner,
        token_program: spl_token::ID,
    }
    .to_account_metas(None);
    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}

pub fn stake_tickets(
    amount: u64,
    seed: &[u8],
    market: Pubkey,
    ticket_holder: Pubkey,
    ticket_source: Option<Pubkey>,
    payer: Pubkey,
) -> Instruction {
    let deposit = term_deposit_bytes(&market, &ticket_holder, seed);
    let ticket_mint = ticket_mint(&market);
    let data = jet_fixed_term::instruction::StakeTickets {
        params: StakeTicketsParams {
            amount,
            seed: seed.to_vec(),
        },
    }
    .data();
    let accounts = jet_fixed_term::accounts::StakeTickets {
        deposit,
        market,
        ticket_holder,
        ticket_token_account: ticket_source.unwrap_or_else(|| ata(&ticket_holder, &ticket_mint)),
        ticket_mint,
        payer,
        token_program: spl_token::ID,
        system_program: solana_sdk::system_program::ID,
    }
    .to_account_metas(None);
    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}

/// see `redeem_deposit_accounts`
pub fn redeem_deposit(accounts: jet_fixed_term::accounts::RedeemDeposit) -> Instruction {
    let data = jet_fixed_term::instruction::RedeemDeposit {}.data();
    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts.to_account_metas(None))
}

pub fn redeem_deposit_accounts(
    market: Pubkey,
    owner: Pubkey,
    underlying_mint: Pubkey,
    authority: Pubkey,
    deposit: Pubkey,
    token_destination: Option<Pubkey>,
    payer: Pubkey,
) -> jet_fixed_term::accounts::RedeemDeposit {
    let token_account = token_destination.unwrap_or_else(|| ata(&owner, &underlying_mint));
    jet_fixed_term::accounts::RedeemDeposit {
        deposit,
        owner,
        authority,
        token_account,
        payer,
        market,
        underlying_token_vault: underlying_token_vault(&market),
        token_program: spl_token::ID,
    }
}

pub fn refresh_position(
    expect_price: bool,
    market: Pubkey,
    margin_account: Pubkey,
    underlying_oracle: Pubkey,
    ticket_oracle: Pubkey,
) -> Instruction {
    Instruction {
        program_id: jet_fixed_term::ID,
        accounts: jet_fixed_term::accounts::RefreshPosition {
            margin_user: margin_user(&market, &margin_account),
            market,
            margin_account,
            underlying_oracle,
            ticket_oracle,
            token_program: spl_token::ID,
        }
        .to_account_metas(None),
        data: jet_fixed_term::instruction::RefreshPosition { expect_price }.data(),
    }
}

/// see `sell_tickets_order_accounts`
pub fn sell_tickets_order(
    params: OrderParams,
    accounts: jet_fixed_term::accounts::SellTicketsOrder,
) -> Instruction {
    let data = jet_fixed_term::instruction::SellTicketsOrder { params }.data();
    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts.to_account_metas(None))
}

pub fn sell_tickets_order_accounts(
    orderbook_mut: OrderbookMut,
    authority: Pubkey,
    underlying_mint: &Pubkey,
    ticket_source: Option<Pubkey>,
    token_destination: Option<Pubkey>,
) -> jet_fixed_term::accounts::SellTicketsOrder {
    let ticket_mint = ticket_mint(&orderbook_mut.market);
    jet_fixed_term::accounts::SellTicketsOrder {
        authority,
        user_ticket_vault: ticket_source.unwrap_or_else(|| ata(&authority, &ticket_mint)),
        user_token_vault: token_destination.unwrap_or_else(|| ata(&authority, underlying_mint)),
        ticket_mint,
        underlying_token_vault: underlying_token_vault(&orderbook_mut.market),
        orderbook_mut,
        token_program: spl_token::ID,
    }
}

pub fn lend_order(
    params: OrderParams,
    seed: &[u8],
    market: &Pubkey,
    authority: Pubkey,
    lender_tickets: Option<Pubkey>,
    lender_tokens: Option<Pubkey>,
    orderbook_mut: OrderbookMut,
    underlying_mint: Pubkey,
    payer: Pubkey,
) -> Instruction {
    let data = jet_fixed_term::instruction::LendOrder {
        params,
        seed: seed.to_vec(),
    }
    .data();
    let accounts = lend_order_accounts(
        params,
        seed,
        market,
        authority,
        authority,
        lender_tickets,
        lender_tokens,
        orderbook_mut,
        underlying_mint,
        payer,
    );
    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts.to_account_metas(None))
}

pub fn lend_order_accounts(
    params: OrderParams,
    seed: &[u8],
    market: &Pubkey,
    user: Pubkey,
    authority: Pubkey,
    lender_tickets: Option<Pubkey>,
    lender_tokens: Option<Pubkey>,
    orderbook_mut: OrderbookMut,
    underlying_mint: Pubkey,
    payer: Pubkey,
) -> jet_fixed_term::accounts::LendOrder {
    let ticket_mint = ticket_mint(market);
    let lender_tickets = lender_tickets.unwrap_or_else(|| ata(&authority, &ticket_mint));
    let lender_tokens = lender_tokens.unwrap_or_else(|| ata(&authority, &underlying_mint));
    let deposit = term_deposit_bytes(market, &user, seed);
    jet_fixed_term::accounts::LendOrder {
        authority,
        ticket_settlement: if params.auto_stake {
            deposit
        } else {
            lender_tickets
        },
        lender_tokens,
        underlying_token_vault: underlying_token_vault(market),
        ticket_mint,
        payer,
        orderbook_mut,
        token_program: spl_token::ID,
        system_program: solana_sdk::system_program::ID,
    }
}

pub fn cancel_order(order_id: u128, owner: Pubkey, orderbook_mut: OrderbookMut) -> Instruction {
    let data = jet_fixed_term::instruction::CancelOrder { order_id }.data();
    let accounts = jet_fixed_term::accounts::CancelOrder {
        owner,
        orderbook_mut,
    }
    .to_account_metas(None);

    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}
