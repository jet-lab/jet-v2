use anchor_lang::InstructionData;
use jet_bonds::orderbook::instructions::RegisterAdapterParams;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

pub fn register_adapter_instruction(
    bond_manager_key: &Pubkey,
    user_key: &Pubkey,
    owner_key: &Pubkey,
    payer_key: &Pubkey,
    num_events: u32,
) -> Instruction {
    let adapter_queue_key = Pubkey::find_program_address(
        &[
            jet_bonds::seeds::EVENT_ADAPTER,
            bond_manager_key.as_ref(),
            owner_key.as_ref(),
        ],
        &jet_bonds::ID,
    )
    .0;

    let orderbook_user = Pubkey::find_program_address(
        &[
            jet_bonds::seeds::ORDERBOOK_USER,
            bond_manager_key.as_ref(),
            user_key.as_ref(),
        ],
        &jet_bonds::ID,
    )
    .0;

    let data = jet_bonds::instruction::RegisterAdapter {
        params: RegisterAdapterParams { num_events },
    }
    .data();

    let accounts = vec![
        AccountMeta::new(adapter_queue_key, false),
        AccountMeta::new_readonly(*bond_manager_key, false),
        AccountMeta::new(orderbook_user, false),
        AccountMeta::new_readonly(*user_key, true),
        AccountMeta::new_readonly(*owner_key, true),
        AccountMeta::new(*payer_key, true),
        AccountMeta::new_readonly(system_program::ID, false),
    ];
    Instruction::new_with_bytes(jet_bonds::ID, &data, accounts)
}

pub fn pop_adapter_events_instruction(
    bonds_pid: &Pubkey,
    adapter_queue_key: &Pubkey,
    owner_key: &Pubkey,
    num_events: u32,
) -> Instruction {
    let data = jet_bonds::instruction::PopAdapterEvents { num_events }.data();
    let accounts = vec![
        AccountMeta::new(*adapter_queue_key, false),
        AccountMeta::new_readonly(*owner_key, true),
    ];

    Instruction::new_with_bytes(*bonds_pid, &data, accounts)
}
