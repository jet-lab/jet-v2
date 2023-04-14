//! Instructions that are invoked by an end user through a margin account.

use anchor_lang::{prelude::Pubkey, InstructionData, ToAccountMetas};
use solana_sdk::instruction::Instruction;
use spl_associated_token_account::get_associated_token_address as ata;

use jet_fixed_term::{
    accounts::OrderbookMut, margin::state::AutoRollConfig, orderbook::state::OrderParams,
};

use crate::fixed_term::derive::*;
use crate::margin::derive_token_config;

use super::lend_order_accounts;

pub fn initialize_margin_user(
    margin_account: Pubkey,
    market: Pubkey,
    airspace: Pubkey,
    payer: Pubkey,
) -> Instruction {
    let ticket_collateral_mint = ticket_collateral_mint(&market);
    let underlying_collateral_mint = underlying_collateral_mint(&market);
    let claims_mint = claims_mint(&market);
    let margin_user = margin_user(&market, &margin_account);
    let accounts = jet_fixed_term::accounts::InitializeMarginUser {
        market,
        payer,
        margin_user,
        margin_account,
        claims: user_claims(&margin_user),
        ticket_collateral: user_ticket_collateral(&margin_user),
        underlying_collateral: user_underlying_collateral(&margin_user),
        claims_mint,
        ticket_collateral_mint,
        underlying_collateral_mint,
        rent: solana_sdk::sysvar::rent::ID,
        token_program: spl_token::ID,
        system_program: solana_sdk::system_program::ID,
        claims_metadata: derive_token_config(&airspace, &claims_mint),
        ticket_collateral_metadata: derive_token_config(&airspace, &ticket_collateral_mint),
        underlying_collateral_metadata: derive_token_config(&airspace, &underlying_collateral_mint),
    }
    .to_account_metas(None);
    Instruction::new_with_bytes(
        jet_fixed_term::ID,
        &jet_fixed_term::instruction::InitializeMarginUser {}.data(),
        accounts,
    )
}

/// see `user::redeem_deposit_accounts`
pub fn margin_redeem_deposit(
    market: &Pubkey,
    margin_user: Pubkey,
    accounts: jet_fixed_term::accounts::RedeemDeposit,
) -> Instruction {
    let data = jet_fixed_term::instruction::MarginRedeemDeposit {}.data();
    let accounts = jet_fixed_term::accounts::MarginRedeemDeposit {
        ticket_collateral: user_ticket_collateral(&margin_user),
        ticket_collateral_mint: ticket_collateral_mint(market),
        margin_user,
        inner: accounts,
    }
    .to_account_metas(None);
    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}

/// see `user::sell_tickets_order_accounts`
pub fn margin_sell_tickets_order(
    params: OrderParams,
    margin_user: Pubkey,
    inner: jet_fixed_term::accounts::SellTicketsOrder,
) -> Instruction {
    let data = jet_fixed_term::instruction::MarginSellTicketsOrder { params }.data();
    let accounts = jet_fixed_term::accounts::MarginSellTicketsOrder {
        ticket_collateral: user_ticket_collateral(&margin_user),
        ticket_collateral_mint: ticket_collateral_mint(&inner.orderbook_mut.market),
        inner,
        margin_user,
    }
    .to_account_metas(None);
    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}

pub fn margin_borrow_order(
    params: OrderParams,
    debt_seqno: u64,
    orderbook_mut: OrderbookMut,
    margin_account: Pubkey,
    underlying_mint: &Pubkey,
    payer: Pubkey,
) -> Instruction {
    let margin_user = margin_user(&orderbook_mut.market, &margin_account);
    let data = jet_fixed_term::instruction::MarginBorrowOrder { params }.data();
    let accounts = jet_fixed_term::accounts::MarginBorrowOrder {
        claims: user_claims(&margin_user),
        term_loan: term_loan(&orderbook_mut.market, &margin_user, debt_seqno),
        claims_mint: claims_mint(&orderbook_mut.market),
        underlying_collateral: user_underlying_collateral(&margin_user),
        underlying_collateral_mint: underlying_collateral_mint(&orderbook_mut.market),
        underlying_token_vault: underlying_token_vault(&orderbook_mut.market),
        underlying_settlement: ata(&margin_account, underlying_mint),
        fee_vault: fee_vault(&orderbook_mut.market),
        orderbook_mut,
        margin_account,
        margin_user,
        payer,
        token_program: spl_token::ID,
        system_program: solana_sdk::system_program::ID,
    }
    .to_account_metas(None);

    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}

pub fn margin_lend_order(
    params: OrderParams,
    deposit_seqno: u64,
    market: Pubkey,
    margin_account: Pubkey,
    lender_tokens: Option<Pubkey>,
    orderbook_mut: OrderbookMut,
    underlying_mint: Pubkey,
    payer: Pubkey,
) -> Instruction {
    let margin_user = margin_user(&market, &margin_account);
    let data = jet_fixed_term::instruction::MarginLendOrder { params }.data();
    let inner = lend_order_accounts(
        params,
        &deposit_seqno.to_le_bytes(),
        &market,
        margin_account,
        Some(ata(&margin_account, &ticket_mint(&market))),
        lender_tokens,
        orderbook_mut,
        underlying_mint,
        payer,
    );
    let accounts = jet_fixed_term::accounts::MarginLendOrder {
        ticket_collateral: user_ticket_collateral(&margin_user),
        ticket_collateral_mint: ticket_collateral_mint(&inner.orderbook_mut.market),
        inner,
        margin_user,
    }
    .to_account_metas(None);
    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}

pub fn margin_repay(
    term_loan_seqno: u64,
    amount: u64,
    market: Pubkey,
    source_authority: Pubkey,
    payer: Pubkey,
    margin_user: Pubkey,
    source: Pubkey,
) -> Instruction {
    let data = jet_fixed_term::instruction::Repay { amount }.data();
    let accounts = jet_fixed_term::accounts::Repay {
        term_loan: term_loan(&market, &margin_user, term_loan_seqno),
        next_term_loan: term_loan(&market, &margin_user, term_loan_seqno + 1),
        claims: user_claims(&margin_user),
        claims_mint: claims_mint(&market),
        market,
        margin_user,
        source,
        payer,
        source_authority,
        underlying_token_vault: underlying_token_vault(&market),
        token_program: spl_token::ID,
    }
    .to_account_metas(None);

    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}

pub fn configure_auto_roll(
    market: Pubkey,
    margin_account: Pubkey,
    margin_user: Pubkey,
    config: AutoRollConfig,
) -> Instruction {
    let accounts = jet_fixed_term::accounts::ConfigureAutoRoll {
        margin_user,
        margin_account,
        market,
    }
    .to_account_metas(None);

    match config {
        AutoRollConfig::Borrow(config) => Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &jet_fixed_term::instruction::ConfigureAutoRollBorrow { config }.data(),
            accounts,
        ),

        AutoRollConfig::Lend(config) => Instruction::new_with_bytes(
            jet_fixed_term::ID,
            &jet_fixed_term::instruction::ConfigureAutoRollLend { config }.data(),
            accounts,
        ),
    }
}

pub fn stop_auto_roll_deposit(margin_account: Pubkey, deposit: Pubkey) -> Instruction {
    let accounts = jet_fixed_term::accounts::StopAutoRollDeposit {
        margin_account,
        deposit,
    }
    .to_account_metas(None);

    Instruction::new_with_bytes(
        jet_fixed_term::ID,
        &jet_fixed_term::instruction::StopAutoRollDeposit {}.data(),
        accounts,
    )
}

pub fn stop_auto_roll_loan(
    margin_account: Pubkey,
    margin_user: Pubkey,
    loan: Pubkey,
) -> Instruction {
    let accounts = jet_fixed_term::accounts::StopAutoRollLoan {
        margin_account,
        margin_user,
        loan,
    }
    .to_account_metas(None);

    Instruction::new_with_bytes(
        jet_fixed_term::ID,
        &jet_fixed_term::instruction::StopAutoRollLoan {}.data(),
        accounts,
    )
}

pub fn auto_roll_lend_order(
    deposit_seqno: u64,
    margin_account: Pubkey,
    deposit: Pubkey,
    rent_receiver: Pubkey,
    orderbook_mut: OrderbookMut,
    payer: Pubkey,
) -> Instruction {
    let market = &orderbook_mut.market;
    let margin_user = margin_user(market, &margin_account);
    let data = jet_fixed_term::instruction::AutoRollLendOrder {}.data();
    let accounts = jet_fixed_term::accounts::AutoRollLendOrder {
        deposit,
        new_deposit: term_deposit(market, &margin_account, deposit_seqno + 1),
        ticket_collateral: user_ticket_collateral(&margin_user),
        ticket_collateral_mint: ticket_collateral_mint(market),
        ticket_mint: ticket_mint(market),
        underlying_token_vault: underlying_token_vault(market),
        rent_receiver,
        payer,
        margin_account,
        margin_user,
        orderbook_mut,
        token_program: spl_token::ID,
        system_program: solana_sdk::system_program::ID,
    }
    .to_account_metas(None);

    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}

pub fn auto_roll_borrow_order(
    next_debt_seqno: u64,
    margin_account: Pubkey,
    loan: Pubkey,
    rent_receiver: Pubkey,
    orderbook_mut: OrderbookMut,
    payer: Pubkey,
) -> Instruction {
    let market = &orderbook_mut.market;
    let margin_user = margin_user(market, &margin_account);
    let data = jet_fixed_term::instruction::AutoRollBorrowOrder {}.data();
    let accounts = jet_fixed_term::accounts::AutoRollBorrowOrder {
        margin_user,
        margin_account,
        loan,
        new_loan: term_loan(market, &margin_user, next_debt_seqno),
        claims: user_claims(&margin_user),
        claims_mint: claims_mint(market),
        underlying_collateral: user_underlying_collateral(&margin_user),
        underlying_collateral_mint: underlying_collateral_mint(market),
        underlying_token_vault: underlying_token_vault(market),
        fee_vault: fee_vault(market),
        rent_receiver,
        orderbook_mut,
        payer,
        token_program: spl_token::ID,
        system_program: solana_sdk::system_program::ID,
    }
    .to_account_metas(None);

    Instruction::new_with_bytes(jet_fixed_term::ID, &data, accounts)
}

// fn margin_lend_order_accounts(
//     &self,
//     margin_account: Pubkey,
//     lender_tokens: Option<Pubkey>,
//     params: OrderParams,
//     deposit_seqno: u64,
// ) -> jet_fixed_term::accounts::MarginLendOrder {
//     let margin_user = self.margin_user(margin_account);
//     jet_fixed_term::accounts::MarginLendOrder {
//         margin_user: margin_user.address,
//         ticket_collateral: margin_user.ticket_collateral,
//         ticket_collateral_mint: self.ticket_collateral,
//         inner: self.lend_order_accounts(
//             margin_account,
//             Some(self.term_deposit_key(&margin_account, &deposit_seqno.to_le_bytes())),
//             None,
//             lender_tokens,
//             params,
//             &deposit_seqno.to_le_bytes(),
//         ),
//     }
// }
