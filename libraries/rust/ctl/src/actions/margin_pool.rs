use anchor_lang::AccountDeserialize;
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use comfy_table::{presets::UTF8_FULL, Table};
use jet_margin_sdk::{
    ix_builder::{
        get_metadata_address, ControlIxBuilder, MarginPoolConfiguration, MarginPoolIxBuilder,
    },
    jet_control::TokenMetadataParams,
    jet_margin_pool::{self, MarginPool},
    jet_metadata::{PositionTokenMetadata, TokenMetadata},
};
use serde::{Deserialize, Serialize};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

use super::margin::TokenConfig;
use crate::{
    client::{Client, Plan},
    governance::resolve_payer,
};

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct MarginPoolParameters {
    #[clap(long)]
    pub flags: Option<u64>,

    #[clap(long)]
    pub utilization_rate_1: Option<u16>,

    #[clap(long)]
    pub utilization_rate_2: Option<u16>,

    #[clap(long)]
    pub borrow_rate_0: Option<u16>,

    #[clap(long)]
    pub borrow_rate_1: Option<u16>,

    #[clap(long)]
    pub borrow_rate_2: Option<u16>,

    #[clap(long)]
    pub borrow_rate_3: Option<u16>,

    #[clap(long)]
    pub management_fee_rate: Option<u16>,
}

#[derive(Debug, Parser, Deserialize)]
pub struct ConfigurePoolCliOptions {
    #[clap(flatten)]
    #[serde(flatten)]
    pub token_config: TokenConfig,

    #[clap(flatten)]
    #[serde(flatten)]
    pub margin_pool: MarginPoolParameters,
}

pub async fn process_list_pools(client: &Client) -> Result<Plan> {
    let pools = find_all_margin_pools(client).await?;

    println!("Found {} pools", pools.len());

    let mut output_table = Table::new();

    output_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
        .set_header(vec![
            "Token",
            "Pool Address",
            "Deposits",
            "Loans",
            "Interest Rate",
            "Fees",
        ]);

    let summary_tasks = pools
        .into_iter()
        .map(|(address, pool)| collect_pool_summary(client, address, pool));

    for summary_result in futures::future::join_all(summary_tasks).await {
        match summary_result {
            Ok(summary) => {
                output_table.add_row(summary);
            }
            Err(e) => eprintln!("failed to generate summary: {e:?}"),
        }
    }

    println!("{output_table}");

    Ok(Plan::default())
}

pub async fn process_collect_pool_fees(client: &Client) -> Result<Plan> {
    let pools = find_all_margin_pools(client).await?;

    let instructions = pools
        .into_iter()
        .filter_map(|(address, pool)| {
            let ix_build = MarginPoolIxBuilder::new(pool.token_mint);
            let (fee_vault_address, _) = Pubkey::find_program_address(
                &[
                    jet_margin_sdk::jet_control::seeds::FEE_DESTINATION,
                    address.as_ref(),
                ],
                &jet_margin_sdk::jet_control::ID,
            );

            if pool.fee_destination != fee_vault_address {
                return None;
            }

            Some((
                format!("collect margin-pool fees for token {}", pool.token_mint),
                ix_build.collect(fee_vault_address),
            ))
        })
        .collect::<Vec<_>>();

    Ok(instructions
        .chunks(6)
        .fold(client.plan()?, |plan, chunk| {
            let (steps, ix_list): (Vec<String>, Vec<Instruction>) =
                chunk.into_iter().cloned().unzip();

            plan.instructions([], steps, ix_list)
        })
        .build())
}

pub async fn process_create_pool(client: &Client, token: Pubkey) -> Result<Plan> {
    let margin_pool = MarginPoolIxBuilder::new(token);
    let ctrl = ControlIxBuilder::new(resolve_payer(client)?);

    if client.account_exists(&margin_pool.address).await? {
        println!("the pool already exists for token {token}");
        return Ok(Plan::default());
    }

    if !client.account_exists(&token).await? {
        println!("the token {token} does not exist");
        return Ok(Plan::default());
    }

    Ok(client
        .plan()?
        .instructions(
            [],
            [format!("create-margin-pool for token {token}")],
            [ctrl.create_margin_pool(&token)],
        )
        .build())
}

pub async fn process_configure_pool(
    client: &Client,
    options: ConfigurePoolCliOptions,
) -> Result<Plan> {
    let margin_pool = MarginPoolIxBuilder::new(options.token_config.mint);
    let ctrl = ControlIxBuilder::new(resolve_payer(client)?);
    let mut configuration = match client.account_exists(&margin_pool.address).await? {
        false => MarginPoolConfiguration {
            parameters: Some(Default::default()),
            metadata: Some(Default::default()),
            ..Default::default()
        },
        true => download_margin_pool_config(client, &margin_pool).await?,
    };

    println!("changes to pool for token {}:", options.token_config.mint);

    if options.token_config.pyth_price.is_some()
        && options.token_config.pyth_price != configuration.pyth_price
    {
        configuration.pyth_price = options.token_config.pyth_price;
        configuration.pyth_product = options.token_config.pyth_product;

        println!(
            "set oracle: price={}, product={}",
            configuration.pyth_price.unwrap(),
            configuration.pyth_product.unwrap_or_default()
        );
    } else {
        configuration.pyth_price = None;
        configuration.pyth_product = None;
    }

    override_pool_config_with_options(&mut configuration, &options);
    println!();

    Ok(client
        .plan()?
        .instructions(
            [],
            [format!(
                "configure-margin-pool for token {}",
                options.token_config.mint
            )],
            [ctrl.configure_margin_pool(&options.token_config.mint, &configuration)],
        )
        .build())
}

macro_rules! override_field {
    ($obj:ident, $field:ident) => {
        if let Some(value) = $field {
            if *value != $obj.$field {
                println!(
                    "set {}: {:?} -> {:?}",
                    stringify!($field),
                    &$obj.$field,
                    &value
                );
                $obj.$field = (*value).into();
            }
        }
    };
}

fn override_pool_config_with_options(
    config: &mut MarginPoolConfiguration,
    options: &ConfigurePoolCliOptions,
) {
    let ConfigurePoolCliOptions {
        margin_pool,
        token_config,
        ..
    } = options;
    let TokenConfig {
        token_kind,
        collateral_weight,
        max_leverage,
        ..
    } = token_config;
    let MarginPoolParameters {
        flags,
        utilization_rate_1,
        utilization_rate_2,
        borrow_rate_0,
        borrow_rate_1,
        borrow_rate_2,
        borrow_rate_3,
        management_fee_rate,
    } = margin_pool;

    let orig_params = config.parameters.clone().unwrap();
    let params = config.parameters.as_mut().unwrap();

    override_field!(params, flags);
    override_field!(params, utilization_rate_1);
    override_field!(params, utilization_rate_2);
    override_field!(params, borrow_rate_0);
    override_field!(params, borrow_rate_1);
    override_field!(params, borrow_rate_2);
    override_field!(params, borrow_rate_3);
    override_field!(params, management_fee_rate);

    if orig_params == *params {
        config.parameters = None;
    }

    if token_kind.is_some() || collateral_weight.is_some() || max_leverage.is_some() {
        let metadata = config.metadata.as_mut().unwrap();

        override_field!(metadata, token_kind);
        override_field!(metadata, collateral_weight);
        override_field!(metadata, max_leverage);
    } else {
        config.metadata = None;
    }
}

async fn find_all_margin_pools(client: &Client) -> Result<Vec<(Pubkey, MarginPool)>> {
    let pool_accounts = client
        .rpc
        .get_program_accounts(&jet_margin_pool::ID, None)
        .await?;

    Ok(pool_accounts
        .into_iter()
        .filter_map(|(address, account)| {
            let pool = match MarginPool::try_deserialize(&mut &account.data[..]) {
                Ok(pool) => pool,
                Err(_) => {
                    return None;
                }
            };

            Some((address, pool))
        })
        .collect())
}

async fn download_margin_pool_config(
    client: &Client,
    margin_pool: &MarginPoolIxBuilder,
) -> Result<MarginPoolConfiguration> {
    let margin_pool_data = client
        .read_anchor_account::<MarginPool>(&margin_pool.address)
        .await?;
    let deposit_note_md = client
        .read_anchor_account::<PositionTokenMetadata>(&get_metadata_address(
            &margin_pool.deposit_note_mint,
        ))
        .await?;
    let loan_note_md = client
        .read_anchor_account::<PositionTokenMetadata>(&get_metadata_address(
            &margin_pool.loan_note_mint,
        ))
        .await?;

    Ok(MarginPoolConfiguration {
        parameters: Some(margin_pool_data.config.clone()),
        pyth_price: Some(margin_pool_data.token_price_oracle),
        pyth_product: None,
        metadata: Some(TokenMetadataParams {
            token_kind: deposit_note_md.token_kind,
            collateral_weight: deposit_note_md.value_modifier,
            max_leverage: loan_note_md.value_modifier,
        }),
    })
}

#[derive(Debug)]
struct PoolSummary {
    token: String,
    address: Pubkey,
    deposits: f64,
    vault_balance: f64,
    fee_vault_balance: f64,
    loans: f64,
    rate: f64,
}

impl From<PoolSummary> for comfy_table::Cells {
    fn from(entry: PoolSummary) -> Self {
        let deposits = match entry.deposits {
            amount if amount != entry.vault_balance => {
                comfy_table::Cell::from(format!("{:.4}", entry.deposits))
                    .fg(comfy_table::Color::Yellow)
            }
            _ => entry.deposits.into(),
        };

        let rate = format!("{:.2}%", entry.rate * 100.0);

        Self(vec![
            entry.token.into(),
            entry.address.into(),
            deposits,
            entry.loans.into(),
            rate.into(),
            entry.fee_vault_balance.into(),
        ])
    }
}

async fn collect_pool_summary(
    client: &Client,
    address: Pubkey,
    pool: MarginPool,
) -> Result<PoolSummary> {
    let token = match get_token_symbol(client, &pool).await {
        Ok(symbol) => symbol,
        Err(e) => {
            eprintln!("{e}");
            pool.token_mint.to_string()
        }
    };
    let token_mint = client.read_mint(&pool.token_mint).await?;
    let (fee_vault_address, _) = Pubkey::find_program_address(
        &[
            jet_margin_sdk::jet_control::seeds::FEE_DESTINATION,
            pool.address.as_ref(),
        ],
        &jet_margin_sdk::jet_control::ID,
    );

    let deposits = spl_token::amount_to_ui_amount(pool.deposit_tokens, token_mint.decimals);
    let vault_balance = client
        .rpc
        .get_token_account_balance(&pool.vault)
        .await?
        .ui_amount
        .unwrap();
    let fee_vault_balance = client
        .rpc
        .get_token_account_balance(&fee_vault_address)
        .await?
        .ui_amount
        .unwrap();

    let loans = spl_token::amount_to_ui_amount(
        jet_proto_math::Number::from_bits(pool.borrowed_tokens).as_u64(0),
        token_mint.decimals,
    );
    let rate = pool.interest_rate().to_string().parse::<f64>()?;

    Ok(PoolSummary {
        token,
        address,
        deposits,
        vault_balance,
        fee_vault_balance,
        loans,
        rate,
    })
}

async fn get_token_symbol(client: &Client, pool: &MarginPool) -> Result<String> {
    let token_md_address = get_metadata_address(&pool.token_mint);
    let token_md_account = client
        .rpc
        .get_account_data(&token_md_address)
        .await
        .with_context(|| format!("getting metadata for token {}", &pool.token_mint))?;

    let token_md = TokenMetadata::try_deserialize(&mut &token_md_account[..])?;
    let product_data = client
        .rpc
        .get_account_data(&token_md.pyth_product)
        .await
        .with_context(|| {
            format!(
                "getting pyth product {} for token {}",
                token_md.pyth_product, pool.token_mint
            )
        })?;

    let pyth_product = pyth_sdk_solana::state::load_product_account(&product_data)
        .with_context(|| format!("failed loading pyth product for token {}", pool.token_mint))?;

    pyth_product
        .iter()
        .find_map(|(k, v)| match k {
            "base" => Some(v.to_string()),
            _ => None,
        })
        .ok_or_else(|| {
            anyhow!(
                "pyth product {} for token {} has no base attribute",
                &token_md.pyth_product,
                &pool.token_mint
            )
        })
}
