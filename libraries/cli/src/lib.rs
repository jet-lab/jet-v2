use anyhow::Result;
use clap::{Parser, Subcommand};
use jet_bonds::orderbook::state::{EVENT_QUEUE_LEN, ORDERBOOK_SLAB_LEN};
use jet_bonds_lib::builder::BondsIxBuilder;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};

#[derive(Debug, Parser)]
pub struct Opts {
    /// The path to the signer to use (i.e. keypair or ledger-wallet)
    #[clap(global = true, long, short = 'k')]
    signer_path: Option<String>,

    /// The network endpoint to use
    #[clap(global = true, long, short = 'u')]
    rpc_endpoint: Option<String>,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    DeployMarket {
        /// Versioning information
        #[clap(long)]
        version: u64,

        /// Seed used to generate the `BondManager` pda
        #[clap(long)]
        seed: u64,

        /// Bond duration, measured in seconds
        #[clap(long)]
        duration: i64,

        /// Exponent used for conversion between tickets and tokens
        ///
        /// Ex. `3` would mean that one token is worth 10^3 tickets
        #[clap(long)]
        conversion_factor: i8,

        /// Minimum order size, in lamports, for posting to the orderbook
        #[clap(long)]
        min_base_order_size: u64,

        /// Pubkey of the token underlying this bond market
        #[clap(long)]
        underlying_mint: Pubkey,

        /// Pubkey for the authority allowed to make changes to this market
        #[clap(long)]
        program_authority: Option<Pubkey>,

        /// Path to a keypair file used to initialize the event queue account
        #[clap(long)]
        event_queue_keypair: Option<String>,

        /// Path to a keypair file used to initialize the bids account
        #[clap(long)]
        bids_keypair: Option<String>,

        /// Path to a keypair file used to initialize the asks account
        #[clap(long)]
        asks_keypair: Option<String>,
    },
}

fn execute_plan(
    client: &RpcClient,
    plan: Vec<Instruction>,
    payer: &Pubkey,
    signers: &[&Keypair],
) -> Result<()> {
    fn sign_send_instruction(
        client: &RpcClient,
        ix: Instruction,
        payer: &Pubkey,
        signers: &[&Keypair],
    ) -> Result<Signature> {
        let mut ix_signers = Vec::<&Keypair>::new();

        let msg = Message::new(&[ix], Some(payer));
        for signer in msg.signer_keys() {
            for kp in signers.clone() {
                if &kp.pubkey() == signer {
                    ix_signers.push(kp);
                }
            }
        }
        let mut tx = Transaction::new_unsigned(msg);
        tx.sign(&ix_signers, client.get_latest_blockhash()?);

        client
            .send_and_confirm_transaction(&tx)
            .map_err(anyhow::Error::from)
    }

    for ix in plan {
        sign_send_instruction(client, ix, payer, signers)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn process_deploy_market(
    client: &RpcClient,
    underlying_mint: Pubkey,
    authority: Pubkey,
    payer: Pubkey,
    underlying_oracle: Option<Pubkey>,
    ticket_oracle: Option<Pubkey>,
    eq_kp: &Keypair,
    bids_kp: &Keypair,
    asks_kp: &Keypair,
    version: u64,
    seed: u64,
    duration: i64,
    conversion_factor: i8,
    min_base_order_size: u64,
) -> Result<Vec<Instruction>> {
    let builder = BondsIxBuilder::new_from_seed(&underlying_mint, seed)
        .with_authority(&authority)
        .with_payer(&payer)
        .with_orderbook_accounts(
            Some(bids_kp.pubkey()),
            Some(asks_kp.pubkey()),
            Some(eq_kp.pubkey()),
        );
    let init_manager = builder.initialize_manager(
        version,
        seed,
        duration,
        conversion_factor,
        &underlying_mint,
        &underlying_oracle.unwrap_or_default(),
        &ticket_oracle.unwrap_or_default(),
    )?;
    let init_eq = {
        let rent = client.get_minimum_balance_for_rent_exemption(EVENT_QUEUE_LEN as usize)?;
        solana_sdk::system_instruction::create_account(
            &payer,
            &eq_kp.pubkey(),
            rent,
            EVENT_QUEUE_LEN as u64,
            &BondsIxBuilder::jet_bonds_id(),
        )
    };
    let init_bids = {
        let rent = client.get_minimum_balance_for_rent_exemption(ORDERBOOK_SLAB_LEN as usize)?;
        solana_sdk::system_instruction::create_account(
            &payer,
            &bids_kp.pubkey(),
            rent,
            ORDERBOOK_SLAB_LEN as u64,
            &jet_bonds::ID,
        )
    };
    let init_asks = {
        let rent = client.get_minimum_balance_for_rent_exemption(ORDERBOOK_SLAB_LEN as usize)?;
        solana_sdk::system_instruction::create_account(
            &payer,
            &asks_kp.pubkey(),
            rent,
            ORDERBOOK_SLAB_LEN as u64,
            &jet_bonds::ID,
        )
    };

    let init_orderbook = builder.initialize_orderbook(&authority, min_base_order_size)?;

    let ixns = vec![init_manager, init_eq, init_asks, init_bids, init_orderbook];
    Ok(ixns)
}

pub fn run(opts: Opts) -> Result<()> {
    let rpc_endpoint = opts
        .rpc_endpoint
        .map(solana_clap_utils::input_validators::normalize_to_url_if_moniker)
        .unwrap_or("http://localhost:8899".into());
    let client = RpcClient::new_with_commitment(rpc_endpoint, CommitmentConfig::confirmed());
    let wallet = solana_clap_utils::keypair::keypair_from_path(
        &Default::default(),
        &opts
            .signer_path
            .unwrap_or_else(|| shellexpand::tilde("~/.config/solana/id.json").into()),
        "wallet",
        false,
    )
    .map_err(|_| anyhow::Error::msg("failed to retrieve signer"))?;
    let signers = &mut vec![&wallet];

    match opts.command {
        Command::DeployMarket {
            version,
            seed,
            duration,
            conversion_factor,
            min_base_order_size,
            underlying_mint,
            program_authority,
            event_queue_keypair,
            bids_keypair,
            asks_keypair,
        } => {
            let eq_kp = match event_queue_keypair {
                Some(p) => solana_clap_utils::keypair::keypair_from_path(
                    &Default::default(),
                    &p,
                    "event_queue",
                    false,
                )
                .map_err(|_| anyhow::Error::msg("failed to read keypair"))?,
                None => Keypair::new(),
            };
            let bids_kp = match bids_keypair {
                Some(p) => solana_clap_utils::keypair::keypair_from_path(
                    &Default::default(),
                    &p,
                    "event_queue",
                    false,
                )
                .map_err(|_| anyhow::Error::msg("failed to read keypair"))?,
                None => Keypair::new(),
            };
            let asks_kp = match asks_keypair {
                Some(p) => solana_clap_utils::keypair::keypair_from_path(
                    &Default::default(),
                    &p,
                    "event_queue",
                    false,
                )
                .map_err(|_| anyhow::Error::msg("failed to read keypair"))?,
                None => Keypair::new(),
            };

            signers.extend_from_slice(&[&eq_kp, &bids_kp, &asks_kp]);
            let plan = process_deploy_market(
                &client,
                underlying_mint,
                program_authority.unwrap_or_else(|| wallet.pubkey()),
                wallet.pubkey(),
                None,
                None,
                &eq_kp,
                &bids_kp,
                &asks_kp,
                version,
                seed,
                duration,
                conversion_factor,
                min_base_order_size,
            )?;
            execute_plan(&client, plan, &wallet.pubkey(), signers)?;
        }
    };

    Ok(())
}
