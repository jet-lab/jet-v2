use anchor_lang::prelude::*;

#[event]
pub struct TokensExchanged {
    pub bond_manager: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct TicketRedeemed {
    pub bond_manager: Pubkey,
    pub ticket_holder: Pubkey,
    pub redeemed_value: u64,
    pub maturation_timestamp: i64,
    pub redeemed_timestamp: i64,
}

#[event]
pub struct TicketsStaked {
    pub bond_manager: Pubkey,
    pub ticket_holder: Pubkey,
    pub amount: u64,
}

#[event]
pub struct TicketTransferred {
    pub ticket: Pubkey,
    pub previous_owner: Pubkey,
    pub new_owner: Pubkey,
}
