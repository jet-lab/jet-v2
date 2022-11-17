use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use anchor_lang::{AccountDeserialize, Discriminator};
use anyhow::{bail, Result};
use clap::Parser;

use solana_clap_utils::input_validators::normalize_to_url_if_moniker;
use solana_cli_config::{Config as SolanaConfig, CONFIG_FILE as SOLANA_CONFIG_FILE};
use solana_client::{
    client_error::ClientErrorKind,
    nonblocking::rpc_client::RpcClient,
    rpc_request::{RpcError, RpcResponseErrorData},
};
use solana_sdk::{
    commitment_config::CommitmentConfig, hash::Hash, instruction::Instruction, program_pack::Pack,
    pubkey, pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction,
    sysvar::rent::Rent, transaction::Transaction,
};

use pyth_sdk_solana::state::ProductAccount;

use jet_simulation::solana_rpc_api::{RpcConnection, SolanaRpcClient};

const PYTH_DEVNET_PROGRAM: Pubkey = pubkey!("gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s");
const PYTH_MAINNET_PROGRAM: Pubkey = pubkey!("FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH");

const MAINNET_HASH: &str = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d";
const DEVNET_HASH: &str = "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG";

#[derive(Parser, Debug)]
pub struct CliOpts {
    /// The network endpoint to use for reading price oracles
    #[clap(long, short = 's')]
    pub source_endpoint: String,

    /// The network endpoint to publish prices onto
    #[clap(long, short = 't')]
    pub target_endpoint: String,

    /// The keypair to use for signing price updates
    #[clap(long, short = 'k')]
    pub keypair_path: Option<String>,

    /// The interval to refresh prices
    #[clap(long,
           short = 'i',
           parse(try_from_str = parse_interval_duration),
           default_value_t = default_interval_duration()
    )]
    pub interval: humantime::Duration,
}

#[tokio::main]
async fn main() {
    let opts = CliOpts::parse();

    if let Err(e) = run(opts).await {
        println!("error: ");

        for err in e.chain() {
            println!("{err}");
        }

        println!("{}", e.backtrace());
    }
}

async fn run(opts: CliOpts) -> Result<()> {
    let source_endpoint = normalize_to_url_if_moniker(opts.source_endpoint);
    let target_endpoint = normalize_to_url_if_moniker(opts.target_endpoint);
    let keypair_path = opts.keypair_path.unwrap_or_else(|| {
        let solana_config =
            SolanaConfig::load(SOLANA_CONFIG_FILE.as_ref().unwrap()).unwrap_or_default();

        solana_config.keypair_path
    });

    let keypair_path = PathBuf::from(keypair_path);

    if !keypair_path.exists() {
        bail!("no keypair to use at {}", keypair_path.display())
    }

    let signer_data_json = std::fs::read_to_string(keypair_path)?;
    let signer_data: Vec<u8> = serde_json::from_str(&signer_data_json)?;
    let signer = Keypair::from_bytes(&signer_data)?;

    let source_client =
        RpcClient::new_with_commitment(source_endpoint, CommitmentConfig::processed());
    let target_client =
        RpcClient::new_with_commitment(target_endpoint.clone(), CommitmentConfig::processed());
    let target_sdk_client = Arc::new(RpcConnection::new(
        Keypair::from_bytes(&signer_data)?,
        RpcClient::new_with_commitment(target_endpoint, CommitmentConfig::processed()),
    )) as Arc<dyn SolanaRpcClient>;

    let oracle_list = discover_oracles(&source_client, &target_client).await?;
    let pool_list = discover_pools(&target_sdk_client, &oracle_list).await?;

    let mut id_file = None;

    loop {
        sync_oracles(&source_client, &target_client, &signer, &oracle_list).await?;
        sync_pool_balances(&target_client, &signer, &pool_list).await?;

        if id_file.is_none() {
            id_file = Some(RunningProcessIdFile::new());
        }

        tokio::time::sleep(opts.interval.into()).await;
    }
}

struct OracleInfo {
    source_oracle: Pubkey,
    target_mint: Pubkey,
    price_ratio: f64,
}

fn get_scratch_address(owner: &Pubkey, token: &Pubkey) -> Pubkey {
    Pubkey::create_with_seed(owner, &token.to_string()[..31], &spl_token::ID).unwrap()
}

fn create_scratch_account_ix(owner: &Pubkey, token: &Pubkey) -> Vec<Instruction> {
    let address = get_scratch_address(owner, token);
    let rent = Rent::default().minimum_balance(spl_token::state::Account::LEN);

    vec![
        system_instruction::create_account_with_seed(
            owner,
            &address,
            owner,
            &token.to_string()[..31],
            rent,
            spl_token::state::Account::LEN as u64,
            &spl_token::ID,
        ),
        spl_token::instruction::initialize_account(&spl_token::ID, &address, token, owner).unwrap(),
    ]
}

async fn sync_pool_balances(
    target: &RpcClient,
    signer: &Keypair,
    pools: &[(Pubkey, Pubkey)],
) -> Result<()> {
    for (token_a, token_b) in pools {
        let mut instructions = vec![];

        let scratch_a = get_scratch_address(&signer.pubkey(), token_a);
        let scratch_b = get_scratch_address(&signer.pubkey(), token_b);

        if target.get_balance(&scratch_a).await? == 0 {
            instructions.extend(create_scratch_account_ix(&signer.pubkey(), token_a));
        }
        if target.get_balance(&scratch_b).await? == 0 {
            instructions.extend(create_scratch_account_ix(&signer.pubkey(), token_b));
        }

        instructions.push(
            jet_margin_sdk::ix_builder::test_service::spl_swap_pool_balance(
                token_a,
                token_b,
                &scratch_a,
                &scratch_b,
                &signer.pubkey(),
            ),
        );

        let balance_tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&signer.pubkey()),
            &[signer],
            target.get_latest_blockhash().await?,
        );

        match target.send_and_confirm_transaction(&balance_tx).await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{e}");

                if let ClientErrorKind::RpcError(RpcError::RpcResponseError {
                    data: RpcResponseErrorData::SendTransactionPreflightFailure(failure),
                    ..
                }) = e.kind()
                {
                    eprintln!("{:#?}", failure.logs);
                }
            }
        }
    }

    Ok(())
}

async fn sync_oracles(
    source: &RpcClient,
    target: &RpcClient,
    signer: &Keypair,
    oracles: &[OracleInfo],
) -> Result<()> {
    for oracle in oracles {
        let source_account = source.get_account_data(&oracle.source_oracle).await?;
        let source_price = pyth_sdk_solana::state::load_price_account(&source_account)?;

        let update_target_ix = jet_margin_sdk::ix_builder::test_service::token_update_pyth_price(
            &signer.pubkey(),
            &oracle.target_mint,
            (source_price.agg.price as f64 * oracle.price_ratio) as i64,
            source_price.agg.conf as i64,
            source_price.expo,
        );

        let recent_blockhash = target.get_latest_blockhash().await?;
        let update_price_tx = Transaction::new_signed_with_payer(
            &[update_target_ix],
            Some(&signer.pubkey()),
            &[signer],
            recent_blockhash,
        );

        match target.send_and_confirm_transaction(&update_price_tx).await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{e}");

                if let ClientErrorKind::RpcError(RpcError::RpcResponseError {
                    data: RpcResponseErrorData::SendTransactionPreflightFailure(failure),
                    ..
                }) = e.kind()
                {
                    eprintln!("{:#?}", failure.logs);
                }
            }
        }
    }

    Ok(())
}

async fn discover_pools(
    target: &Arc<dyn SolanaRpcClient>,
    oracles: &[OracleInfo],
) -> Result<Vec<(Pubkey, Pubkey)>> {
    let supported_mints = HashSet::from_iter(oracles.iter().map(|o| o.target_mint));
    let result = jet_margin_sdk::spl_swap::SplSwapPool::get_pools(
        target,
        &supported_mints,
        spl_token_swap::ID,
    )
    .await?;

    println!("found {} pools", result.len());

    Ok(result.keys().cloned().collect())
}

async fn discover_oracles(source: &RpcClient, target: &RpcClient) -> Result<Vec<OracleInfo>> {
    use jet_margin_sdk::jet_test_service::state::TokenInfo;

    // Find all the tokens in the target in need of price data
    let target_test_accounts = target
        .get_program_accounts(&jet_margin_sdk::jet_test_service::ID)
        .await?;

    let target_token_infos = target_test_accounts
        .clone()
        .into_iter()
        .filter_map(|(address, account)| {
            let discriminator = jet_margin_sdk::jet_test_service::state::TokenInfo::discriminator();

            if account.data[..8] != discriminator {
                return None;
            }

            let info = TokenInfo::try_deserialize(&mut &account.data[..]).unwrap();

            Some((address, info))
        })
        .collect::<Vec<_>>();

    // Load all the pyth products available in the source network
    let pyth_program_id = get_pyth_program_id(source).await?;
    let pyth_accounts = source.get_program_accounts(&pyth_program_id).await?;

    let pyth_products = pyth_accounts
        .into_iter()
        .filter_map(|(address, account)| {
            if account.data.len() != pyth_sdk_solana::state::PROD_ACCT_SIZE {
                return None;
            }

            pyth_sdk_solana::state::load_product_account(&account.data)
                .map(|deserialized| (address, *deserialized))
                .ok()
        })
        .collect::<Vec<_>>();

    println!("found {} products in source", pyth_products.len());
    println!("found {} tokens in target", target_token_infos.len());

    let oracle_matches = target_token_infos
        .into_iter()
        .filter_map(|(_, info)| {
            pyth_products.iter().find_map(|(_, product)| {
                match (product.get_attr("quote_currency"), product.get_attr("base")) {
                    (Some(quote), Some(base)) if quote == "USD" && base == info.source_symbol => {
                        println!("matched oracle for {} with {base}/{quote}", info.name);

                        Some(OracleInfo {
                            source_oracle: product.px_acc,
                            target_mint: info.mint,
                            price_ratio: info.price_ratio,
                        })
                    }

                    _ => None,
                }
            })
        })
        .collect::<Vec<_>>();

    println!("found {} matching products", oracle_matches.len());

    Ok(oracle_matches)
}

async fn get_pyth_program_id(rpc: &RpcClient) -> Result<Pubkey> {
    let network_kind = NetworkKind::get_from_rpc(rpc).await?;

    Ok(match network_kind {
        NetworkKind::Mainnet => PYTH_MAINNET_PROGRAM,
        NetworkKind::Devnet => PYTH_DEVNET_PROGRAM,
        NetworkKind::Localnet => panic!("no pyth program supported on localnet"),
    })
}

#[derive(Debug, Eq, PartialEq)]
enum NetworkKind {
    Mainnet,
    Devnet,
    Localnet,
}

impl NetworkKind {
    async fn get_from_rpc(rpc: &RpcClient) -> Result<Self> {
        let mainnet_hash = Hash::from_str(MAINNET_HASH).unwrap();
        let devnet_hash = Hash::from_str(DEVNET_HASH).unwrap();

        let network_hash = rpc.get_genesis_hash().await?;

        Ok(match network_hash {
            hash if hash == mainnet_hash => NetworkKind::Mainnet,
            hash if hash == devnet_hash => NetworkKind::Devnet,
            _ => NetworkKind::Localnet,
        })
    }
}

fn parse_interval_duration(arg: &str) -> Result<humantime::Duration> {
    Ok(arg.parse::<humantime::Duration>().map(Into::into)?)
}

fn default_interval_duration() -> humantime::Duration {
    std::time::Duration::from_secs(5).into()
}

trait PythAttributeGetter {
    fn get_attr(&self, name: &str) -> Option<&str>;
}

impl PythAttributeGetter for ProductAccount {
    fn get_attr(&self, name: &str) -> Option<&str> {
        self.iter().find(|(k, _)| *k == name).map(|(_, v)| v)
    }
}

struct RunningProcessIdFile;

impl RunningProcessIdFile {
    const PATH: &'static str = "tests/oracle-mirror.pid";

    fn new() -> Self {
        let pid = std::process::id();
        std::fs::write(Self::PATH, pid.to_string()).unwrap();

        Self
    }
}

impl Drop for RunningProcessIdFile {
    fn drop(&mut self) {
        let file = Path::new(Self::PATH);

        if file.exists() {
            std::fs::remove_file(file).unwrap()
        }
    }
}
