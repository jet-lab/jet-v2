//! PDA derivation functions for the fixed term program

use anchor_lang::prelude::Pubkey;
use jet_fixed_term::seeds;

pub fn market(airspace: &Pubkey, underlying_token_mint: &Pubkey, seed: [u8; 32]) -> Pubkey {
    fixed_term_address(&[
        seeds::MARKET,
        airspace.as_ref(),
        underlying_token_mint.as_ref(),
        &seed,
    ])
}

pub fn ticket_mint(market: &Pubkey) -> Pubkey {
    fixed_term_address(&[seeds::TICKET_MINT, market.as_ref()])
}

pub fn underlying_token_vault(market: &Pubkey) -> Pubkey {
    fixed_term_address(&[seeds::UNDERLYING_TOKEN_VAULT, market.as_ref()])
}

pub fn claims_mint(market: &Pubkey) -> Pubkey {
    fixed_term_address(&[seeds::CLAIM_NOTES, market.as_ref()])
}

pub fn ticket_collateral_mint(market: &Pubkey) -> Pubkey {
    fixed_term_address(&[seeds::TICKET_COLLATERAL_NOTES, market.as_ref()])
}

pub fn fixed_term_address(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &jet_fixed_term::ID).0
}

pub fn market_from_tenor(airspace: &Pubkey, token_mint: &Pubkey, tenor: u64) -> Pubkey {
    let mut seed = [0u8; 32];
    seed[..8].copy_from_slice(&tenor.to_le_bytes());

    market(airspace, token_mint, seed)
}

pub fn margin_user(market: &Pubkey, margin_account: &Pubkey) -> Pubkey {
    fixed_term_address(&[
        jet_fixed_term::seeds::MARGIN_USER,
        market.as_ref(),
        margin_account.as_ref(),
    ])
}

pub fn term_loan(market: &Pubkey, margin_user: &Pubkey, debt_seqno: u64) -> Pubkey {
    fixed_term_address(&[
        jet_fixed_term::seeds::TERM_LOAN,
        market.as_ref(),
        margin_user.as_ref(),
        &debt_seqno.to_le_bytes(),
    ])
}

pub fn term_deposit(market: &Pubkey, owner: &Pubkey, deposit_seqno: u64) -> Pubkey {
    fixed_term_address(&[
        jet_fixed_term::seeds::TERM_DEPOSIT,
        market.as_ref(),
        owner.as_ref(),
        &deposit_seqno.to_le_bytes(),
    ])
}

pub fn term_loan_bytes(market: &Pubkey, margin_user: &Pubkey, seed: &[u8]) -> Pubkey {
    fixed_term_address(&[
        jet_fixed_term::seeds::TERM_LOAN,
        market.as_ref(),
        margin_user.as_ref(),
        &seed,
    ])
}

pub fn term_deposit_bytes(market: &Pubkey, owner: &Pubkey, seed: &[u8]) -> Pubkey {
    fixed_term_address(&[
        jet_fixed_term::seeds::TERM_DEPOSIT,
        market.as_ref(),
        owner.as_ref(),
        &seed,
    ])
}

pub fn crank_authorization(market: &Pubkey, crank: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            jet_fixed_term::seeds::CRANK_AUTHORIZATION,
            market.as_ref(),
            crank.as_ref(),
        ],
        &jet_fixed_term::ID,
    )
    .0
}

pub fn user_claims(margin_user: &Pubkey) -> Pubkey {
    fixed_term_address(&[jet_fixed_term::seeds::CLAIM_NOTES, margin_user.as_ref()])
}

pub fn user_ticket_collateral(margin_user: &Pubkey) -> Pubkey {
    fixed_term_address(&[
        jet_fixed_term::seeds::TICKET_COLLATERAL_NOTES,
        margin_user.as_ref(),
    ])
}

pub fn orderbook_market_state(market: &Pubkey) -> Pubkey {
    fixed_term_address(&[
        jet_fixed_term::seeds::ORDERBOOK_MARKET_STATE,
        market.as_ref(),
    ])
}

pub fn fee_vault(market: &Pubkey) -> Pubkey {
    fixed_term_address(&[jet_fixed_term::seeds::FEE_VAULT, market.as_ref()])
}