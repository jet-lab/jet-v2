use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
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
    account::ReadableAccount, commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction, instruction::Instruction, program_pack::Pack, pubkey,
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction, sysvar::rent::Rent,
    transaction::Transaction,
};

use pyth_sdk_solana::state::ProductAccount;

use jet_environment::builder::resolve_swap_program;
use jet_margin_sdk::swap::openbook_swap::OpenBookMarket;
use jet_program_common::programs::SABER;
use jet_simulation::solana_rpc_api::SolanaRpcClient;
use jet_solana_client::{
    network::NetworkKind,
    rpc::native::RpcConnection,
    signature::sign_versioned_transaction,
    transaction::{condense, ToTransaction, TransactionBuilder},
};

const PYTH_DEVNET_PROGRAM: Pubkey = pubkey!("gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s");
const PYTH_MAINNET_PROGRAM: Pubkey = pubkey!("FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH");

#[derive(Parser, Debug)]
pub struct CliOpts {
    /// The network endpoint to use for reading price oracles
    #[clap(long, short = 's', env = "SOURCE_RPC_URL")]
    pub source_endpoint: String,

    /// The network endpoint to publish prices onto
    #[clap(long, short = 't', env = "TARGET_RPC_URL")]
    pub target_endpoint: String,

    /// The keypair to use for signing price updates
    #[clap(long, short = 'k', env = "SIGNER_PATH")]
    pub keypair_path: Option<String>,

    /// The interval to refresh prices
    #[clap(long,
           short = 'i',
           parse(try_from_str = parse_interval_duration),
           default_value_t = default_interval_duration()
    )]
    pub interval: humantime::Duration,

    /// Don't try to sync the oracles
    #[clap(long)]
    pub no_oracle_sync: bool,

    /// Don't try to rebalance swap pools
    #[clap(long)]
    pub no_pool_sync: bool,
}

pub async fn run(opts: CliOpts) -> Result<()> {
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
    let target_sdk_client = Arc::new((
        RpcConnection::from(RpcClient::new_with_commitment(
            target_endpoint,
            CommitmentConfig::processed(),
        )),
        Keypair::from_bytes(&signer_data)?,
    )) as Arc<dyn SolanaRpcClient>;

    let spl_swap_program = get_spl_program(&target_client).await?;
    let openbook_program = get_openbook_program(&target_client).await?;

    let oracle_list = discover_oracles(&source_client, &target_client).await?;
    let spl_pool_list =
        discover_spl_pools(&target_sdk_client, &oracle_list, spl_swap_program).await?;
    let saber_pool_list = discover_saber_pools(&target_sdk_client, &oracle_list).await?;
    let openbook_market_list =
        discover_openbook_markets(&target_sdk_client, &oracle_list, openbook_program).await?;

    let mut id_file = None;

    loop {
        if !opts.no_oracle_sync {
            sync_oracles(&source_client, &target_client, &signer, &oracle_list).await?;
        }
        if !opts.no_pool_sync {
            sync_pool_balances(&target_client, &signer, &spl_pool_list, &spl_swap_program).await?;
            sync_pool_balances(&target_client, &signer, &saber_pool_list, &SABER).await?;
            replace_openbook_orders(
                &target_client,
                &signer,
                &openbook_market_list,
                &openbook_program,
            )
            .await?;
        }

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
    program: &Pubkey,
) -> Result<()> {
    for (token_a, token_b) in pools {
        let mut instructions = vec![ComputeBudgetInstruction::set_compute_unit_limit(600_000)];

        let scratch_a = get_scratch_address(&signer.pubkey(), token_a);
        let scratch_b = get_scratch_address(&signer.pubkey(), token_b);

        if target.get_balance(&scratch_a).await? == 0 {
            instructions.extend(create_scratch_account_ix(&signer.pubkey(), token_a));
        }
        if target.get_balance(&scratch_b).await? == 0 {
            instructions.extend(create_scratch_account_ix(&signer.pubkey(), token_b));
        }

        let spl_swap_program = get_spl_program(target).await?;
        match program {
            p if p == &spl_swap_program => {
                instructions.push(
                    jet_margin_sdk::ix_builder::test_service::spl_swap_pool_balance(
                        program,
                        token_a,
                        token_b,
                        &scratch_a,
                        &scratch_b,
                        &signer.pubkey(),
                    ),
                );
            }
            p if p == &SABER => {
                instructions.push(
                    jet_margin_sdk::ix_builder::test_service::saber_swap_pool_balance(
                        program,
                        token_a,
                        token_b,
                        &scratch_a,
                        &scratch_b,
                        &signer.pubkey(),
                    ),
                );
            }
            _ => {
                eprintln!("Unknown swap program {program}. Pool not balanced");
                return Ok(());
            }
        }

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

async fn replace_openbook_orders(
    target: &RpcClient,
    signer: &Keypair,
    markets: &HashMap<(Pubkey, Pubkey), OpenBookMarket>,
    program: &Pubkey,
) -> Result<()> {
    for ((token_a, token_b), market) in markets {
        let mut instructions = vec![ComputeBudgetInstruction::set_compute_unit_limit(800_000)];

        let scratch_a = get_scratch_address(&signer.pubkey(), token_a);
        let scratch_b = get_scratch_address(&signer.pubkey(), token_b);

        if target.get_balance(&scratch_a).await? == 0 {
            instructions.extend(create_scratch_account_ix(&signer.pubkey(), token_a));
        }
        if target.get_balance(&scratch_b).await? == 0 {
            instructions.extend(create_scratch_account_ix(&signer.pubkey(), token_b));
        }

        instructions.push(
            jet_margin_sdk::ix_builder::test_service::openbook_market_cancel_orders(
                program,
                token_a,
                token_b,
                &scratch_a,
                &scratch_b,
                &signer.pubkey(),
                &market.bids,
                &market.asks,
                &market.event_queue,
            ),
        );

        let cancel_tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&signer.pubkey()),
            &[signer],
            target.get_latest_blockhash().await?,
        );

        let balance_ix = jet_margin_sdk::ix_builder::test_service::openbook_market_make(
            program,
            token_a,
            token_b,
            &scratch_a,
            &scratch_b,
            &signer.pubkey(),
            &market.bids,
            &market.asks,
            &market.request_queue,
            &market.event_queue,
        );

        let balance_tx = Transaction::new_signed_with_payer(
            &[
                ComputeBudgetInstruction::set_compute_unit_limit(800_000),
                balance_ix,
            ],
            Some(&signer.pubkey()),
            &[signer],
            target.get_latest_blockhash().await?,
        );

        for tx in [cancel_tx, balance_tx] {
            match target.send_and_confirm_transaction(&tx).await {
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
    }

    Ok(())
}

async fn sync_oracles(
    source: &RpcClient,
    target: &RpcClient,
    signer: &Keypair,
    oracles: &[OracleInfo],
) -> Result<()> {
    let oracle_addresses = oracles.iter().map(|o| o.source_oracle).collect::<Vec<_>>();
    let source_accounts = source.get_multiple_accounts(&oracle_addresses).await?;

    let txs = oracles
        .iter()
        .zip(source_accounts)
        .filter_map(|(oracle, account)| {
            account.as_ref()?;
            let account = account.unwrap();
            let source_price = pyth_sdk_solana::state::load_price_account(account.data()).ok()?;

            let update_target_ix =
                jet_margin_sdk::ix_builder::test_service::token_update_pyth_price(
                    &signer.pubkey(),
                    &oracle.target_mint,
                    (source_price.agg.price as f64 * oracle.price_ratio) as i64,
                    source_price.agg.conf as i64,
                    source_price.expo,
                );

            Some(TransactionBuilder::from(vec![update_target_ix]))
        })
        .collect::<Vec<_>>();

    let txs = condense(&txs, &signer.pubkey())?;

    for txb in txs {
        let recent_blockhash = target.get_latest_blockhash().await?;
        let mut tx = txb.to_transaction(&signer.pubkey(), recent_blockhash);
        sign_versioned_transaction(signer, &mut tx);

        if let Err(e) = target.send_transaction(&tx).await {
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

    Ok(())
}

async fn discover_spl_pools(
    target: &Arc<dyn SolanaRpcClient>,
    oracles: &[OracleInfo],
    program: Pubkey,
) -> Result<Vec<(Pubkey, Pubkey)>> {
    let supported_mints = HashSet::from_iter(oracles.iter().map(|o| o.target_mint));
    let result =
        jet_margin_sdk::swap::spl_swap::SplSwapPool::get_pools(target, &supported_mints, program)
            .await?;

    println!("found {} SPL pools", result.len());

    Ok(result.keys().cloned().collect())
}

async fn discover_saber_pools(
    target: &Arc<dyn SolanaRpcClient>,
    oracles: &[OracleInfo],
) -> Result<Vec<(Pubkey, Pubkey)>> {
    let supported_mints = HashSet::from_iter(oracles.iter().map(|o| o.target_mint));
    let result =
        jet_margin_sdk::swap::saber_swap::SaberSwapPool::get_pools(target, &supported_mints)
            .await?;

    println!("found {} Saber pools", result.len());

    Ok(result.keys().cloned().collect())
}

async fn discover_openbook_markets(
    target: &Arc<dyn SolanaRpcClient>,
    oracles: &[OracleInfo],
    program: Pubkey,
) -> Result<HashMap<(Pubkey, Pubkey), OpenBookMarket>> {
    let supported_mints = HashSet::from_iter(oracles.iter().map(|o| o.target_mint));
    let result = OpenBookMarket::get_markets(target, &supported_mints, program).await?;

    println!("found {} Openbook markets", result.len());

    Ok(result)
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

async fn get_spl_program(rpc: &RpcClient) -> Result<Pubkey> {
    let network = get_network_kind_from_rpc(rpc).await?;
    resolve_swap_program(network, "orca-spl-swap").map_err(|e| anyhow::anyhow!("{:?}", e))
}

async fn get_openbook_program(rpc: &RpcClient) -> Result<Pubkey> {
    let network = get_network_kind_from_rpc(rpc).await?;
    resolve_swap_program(network, "openbook").map_err(|e| anyhow::anyhow!("{:?}", e))
}

async fn get_pyth_program_id(rpc: &RpcClient) -> Result<Pubkey> {
    let network_kind = get_network_kind_from_rpc(rpc).await?;

    Ok(match network_kind {
        NetworkKind::Mainnet => PYTH_MAINNET_PROGRAM,
        NetworkKind::Devnet => PYTH_DEVNET_PROGRAM,
        NetworkKind::Localnet => panic!("no pyth program supported on localnet"),
    })
}

async fn get_network_kind_from_rpc(rpc: &RpcClient) -> Result<NetworkKind> {
    let network_hash = rpc.get_genesis_hash().await?;
    Ok(NetworkKind::from_genesis_hash(&network_hash))
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
        if std::fs::write(Self::PATH, pid.to_string()).is_err() {
            eprintln!("Unable to create oracle mirror PID file");
        };

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
