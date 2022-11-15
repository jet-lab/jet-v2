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
    ix_builder::{get_metadata_address, ControlIxBuilder, MarginIxBuilder, MarginPoolIxBuilder},
    jet_margin::{self, syscall::thread_local_mock, MarginAccount, PriceInfo, Valuation},
    jet_margin_pool::{self, MarginPool},
    jet_metadata::{self, PositionTokenMetadata},
};
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

pub async fn process_register_adapter(client: &Client, adapter: Pubkey) -> Result<Plan> {
    let adapter_md_address = get_metadata_address(&adapter);
    let ix = ControlIxBuilder::new(resolve_payer(client)?);

    if client.account_exists(&adapter_md_address).await? {
        println!("address {adapter} is already registered");
        return Ok(Plan::new());
    }

    Ok(client
        .plan()?
        .instructions(
            [],
            [format!("set-adapter {adapter}")],
            [ix.register_adapter(&adapter)],
        )
        .build())
}

pub async fn process_refresh_metadata(client: &Client, token: Pubkey) -> Result<Plan> {
    let margin_accounts = get_all_accounts(client).await?;
    let mut plan = client.plan()?;
    let deposit_token = MarginPoolIxBuilder::new(token).deposit_note_mint;
    let config = client
        .read_anchor_account::<PositionTokenMetadata>(&get_metadata_address(&deposit_token))
        .await?;

    let mut position_count = 0;
    let mut fix_count = 0;

    println!("current config: {config:#?}");
    println!("found {} margin accounts", margin_accounts.len());

    for (address, mut account) in margin_accounts {
        let ix = MarginIxBuilder::new_with_payer(
            account.owner,
            u16::from_le_bytes(account.user_seed),
            client.signer()?,
            None,
        );

        if let Some(position) = account.get_position(&deposit_token) {
            position_count += 1;

            if position.kind() != config.token_kind.into()
                || position.value_modifier != config.value_modifier
                || position.max_staleness != config.max_staleness
            {
                fix_count += 1;
                plan = plan.instructions(
                    [],
                    [format!("refresh-position-md {token} for {address}")],
                    [ix.refresh_position_metadata(&deposit_token)],
                );
            }
        }
    }

    println!("accounts to fix: {fix_count} / {position_count}");

    Ok(plan.build())
}

pub async fn process_set_liquidator(
    client: &Client,
    liquidator: Pubkey,
    is_liquidator: bool,
) -> Result<Plan> {
    let liquidator_md_address = get_metadata_address(&liquidator);
    let ix = ControlIxBuilder::new(resolve_payer(client)?);

    let is_currently_liquidator = client.account_exists(&liquidator_md_address).await?;
    if is_currently_liquidator == is_liquidator {
        let word = if is_liquidator { "already" } else { "not" };
        println!("address {liquidator} is {word} a liquidator");
        return Ok(Plan::new());
    }

    Ok(client
        .plan()?
        .instructions(
            [],
            [format!("set-liquidator {liquidator} = {is_liquidator}")],
            [ix.set_liquidator(&liquidator, is_liquidator)],
        )
        .build())
}

pub async fn process_list_top_accounts(client: &Client, limit: usize) -> Result<Plan> {
    let mut all_user_accounts = get_all_accounts(client).await?;

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
            valuation: account.valuation().unwrap(),
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

    Ok(Plan::new())
}

async fn get_all_accounts(client: &Client) -> Result<Vec<(Pubkey, MarginAccount)>> {
    let all_margin_accounts = client.rpc().get_program_accounts(&jet_margin::ID).await?;
    let margin_user_account_size = 8 + std::mem::size_of::<MarginAccount>();

    Ok(all_margin_accounts
        .into_iter()
        .filter_map(|(address, account)| {
            if account.data.len() != margin_user_account_size {
                return None;
            }

            match MarginAccount::try_deserialize(&mut &account.data[..]) {
                Ok(deserialized) => Some((address, deserialized)),
                Err(_) => {
                    eprintln!("could not deserialize margin account {address}");
                    None
                }
            }
        })
        .collect())
}

async fn get_all_accounts(client: &Client) -> Result<Vec<(Pubkey, MarginAccount)>> {
    let all_margin_accounts = client.rpc().get_program_accounts(&jet_margin::ID).await?;
    let margin_user_account_size = 8 + std::mem::size_of::<MarginAccount>();

    Ok(all_margin_accounts
        .into_iter()
        .filter_map(|(address, account)| {
            if account.data.len() != margin_user_account_size {
                return None;
            }

            match MarginAccount::try_deserialize(&mut &account.data[..]) {
                Ok(deserialized) => Some((address, deserialized)),
                Err(_) => {
                    eprintln!("could not deserialize margin account {address}");
                    None
                }
            }
        })
        .collect())
}

pub async fn process_inspect(client: &Client, addresses: Vec<Pubkey>) -> Result<Plan> {
    thread_local_mock::mock_clock(Some(0));
    for address in addresses {
        let account = client
            .read_anchor_account::<MarginAccount>(&address)
            .await?;
        println!("{address:#?}");
        println!("{account:#?}");
        if let Some(oldest_price) = account.positions().map(|p| p.price.timestamp).min() {
            thread_local_mock::mock_clock(Some(oldest_price));
            print!("{:#?}", account.valuation()?);
            let dt: DateTime<Local> = (UNIX_EPOCH + Duration::from_secs(oldest_price)).into();
            println!("   priced_at: {}", dt.to_rfc2822());
        }
        println!();
    }

    Ok(Plan::default())
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
                    price_oracle.expo,
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
