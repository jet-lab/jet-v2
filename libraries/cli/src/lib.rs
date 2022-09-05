use anchor_client::solana_sdk::program_pack::Pack;
use anchor_client::solana_sdk::system_instruction;
use anchor_client::solana_sdk::transaction::Transaction;
use anchor_client::{
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair},
        signer::Signer,
    },
};
use clap::Parser;
use jet_bonds::orderbook::state::{EVENT_QUEUE_LEN, ORDERBOOK_SLAB_LEN};
use jet_bonds_lib::transactions::{
    authorize_crank_signer_transaction, initialize_bond_manager_transaction,
    initialize_bond_ticket_account_transaction, initialize_event_queue_transaction,
    initialize_orderbook_slab_transaction, initialize_orderbook_transaction,
    initialize_orderbook_user_transaction,
};
use spl_token::state::Mint;

#[derive(Debug, Parser)]
pub struct Opts {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    DeployKeypairAccount {
        #[clap(long, short)]
        url: String,
        #[clap(long, short)]
        payer: String,
        #[clap(subcommand)]
        account: KeypairAccount,
    },
    GeneratePubkey {
        #[clap(subcommand)]
        account: BondsAccount,
    },
    InitializeProgramState {
        #[clap(long, short)]
        url: String,
        #[clap(long, short)]
        payer: String,
        #[clap(subcommand)]
        state: ProgramState,
    },
}

#[derive(Debug, Parser)]
enum KeypairAccount {
    EventQueue {
        #[clap(long)]
        keypair_file: Option<String>,
    },
    OrderbookSlab {
        #[clap(long)]
        keypair_file: Option<String>,
    },
    TestMint {
        #[clap(long)]
        keypair_file: Option<String>,
    },
}

#[derive(Debug, Parser)]
enum BondsAccount {
    BondManager {
        #[clap(long)]
        mint: Pubkey,
        #[clap(long)]
        seed: u64,
    },
}

#[derive(Debug, Parser)]
enum ProgramState {
    BondManager {
        #[clap(long)]
        authority: String,
        #[clap(long)]
        mint: Pubkey,
        #[clap(long)]
        seed: u64,
        #[clap(long)]
        version_tag: u64,
        #[clap(long)]
        conversion_decimals: i8,
        #[clap(long)]
        duration: i64,
    },
    Orderbook {
        #[clap(long)]
        authority: String,
        #[clap(long)]
        bond_manager_key: Pubkey,
        #[clap(long)]
        event_queue_key: Pubkey,
        #[clap(long)]
        bids_key: Pubkey,
        #[clap(long)]
        asks_key: Pubkey,
        #[clap(long)]
        minimum_order_size: u64,
    },
    CrankMetadata {
        #[clap(long)]
        authority: String,
        #[clap(long)]
        crank_signer: String,
    },
    OrderbookUser {
        #[clap(long)]
        user_keypair: String,
        #[clap(long)]
        bond_manager_key: Pubkey,
    },
    TicketsUser {
        #[clap(long)]
        user_keypair: String,
        #[clap(long)]
        bond_manager_key: Pubkey,
    },
}

fn parse_url(url: String) -> String {
    match url {
        _ if url == "localhost" => String::from("http://127.0.0.1:8899"),
        _ if url == "devnet" => String::from("https://api.devnet.solana.com/"),
        _ => url,
    }
}

fn deploy_keypair_account(client: RpcClient, payer: Keypair, account: KeypairAccount) {
    let recent_blockhash = client.get_latest_blockhash().unwrap();
    match account {
        KeypairAccount::EventQueue { keypair_file } => {
            let keypair = match keypair_file {
                Some(fp) => read_keypair_file(fp).unwrap(),
                None => Keypair::new(),
            };
            let rent = client
                .get_minimum_balance_for_rent_exemption(EVENT_QUEUE_LEN as usize)
                .unwrap();
            let transaction = initialize_event_queue_transaction(
                &jet_bonds::ID,
                &keypair,
                &payer,
                rent,
                recent_blockhash,
            );
            client.send_and_confirm_transaction(&transaction).unwrap();
        }
        KeypairAccount::OrderbookSlab { keypair_file } => {
            let keypair = match keypair_file {
                Some(fp) => read_keypair_file(fp).unwrap(),
                None => Keypair::new(),
            };
            let rent = client
                .get_minimum_balance_for_rent_exemption(ORDERBOOK_SLAB_LEN as usize)
                .unwrap();
            let transaction = initialize_orderbook_slab_transaction(
                &jet_bonds::ID,
                &keypair,
                &payer,
                rent,
                recent_blockhash,
            );
            client.send_and_confirm_transaction(&transaction).unwrap();
        }
        KeypairAccount::TestMint { keypair_file } => {
            let keypair = match keypair_file {
                Some(fp) => read_keypair_file(fp).unwrap(),
                None => Keypair::new(),
            };
            let rent = client
                .get_minimum_balance_for_rent_exemption(Mint::LEN)
                .unwrap();
            let recent_blockhash = client.get_latest_blockhash().unwrap();
            let transaction =
                initialize_test_mint_transaction(&keypair, &payer, 6, rent, recent_blockhash);
            client.send_and_confirm_transaction(&transaction).unwrap();
        }
    }
}

fn initialize_test_mint_transaction(
    mint_keypair: &Keypair,
    payer: &Keypair,
    decimals: u8,
    rent: u64,
    recent_blockhash: anchor_client::solana_sdk::hash::Hash,
) -> Transaction {
    let instructions = {
        let create_mint_account = {
            let space = Mint::LEN;
            system_instruction::create_account(
                &payer.pubkey(),
                &mint_keypair.pubkey(),
                rent,
                space as u64,
                &spl_token::ID,
            )
        };
        let initialize_mint = spl_token::instruction::initialize_mint(
            &spl_token::ID,
            &mint_keypair.pubkey(),
            &mint_keypair.pubkey(),
            Some(&mint_keypair.pubkey()),
            decimals,
        )
        .unwrap();

        &[create_mint_account, initialize_mint]
    };
    let signing_keypairs = &[payer, mint_keypair];
    Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}

fn generate_pubkey(account: BondsAccount) {
    match account {
        BondsAccount::BondManager { mint, seed } => {
            let pubkey = Pubkey::find_program_address(
                &[
                    b"bond_manager".as_ref(),
                    mint.as_ref(),
                    seed.to_le_bytes().as_ref(),
                ],
                &jet_bonds::ID,
            )
            .0;

            println!("{pubkey}")
        }
    }
}

fn initialize_program_state(client: RpcClient, payer: Keypair, state: ProgramState) {
    let recent_blockhash = client.get_latest_blockhash().unwrap();
    match state {
        ProgramState::BondManager {
            authority,
            mint,
            seed,
            version_tag,
            conversion_decimals,
            duration,
        } => {
            let program_authority = read_keypair_file(authority).unwrap();

            let transaction = initialize_bond_manager_transaction(
                &jet_bonds::ID,
                &mint,
                &program_authority,
                None,
                version_tag,
                duration,
                conversion_decimals,
                seed,
                &payer,
                recent_blockhash,
            );
            client.send_and_confirm_transaction(&transaction).unwrap();
        }
        ProgramState::Orderbook {
            authority,
            bond_manager_key,
            event_queue_key,
            bids_key,
            asks_key,
            minimum_order_size,
        } => {
            let program_authority = read_keypair_file(authority).unwrap();
            let transaction = initialize_orderbook_transaction(
                &jet_bonds::ID,
                &bond_manager_key,
                &event_queue_key,
                &bids_key,
                &asks_key,
                &program_authority,
                &payer,
                minimum_order_size,
                recent_blockhash,
            );
            client.send_and_confirm_transaction(&transaction).unwrap();
        }
        ProgramState::CrankMetadata {
            authority,
            crank_signer,
        } => {
            let program_authority = read_keypair_file(authority).unwrap();
            let crank = read_keypair_file(crank_signer).unwrap();

            let recent_blockhash = client.get_latest_blockhash().unwrap();
            let transaction = authorize_crank_signer_transaction(
                &crank,
                &program_authority,
                &payer,
                recent_blockhash,
            );

            client.send_and_confirm_transaction(&transaction).unwrap();
        }
        ProgramState::OrderbookUser {
            user_keypair,
            bond_manager_key,
        } => {
            let user_keypair = read_keypair_file(user_keypair).unwrap();
            let recent_blockhash = client.get_latest_blockhash().unwrap();
            let transaction = initialize_orderbook_user_transaction(
                &bond_manager_key,
                &user_keypair,
                &payer,
                recent_blockhash,
            );
            client.send_and_confirm_transaction(&transaction).unwrap();
        }
        ProgramState::TicketsUser {
            user_keypair,
            bond_manager_key,
        } => {
            let user_keypair = read_keypair_file(user_keypair).unwrap();
            let recent_blockhash = client.get_latest_blockhash().unwrap();
            let transaction = initialize_bond_ticket_account_transaction(
                &jet_bonds::ID,
                &bond_manager_key,
                &user_keypair.pubkey(),
                &payer,
                recent_blockhash,
            );
            client.send_and_confirm_transaction(&transaction).unwrap();
        }
    }
}

pub fn run(opts: Opts) {
    match opts.command {
        Command::DeployKeypairAccount {
            url,
            payer,
            account,
        } => {
            let client =
                RpcClient::new_with_commitment(parse_url(url), CommitmentConfig::confirmed());
            let payer = read_keypair_file(payer).unwrap();
            deploy_keypair_account(client, payer, account)
        }
        Command::GeneratePubkey { account } => generate_pubkey(account),
        Command::InitializeProgramState { url, payer, state } => {
            let client =
                RpcClient::new_with_commitment(parse_url(url), CommitmentConfig::confirmed());
            let payer = read_keypair_file(payer).unwrap();
            initialize_program_state(client, payer, state)
        }
    }
}
