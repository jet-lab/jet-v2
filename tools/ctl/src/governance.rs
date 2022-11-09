use std::str::FromStr;

use anyhow::{bail, Context, Result};

use anchor_lang::{idl::IdlAccount, AnchorDeserialize};
use solana_client::{
    client_error::{ClientError, ClientErrorKind},
    nonblocking::rpc_client::RpcClient,
    rpc_request::RpcError,
};
use solana_sdk::{
    account_info::IntoAccountInfo,
    bpf_loader_upgradeable,
    instruction::{AccountMeta, Instruction},
    loader_upgradeable_instruction::UpgradeableLoaderInstruction,
    pubkey,
    pubkey::Pubkey,
    transaction::Transaction,
};
use spl_governance::state::{
    enums::TransactionExecutionStatus,
    governance::GovernanceV2,
    proposal::{OptionVoteResult, ProposalV2},
    proposal_transaction::{
        get_proposal_transaction_address, AccountMetaData, InstructionData, ProposalTransactionV2,
    },
    realm::RealmV2,
    token_owner_record::{get_token_owner_record_address, get_token_owner_record_data},
};

use crate::{
    anchor_ix_parser::{AnchorParser, ParsedAccountInput, ParsedInstruction},
    client::{Client, NetworkKind, Plan, TransactionEntry},
};

pub const JET_STAKING_PROGRAM: Pubkey = pubkey!("JPLockxtkngHkaQT5AuRYow3HyUv5qWzmhwsCPd653n");
pub const JET_GOVERNANCE_PROGRAM: Pubkey = pubkey!("JPGov2SBA6f7XSJF5R4Si5jEJekGiyrwP2m7gSEqLUs");
pub const JET_ENG_GOVERNANCE: Pubkey = pubkey!("7R6FjP2HfXAgKQjURC4tCBrUmRQLCgEUeX2berrfU4ox");
pub const JET_ENG_TREASURY: Pubkey = pubkey!("2J2K1wHK3U8bsow1shUZJvEx1L2og2h5T5JGPqBS1uKA");
pub const JET_DAO_GOVERNANCE: Pubkey = pubkey!("7dwYkRSBMyC2ix1q7NeoKe5YjKdezbtf9KTe4SQ2oKsW");
pub const JET_CUSTODY_GOVERNANCE: Pubkey = pubkey!("BKFjv7iwsbPtWbPNYRBozoKr5C3qAzZqc5Y9Y9X1gTqF");

pub const DEFAULT_IDLS: &[Pubkey] = &[
    pubkey!("JPMRGNgRk3w2pzBM1RLNBnpGxQYsFQ3yXKpuk4tTXVZ"),
    pubkey!("JPPooLEqRo3NCSx82EdE2VZY5vUaSsgskpZPBHNGVLZ"),
    pubkey!("JPMetawzxw7WyH3qHUVScYHWFBGhjwqDnM2R9qVbRLp"),
];

pub const JET_GOV_MAP: &[(&str, Pubkey)] = &[
    ("dao", JET_DAO_GOVERNANCE),
    ("eng", JET_ENG_GOVERNANCE),
    ("custody", JET_CUSTODY_GOVERNANCE),
];

pub fn get_governance_address_from_user_string(name: &str) -> Result<Pubkey> {
    for (entry_name, pubkey) in JET_GOV_MAP {
        if name == *entry_name {
            return Ok(*pubkey);
        }
    }

    Ok(Pubkey::from_str(name)?)
}

pub async fn get_governance_and_realm(
    client: &Client,
    address: &Pubkey,
) -> Result<(GovernanceV2, RealmV2)> {
    let governance = get_borsh_account::<GovernanceV2>(client.rpc(), address).await?;

    get_borsh_account::<RealmV2>(client.rpc(), &governance.realm)
        .await
        .map(|r| (governance, r))
}

pub async fn convert_plan_to_proposal(
    client: &Client,
    plan: Plan,
    proposal_address: Pubkey,
    proposal_option: u8,
) -> Result<Plan> {
    let proposal = get_proposal_state(client, &proposal_address).await?;
    let mut tx_next_index = proposal.options[proposal_option as usize].transactions_next_index;

    validate_proposal(&proposal)?;

    Ok(plan
        .into_iter()
        .map(|entry| {
            let instructions = get_transaction_instructions(&entry.transaction);

            let insert_tx = spl_governance::instruction::insert_transaction(
                &JET_GOVERNANCE_PROGRAM,
                &proposal.governance,
                &proposal_address,
                &proposal.token_owner_record,
                &client.signer().unwrap(),
                &client.signer().unwrap(),
                proposal_option,
                tx_next_index,
                0,
                instructions,
            );

            tx_next_index += 1;

            TransactionEntry {
                transaction: Transaction::new_with_payer(&[insert_tx], None),
                steps: entry.steps,
            }
        })
        .collect())
}

pub async fn inspect_proposal_instructions(
    client: &Client,
    proposal_address: Pubkey,
) -> Result<()> {
    let proposal: ProposalV2 = get_borsh_account(client.rpc(), &proposal_address)
        .await
        .with_context(|| "getting proposal")?;
    let mut anchor_parser = AnchorParser::new(client.rpc());

    for default_program in DEFAULT_IDLS {
        anchor_parser.load_idl(default_program).await?;
    }

    println!("transactions will have authority: {}", proposal.governance);

    for (opt_index, option) in proposal.options.iter().enumerate() {
        println!("transactions for option {}:", option.label);

        for tx_index in 0..option.transactions_next_index {
            let tx_address = get_proposal_transaction_address(
                &JET_GOVERNANCE_PROGRAM,
                &proposal_address,
                &(opt_index as u8).to_le_bytes(),
                &tx_index.to_le_bytes(),
            );

            if let Some(proposal_tx) = get_prosposal_transaction(client.rpc(), &tx_address).await? {
                let instructions = proposal_tx
                    .instructions
                    .iter()
                    .map(|ix_data| Instruction {
                        program_id: ix_data.program_id,
                        data: ix_data.data.clone(),
                        accounts: ix_data
                            .accounts
                            .iter()
                            .map(|m| AccountMeta {
                                pubkey: m.pubkey,
                                is_signer: m.is_signer,
                                is_writable: m.is_writable,
                            })
                            .collect(),
                    })
                    .collect::<Vec<_>>();

                println!("tx #{tx_index}:");

                for (ix_index, instruction) in instructions.iter().enumerate() {
                    println!("ix #{ix_index}:");

                    let parsed =
                        try_parse_instruction(client, &mut anchor_parser, instruction).await?;
                    println!("{parsed:#?}");
                }
            } else {
                println!("tx #{tx_index} not found, likely was removed: {tx_address}");
            }
        }
    }

    Ok(())
}

pub async fn get_execute_instructions(
    client: &Client,
    proposal_address: Pubkey,
) -> Result<Vec<(u16, Instruction)>> {
    let proposal = get_proposal_state(client, &proposal_address).await?;
    let mut instructions = vec![];

    for (opt_index, option) in proposal.options.iter().enumerate() {
        if option.vote_result != OptionVoteResult::Succeeded {
            continue;
        }

        for tx_index in 0..option.transactions_next_index {
            let tx_address = get_proposal_transaction_address(
                &JET_GOVERNANCE_PROGRAM,
                &proposal_address,
                &(opt_index as u8).to_le_bytes(),
                &tx_index.to_le_bytes(),
            );

            if let Some(proposal_tx) = get_prosposal_transaction(client.rpc(), &tx_address).await? {
                if proposal_tx.execution_status == TransactionExecutionStatus::Success {
                    continue;
                }

                let ix_program = proposal_tx.instructions[0].program_id;
                let ix_accounts = proposal_tx
                    .instructions
                    .iter()
                    .flat_map(|ix| ix.accounts.iter())
                    .map(|md| AccountMeta {
                        pubkey: md.pubkey,
                        is_signer: md.is_signer && md.pubkey.is_on_curve(),
                        is_writable: md.is_writable,
                    })
                    .collect::<Vec<_>>();

                instructions.push((
                    tx_index,
                    spl_governance::instruction::execute_transaction(
                        &JET_GOVERNANCE_PROGRAM,
                        &proposal.governance,
                        &proposal_address,
                        &tx_address,
                        &ix_program,
                        &ix_accounts,
                    ),
                ));
            } else {
                println!("tx #{tx_index} not found, likely was removed: {tx_address}");
            }
        }

        break;
    }

    Ok(instructions)
}

pub async fn get_clear_tx_instructions(
    client: &Client,
    address: &Pubkey,
) -> Result<Vec<(String, Instruction)>> {
    let proposal = get_proposal_state(client, address).await?;
    let mut instructions = vec![];

    for (opt_index, option) in proposal.options.iter().enumerate() {
        for tx_index in 0..option.transactions_next_index {
            let tx_address = get_proposal_transaction_address(
                &JET_GOVERNANCE_PROGRAM,
                address,
                &(opt_index as u8).to_le_bytes(),
                &tx_index.to_le_bytes(),
            );

            let description = format!("Remove Option {} Tx #{}", &option.label, tx_index);
            instructions.push((
                description,
                spl_governance::instruction::remove_transaction(
                    &JET_GOVERNANCE_PROGRAM,
                    address,
                    &proposal.token_owner_record,
                    &client.signer().unwrap(),
                    &tx_address,
                    &client.signer().unwrap(),
                ),
            ));
        }
    }

    Ok(instructions)
}

pub async fn get_proposal_state(client: &Client, address: &Pubkey) -> Result<ProposalV2> {
    let proposal_data = client.rpc().get_account_data(address).await?;
    Ok(solana_sdk::borsh::try_from_slice_unchecked(&proposal_data)?)
}

pub fn resolve_payer(client: &Client) -> Result<Pubkey> {
    Ok(match client.network_kind {
        NetworkKind::Mainnet => JET_ENG_TREASURY,
        _ => client.signer()?,
    })
}

pub async fn find_user_owner_record(
    client: &Client,
    realm: &Pubkey,
    mint: &Pubkey,
) -> Result<Pubkey> {
    let user_voter_record =
        get_token_owner_record_address(&JET_GOVERNANCE_PROGRAM, realm, mint, &client.signer()?);

    if client.account_exists(&user_voter_record).await? {
        return Ok(user_voter_record);
    }

    let all_voter_records = client
        .rpc()
        .get_program_accounts(&JET_GOVERNANCE_PROGRAM)
        .await?;

    println!("records {}", all_voter_records.len());

    for (address, account) in all_voter_records {
        match get_token_owner_record_data(
            &JET_GOVERNANCE_PROGRAM,
            &(address, account).into_account_info(),
        ) {
            Ok(record)
                if record.governance_delegate == Some(client.signer()?)
                    && record.realm == *realm =>
            {
                return Ok(address)
            }
            _ => continue,
        }
    }

    bail!("no voting power found for the current user")
}

async fn try_parse_instruction(
    client: &Client,
    anchor_parser: &mut AnchorParser<'_>,
    instruction: &Instruction,
) -> Result<ParsedInstruction> {
    let idl_account = IdlAccount::address(&instruction.program_id);

    if client.account_exists(&idl_account).await? {
        anchor_parser.load_idl(&instruction.program_id).await?;
        return anchor_parser.try_parse_instruction(instruction).await;
    }

    match instruction.program_id {
        id if id == bpf_loader_upgradeable::ID => try_parse_program_loader_instruction(instruction),
        _ => bail!(
            "unknown program {}, cannot parse instruction",
            instruction.program_id
        ),
    }
}

fn try_parse_program_loader_instruction(instruction: &Instruction) -> Result<ParsedInstruction> {
    let loader_instruction =
        bincode::deserialize::<UpgradeableLoaderInstruction>(&instruction.data)?;

    Ok(match loader_instruction {
        UpgradeableLoaderInstruction::SetAuthority => ParsedInstruction {
            program: instruction.program_id,
            name: "Set Program Authority".to_owned(),
            data: crate::anchor_ix_parser::DataValue::Struct(vec![]),
            accounts: vec![
                ParsedAccountInput::Account(
                    "Program Data".to_owned(),
                    instruction.accounts[0].pubkey,
                ),
                ParsedAccountInput::Account(
                    "Current Authority".to_owned(),
                    instruction.accounts[1].pubkey,
                ),
                ParsedAccountInput::Account(
                    "New Authority".to_owned(),
                    instruction.accounts[2].pubkey,
                ),
            ],
        },

        UpgradeableLoaderInstruction::Upgrade => ParsedInstruction {
            program: instruction.program_id,
            name: "Upgrade Program".to_owned(),
            data: crate::anchor_ix_parser::DataValue::Struct(vec![]),
            accounts: vec![
                ParsedAccountInput::Account(
                    "Program Data".to_owned(),
                    instruction.accounts[0].pubkey,
                ),
                ParsedAccountInput::Account("Program".to_owned(), instruction.accounts[1].pubkey),
                ParsedAccountInput::Account("Buffer".to_owned(), instruction.accounts[2].pubkey),
                ParsedAccountInput::Account("Spill".to_owned(), instruction.accounts[3].pubkey),
                ParsedAccountInput::Account("Authority".to_owned(), instruction.accounts[6].pubkey),
            ],
        },

        _ => bail!("unknown bpf instruction {:?}", loader_instruction),
    })
}

fn get_transaction_instructions(tx: &Transaction) -> Vec<InstructionData> {
    tx.message
        .instructions
        .iter()
        .map(|cix| {
            let accounts = cix
                .accounts
                .iter()
                .map(|a| {
                    let key = *a as usize;
                    AccountMetaData {
                        pubkey: tx.message.account_keys[key],
                        is_signer: tx.message.is_signer(key),
                        is_writable: tx.message.is_writable(key),
                    }
                })
                .collect::<Vec<_>>();

            InstructionData {
                accounts,
                data: cix.data.clone(),
                program_id: tx.message.account_keys[cix.program_id_index as usize],
            }
        })
        .collect()
}

/// Returns None for if RpcClient returns `AccountNotFound`,
/// which means the transaction was probably removed.
async fn get_prosposal_transaction(
    rpc: &RpcClient,
    address: &Pubkey,
) -> Result<Option<ProposalTransactionV2>> {
    let proposal_tx = get_borsh_account::<ProposalTransactionV2>(rpc, address).await;

    match proposal_tx {
        Ok(tx) => Ok(Some(tx)),
        Err(err) => match err.downcast_ref::<ClientError>() {
            Some(ClientError {
                request: _,
                kind: ClientErrorKind::RpcError(RpcError::ForUser(s)),
            }) if s == &format!("AccountNotFound: pubkey={}", address) => Ok(None),
            _ => Err(err),
        },
    }
}

async fn get_borsh_account<T: AnchorDeserialize>(rpc: &RpcClient, address: &Pubkey) -> Result<T> {
    let data = rpc.get_account_data(address).await?;
    solana_sdk::borsh::try_from_slice_unchecked::<T>(&data).with_context(|| {
        format!(
            "could not deserialize account {address} as type {}",
            std::any::type_name::<T>()
        )
    })
}

fn validate_proposal(proposal: &ProposalV2) -> Result<()> {
    if proposal.governance != JET_ENG_GOVERNANCE {
        bail!("proposal provided does not target the correct authority");
    }

    Ok(())
}
