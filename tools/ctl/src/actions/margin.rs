use std::{
    fmt::Display,
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anchor_lang::AccountDeserialize;
use anyhow::{bail, Result};
use chrono::{DateTime, Local};
use clap::Parser;
use comfy_table::{presets::UTF8_FULL, Table};
use futures::FutureExt;
use jet_margin_sdk::{
    ix_builder::{
        derive_margin_permit, derive_token_config, get_metadata_address, MarginConfigIxBuilder,
        MarginIxBuilder, MarginPoolIxBuilder,
    },
    jet_airspace::state::Airspace,
    jet_margin::{self, MarginAccount, PriceInfo, Valuation},
    jet_margin_pool::{self, MarginPool},
    jet_metadata::{self},
};
use jet_program_common::DEFAULT_AIRSPACE;
use jet_solana_client::rpc::SolanaRpcExtra;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use solana_sdk::{pubkey, pubkey::Pubkey};

use crate::{
    client::{Client, Plan},
    governance::resolve_payer,
};

static TOKEN_BLACKLIST: &[Pubkey] = &[
    pubkey!("EDg8oL2FoK7pmCvR8iRGmbW2HioUaDNWRihkQA5rEqDj"),
    pubkey!("J9n7rmgGgaiysVGnspchnCk4riWC9qDeh9wp8rc2PzGG"),
];

#[serde_as]
#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct TokenConfig {
    /// The mint address for the token being configured
    #[serde_as(as = "DisplayFromStr")]
    pub mint: Pubkey,

    #[clap(long)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub pyth_price: Option<Pubkey>,

    #[clap(long)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub pyth_product: Option<Pubkey>,

    #[clap(long)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub token_kind: Option<TokenKind>,

    #[clap(long)]
    pub collateral_weight: Option<u16>,

    #[clap(long)]
    pub max_leverage: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum TokenKind {
    NonCollateral,
    Collateral,
    Claim,
}

impl From<TokenKind> for jet_metadata::TokenKind {
    fn from(val: TokenKind) -> Self {
        match val {
            TokenKind::NonCollateral => jet_metadata::TokenKind::NonCollateral,
            TokenKind::Collateral => jet_metadata::TokenKind::Collateral,
            TokenKind::Claim => jet_metadata::TokenKind::Claim,
        }
    }
}

impl FromStr for TokenKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NonCollateral" => Ok(Self::NonCollateral),
            "Collateral" => Ok(Self::Collateral),
            "Claim" => bail!("cannot set pool token as claim"),
            s => bail!("invalid token type {s}"),
        }
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::NonCollateral => write!(f, "NonCollateral"),
            TokenKind::Collateral => write!(f, "Collateral"),
            TokenKind::Claim => write!(f, "Claim"),
        }
    }
}

impl PartialEq<jet_metadata::TokenKind> for TokenKind {
    fn eq(&self, other: &jet_metadata::TokenKind) -> bool {
        let converted: jet_metadata::TokenKind = (*self).into();
        converted == *other
    }
}

pub async fn process_register_adapter(
    client: &Client,
    airspace: Pubkey,
    adapter: Pubkey,
) -> Result<Plan> {
    let adapter_md_address = get_metadata_address(&adapter);
    let ix = MarginConfigIxBuilder::new(airspace, resolve_payer(client)?, None);

    if client.account_exists(&adapter_md_address).await? {
        println!("address {adapter} is already registered");
        return Ok(Plan::default());
    }

    Ok(client
        .plan()?
        .instructions(
            [],
            [format!("set-adapter {adapter}")],
            [ix.configure_adapter(adapter, true)],
        )
        .build())
}

pub async fn process_refresh_metadata(client: &Client, airspace: Pubkey) -> Result<Plan> {
    let mut plan = client.plan()?;
    let rpc = client.network_interface();
    let margin_accounts = get_all_accounts(client, &airspace)
        .await?
        .into_iter()
        .filter(|(_, account)| account.airspace == airspace)
        .collect::<Vec<_>>();
    let configs = rpc
        .find_anchor_accounts::<jet_margin::TokenConfig>()
        .await?
        .into_iter()
        .filter(|(_, config)| config.airspace == airspace)
        .collect::<Vec<_>>();

    let mut position_count = 0;
    let mut fix_count = 0;

    println!("found {} configs", configs.len());
    println!("found {} margin accounts", margin_accounts.len());

    for (address, account) in margin_accounts {
        let ix = MarginIxBuilder::new(
            account.airspace,
            account.owner,
            u16::from_le_bytes(account.user_seed),
        )
        .with_authority(client.signer()?);

        for (_, config) in &configs {
            if let Some(position) = account.get_position(&config.mint) {
                position_count += 1;

                if position.kind() != config.token_kind
                    || position.value_modifier != config.value_modifier
                    || position.max_staleness != config.max_staleness
                {
                    fix_count += 1;
                    plan = plan.instructions(
                        [],
                        [format!(
                            "refresh-position-md {} for account {address}",
                            config.mint
                        )],
                        [ix.refresh_position_config(&config.mint)],
                    );
                }
            }
        }
    }

    println!("positions to fix: {fix_count} / {position_count}");

    Ok(plan.build())
}

pub async fn process_set_refresher_permission(
    client: &Client,
    airspace: Pubkey,
    account: Pubkey,
    allow_refresh: bool,
) -> Result<Plan> {
    let permit_address = derive_margin_permit(&airspace, &account);
    let airspace_authority = client
        .read_anchor_account::<Airspace>(&airspace)
        .await?
        .authority;
    let ix = MarginConfigIxBuilder::new(airspace, resolve_payer(client)?, Some(airspace_authority));

    if let Ok(permit) = client
        .read_anchor_account::<jet_margin::Permit>(&permit_address)
        .await
    {
        if permit
            .permissions
            .contains(jet_margin::Permissions::REFRESH_POSITION_CONFIG)
        {
            println!("address {account} already has refresh permission");
            return Ok(Plan::default());
        }
    }

    Ok(client
        .plan()?
        .instructions(
            [],
            [format!("set-refresher {account} = {allow_refresh}")],
            [ix.configure_position_config_refresher(account, allow_refresh)],
        )
        .build())
}

pub async fn process_set_liquidator(
    client: &Client,
    airspace: Pubkey,
    liquidator: Pubkey,
    is_liquidator: bool,
) -> Result<Plan> {
    let liquidator_md_address = get_metadata_address(&liquidator);
    let airspace_authority = client
        .read_anchor_account::<Airspace>(&airspace)
        .await?
        .authority;
    let ix = MarginConfigIxBuilder::new(airspace, resolve_payer(client)?, Some(airspace_authority));

    let is_currently_liquidator = client.account_exists(&liquidator_md_address).await?;
    if is_currently_liquidator == is_liquidator {
        let word = if is_liquidator { "already" } else { "not" };
        println!("address {liquidator} is {word} a liquidator");
        return Ok(Plan::default());
    }

    Ok(client
        .plan()?
        .instructions(
            [],
            [format!("set-liquidator {liquidator} = {is_liquidator}")],
            [ix.configure_liquidator(liquidator, is_liquidator)],
        )
        .build())
}

pub async fn process_update_balances(
    client: &Client,
    margin_account_address: Pubkey,
) -> Result<Plan> {
    let account = client
        .read_anchor_account::<MarginAccount>(&margin_account_address)
        .await?;

    let ix = MarginIxBuilder::new(
        account.airspace,
        account.owner,
        u16::from_le_bytes(account.user_seed),
    )
    .with_authority(client.signer()?);
    let mut steps = vec![];
    let mut instructions = vec![];

    for position in account.positions() {
        let current_state = client.read_token_account(&position.address).await?;

        if position.balance != current_state.amount {
            steps.push(format!("update-balance {}", position.address));
            instructions.push(ix.update_position_balance(position.address));
        }
    }

    Ok(client.plan()?.instructions([], steps, instructions).build())
}

pub async fn process_transfer_position(
    client: &Client,
    source_account: Pubkey,
    target_account: Pubkey,
    token: Pubkey,
    amount: Option<u64>,
) -> Result<Plan> {
    let source = client
        .read_anchor_account::<MarginAccount>(&source_account)
        .await?;

    let ix = MarginIxBuilder::new(
        source.airspace,
        source.owner,
        u16::from_le_bytes(source.user_seed),
    )
    .with_authority(resolve_payer(client)?);
    let pool_ix = MarginPoolIxBuilder::new(token);
    let position_token_mint = pool_ix.deposit_note_mint;
    let amount = match amount {
        Some(n) => n,
        None => {
            client
                .read_token_account(&ix.get_token_account_address(&position_token_mint))
                .await?
                .amount
        }
    };

    Ok(client
        .plan()?
        .instructions(
            [],
            [format!(
                "admin-transfer-position {source_account} -> {target_account}: {amount} {token}"
            )],
            [ix.admin_transfer_position_to(&target_account, &position_token_mint, amount)],
        )
        .build())
}

pub async fn process_list_top_accounts(
    client: &Client,
    airspace: Pubkey,
    limit: usize,
) -> Result<Plan> {
    let mut all_user_accounts = get_all_accounts(client, &airspace).await?;

    let refresh_account_tasks = all_user_accounts.iter_mut().map(|(address, account)| {
        refresh_account_positions(client, account).map(|result| (*address, result))
    });

    let results = futures::future::join_all(refresh_account_tasks).await;
    let mut accounts = vec![];

    let mut blacklist_count = 0;
    for (address, result) in results {
        if let Err(e) = &result {
            println!("failed refreshing margin account {address}: {e:?}");
            continue;
        }

        if !result.unwrap() {
            blacklist_count += 1;
            continue;
        }

        let account = all_user_accounts
            .iter()
            .find(|(addr, _)| *addr == address)
            .unwrap()
            .1;

        accounts.push(MarginAccountSummary {
            address,
            position_count: account.positions().count(),
            valuation: account
                .valuation(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                )
                .unwrap(),
        });
    }

    println!("skipped {blacklist_count} blacklisted accounts");
    println!("found {} margin accounts", accounts.len());

    println!("Top {limit} accounts by risk:");
    show_top_accounts_by(limit, &mut accounts, compare_account_risk);
    println!("Top {limit} accounts by equity:");
    show_top_accounts_by(limit, &mut accounts, compare_account_equity);
    println!("Top {limit} accounts by collateral:");
    show_top_accounts_by(limit, &mut accounts, compare_account_weighted_collateral);
    println!("Top {limit} accounts by required collateral:");
    show_top_accounts_by(limit, &mut accounts, compare_account_required_collateral);

    Ok(Plan::default())
}

async fn get_all_accounts(
    client: &Client,
    airspace: &Pubkey,
) -> Result<Vec<(Pubkey, MarginAccount)>> {
    let all_margin_accounts = client.rpc().get_program_accounts(&jet_margin::ID).await?;
    let margin_user_account_size = 8 + std::mem::size_of::<MarginAccount>();

    Ok(all_margin_accounts
        .into_iter()
        .filter_map(|(address, account)| {
            if account.data.len() != margin_user_account_size {
                return None;
            }

            match MarginAccount::try_deserialize(&mut &account.data[..]) {
                Ok(deserialized) => {
                    (deserialized.airspace == *airspace).then_some((address, deserialized))
                }
                Err(_) => {
                    eprintln!("could not deserialize margin account {address}");
                    None
                }
            }
        })
        .collect())
}

pub async fn process_inspect(client: &Client, addresses: Vec<Pubkey>) -> Result<Plan> {
    for address in addresses {
        let account = client
            .read_anchor_account::<MarginAccount>(&address)
            .await?;
        println!("{address:#?}");
        println!("{account:#?}");
        if let Some(oldest_price) = account.positions().map(|p| p.price.timestamp).min() {
            print!("{:#?}", account.valuation(oldest_price)?);
            let dt: DateTime<Local> = (UNIX_EPOCH + Duration::from_secs(oldest_price)).into();
            println!("   priced_at: {}", dt.to_rfc2822());
        }
        println!();
    }

    Ok(Plan::default())
}

pub async fn process_read_token_config(
    client: &Client,
    airspace: Pubkey,
    address: Pubkey,
) -> Result<Plan> {
    use jet_margin_sdk::jet_margin::TokenConfig;

    let try_config = client
        .read_anchor_account::<TokenConfig>(&address)
        .await
        .ok();

    let try_derive_config = client
        .read_anchor_account::<TokenConfig>(&derive_token_config(&airspace, &address))
        .await
        .ok();

    let config = match (try_config, try_derive_config) {
        (Some(config), _) => config,
        (_, Some(config)) => config,
        _ => {
            println!("no token config found for {address}");
            return Ok(Plan::default());
        }
    };

    println!("{config:#?}");
    Ok(Plan::default())
}

pub async fn process_configure_account_airspaces(client: &Client) -> Result<Plan> {
    let mut plan = client.plan()?.unordered();
    let accounts = get_all_accounts(client, &Pubkey::default()).await?;

    for (address, _) in accounts {
        let builder =
            MarginIxBuilder::new_for_address(DEFAULT_AIRSPACE, address, resolve_payer(client)?);
        let ix = builder.configure_account_airspace();

        plan = plan.instructions([], [format!("configure-account-airspace {address}")], [ix])
    }

    Ok(plan.build())
}

async fn refresh_account_positions(client: &Client, account: &mut MarginAccount) -> Result<bool> {
    let position_mints = account.positions().map(|p| p.token).collect::<Vec<_>>();

    for mint in position_mints {
        let position = account.get_position_mut(&mint).unwrap();
        let price_info = match position.adapter {
            adapter if adapter == jet_margin_pool::ID => {
                let margin_pool_address = client.read_mint(&mint).await?.mint_authority.unwrap();
                let margin_pool = client
                    .read_anchor_account::<MarginPool>(&margin_pool_address)
                    .await?;

                if TOKEN_BLACKLIST.contains(&margin_pool.token_mint) {
                    return Ok(false);
                }

                let mut oracle_data = client
                    .rpc()
                    .get_account(&margin_pool.token_price_oracle)
                    .await?;
                let price_oracle = pyth_sdk_solana::load_price_feed_from_account(
                    &margin_pool.token_price_oracle,
                    &mut oracle_data,
                )?;

                let prices = margin_pool.calculate_prices(&price_oracle)?;

                let price_value = if mint == margin_pool.deposit_note_mint {
                    prices.deposit_note_price
                } else {
                    prices.loan_note_price
                };

                PriceInfo::new_valid(
                    // SAFETY: We only need the exponent, which won't change if the price is stale
                    price_oracle.get_ema_price_unchecked().expo,
                    price_value,
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                )
            }

            adapter => {
                bail!("unknown position adapter {adapter}");
            }
        };

        position.set_price(&price_info).unwrap();
    }

    Ok(true)
}

#[derive(Debug, Clone)]
struct MarginAccountSummary {
    address: Pubkey,
    position_count: usize,
    valuation: Valuation,
}

impl MarginAccountSummary {
    fn risk(&self) -> f64 {
        if self.valuation.effective_collateral.as_u64(0) == 0 {
            return 0.0;
        }

        (self.valuation.required_collateral / self.valuation.effective_collateral)
            .to_string()
            .parse::<f64>()
            .unwrap()
    }
}

impl From<MarginAccountSummary> for comfy_table::Cells {
    fn from(summary: MarginAccountSummary) -> Self {
        Self(vec![
            summary.address.into(),
            summary.position_count.into(),
            summary.valuation.equity.into(),
            summary.valuation.weighted_collateral.into(),
            summary.valuation.required_collateral.into(),
            summary.valuation.effective_collateral.into(),
            summary.risk().into(),
        ])
    }
}

fn show_top_accounts_by(
    limit: usize,
    list: &mut [MarginAccountSummary],
    comparator: impl Fn(&MarginAccountSummary, &MarginAccountSummary) -> std::cmp::Ordering,
) {
    list.sort_by(comparator);

    let rows = list.iter().rev().take(limit).cloned().collect::<Vec<_>>();
    let mut output_table = Table::new();

    output_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
        .set_header(vec![
            "Account",
            "Position Count",
            "Equity",
            "Weighted Collateral",
            "Required Collateral",
            "Effective Collateral",
            "Risk",
        ]);

    for row in rows {
        output_table.add_row(row);
    }

    println!("{output_table}");
}

fn compare_account_equity(
    a: &MarginAccountSummary,
    b: &MarginAccountSummary,
) -> std::cmp::Ordering {
    a.valuation.equity.cmp(&b.valuation.equity)
}

fn compare_account_weighted_collateral(
    a: &MarginAccountSummary,
    b: &MarginAccountSummary,
) -> std::cmp::Ordering {
    a.valuation
        .weighted_collateral
        .cmp(&b.valuation.weighted_collateral)
}

fn compare_account_required_collateral(
    a: &MarginAccountSummary,
    b: &MarginAccountSummary,
) -> std::cmp::Ordering {
    a.valuation
        .required_collateral
        .cmp(&b.valuation.required_collateral)
}

fn compare_account_risk(a: &MarginAccountSummary, b: &MarginAccountSummary) -> std::cmp::Ordering {
    a.risk().partial_cmp(&b.risk()).unwrap()
}
