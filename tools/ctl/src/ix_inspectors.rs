use std::collections::HashMap;

use anchor_lang::{prelude::Pubkey, AccountDeserialize};
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::try_join;
use jet_margin_sdk::{
    jet_control,
    jet_margin_pool::MarginPool,
    jet_metadata::{PositionTokenMetadata, TokenMetadata},
};

use crate::{anchor_ix_parser::ParsedInstruction, client::Client};

pub fn all_inspectors() -> Vec<IxInspector> {
    vec![ConfigureMarginPoolInspector.wrap()]
}

pub async fn inspect(client: &Client, ix: &ParsedInstruction, inspectors: Vec<IxInspector>) {
    let mut ran_one = false;
    for inspector in inspectors {
        if inspector.matches(ix) {
            ran_one = true;
            if let Err(e) = inspector.run(client, ix).await {
                eprintln!("failed to run {}: {e:#?}", inspector.name());
            }
        }
    }
    if ran_one {
        println!("\n=====================================\n");
    }
}

/// this wrapper is needed due to https://github.com/rust-lang/rust/issues/63033
pub struct IxInspector(Box<dyn CustomIxInspector>);
impl std::ops::Deref for IxInspector {
    type Target = dyn CustomIxInspector;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

/// Print some information about what the instruction will do.
///
/// By default, inspectors are applicable to ALL instructions. To constrain the
/// instructions that an inspector will apply to, you can filter it by
/// program_id and instruction_name, or you can implement the `matches` method
/// for full customization.
#[async_trait(?Send)]
pub trait CustomIxInspector {
    fn name(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }

    fn program_id(&self) -> Option<Pubkey> {
        None
    }

    fn instruction_name(&self) -> Option<String> {
        None
    }

    fn matches(&self, ix: &ParsedInstruction) -> bool {
        self.program_id().map(|id| id == ix.program).unwrap_or(true)
            && self
                .instruction_name()
                .map(|id| id == ix.name)
                .unwrap_or(true)
    }

    async fn run(&self, client: &Client, ix: &ParsedInstruction) -> Result<()>;

    fn wrap(self) -> IxInspector
    where
        Self: Sized + 'static,
    {
        IxInspector(Box::new(self))
    }
}

/// Prints:
/// - all current data in the accounts that will be mutated by this instruction
/// - in a separate list, the specific fields that will be changed by this
///   instruction
pub struct ConfigureMarginPoolInspector;
#[async_trait(?Send)]
impl CustomIxInspector for ConfigureMarginPoolInspector {
    fn program_id(&self) -> Option<Pubkey> {
        Some(jet_control::ID)
    }
    fn instruction_name(&self) -> Option<String> {
        Some(String::from("configureMarginPool"))
    }
    async fn run(&self, client: &Client, ix: &ParsedInstruction) -> Result<()> {
        let accounts = ix.account_map();
        let (pool, token_metadata, deposit_metadata, loan_metadata) = try_join!(
            account_by_name::<MarginPool>(client, &accounts, "marginPool"),
            account_by_name::<TokenMetadata>(client, &accounts, "tokenMetadata"),
            account_by_name::<PositionTokenMetadata>(client, &accounts, "depositMetadata"),
            account_by_name::<PositionTokenMetadata>(client, &accounts, "loanMetadata"),
        )?;
        println!("\nExisting state that this instruction affects:\n");
        println!("{pool:#?}");
        println!("{deposit_metadata:#?}");
        println!("{loan_metadata:#?}");
        println!(
            "\nKnown changes that will be triggered by this instruction (may not be exhaustive):\n"
        );
        for (name, ctx, current_value) in [
            ("tokenMint", "pool", &pool.token_mint),
            ("pythProduct", "token", &token_metadata.pyth_product),
            ("pythPrice", "token", &token_metadata.pyth_price),
            ("pythPrice", "pool", &pool.token_price_oracle),
        ] {
            let addr = accounts
                .get(name)
                .context("could not find account {name} in this instruction.")?;
            if addr != current_value {
                println!(">>> {name} in {ctx}: {current_value} --> {addr}");
            }
        }
        for top in ix.data.try_as_struct()? {
            let inner = top.1.try_as_optional()?.as_ref();
            if let Some(inner) = inner {
                for (name, value) in inner.try_as_struct()? {
                    let kind = deposit_metadata.token_kind;
                    if name == "tokenKind"
                        && value.try_as_enum_tuple()?.0 != &format!("{:#?}", kind)
                    {
                        println!(">>> tokenKind: {:#?} --> {value:#?}", kind);
                    }
                    for (name_to_find, prior) in [
                        ("collateralWeight", deposit_metadata.value_modifier as u128),
                        ("maxLeverage", loan_metadata.value_modifier as u128),
                        ("flags", pool.config.flags as u128),
                        ("utilizationRate1", pool.config.utilization_rate_1 as u128),
                        ("utilizationRate2", pool.config.utilization_rate_2 as u128),
                        ("borrowRate0", pool.config.borrow_rate_0 as u128),
                        ("borrowRate1", pool.config.borrow_rate_1 as u128),
                        ("borrowRate2", pool.config.borrow_rate_2 as u128),
                        ("borrowRate3", pool.config.borrow_rate_3 as u128),
                        ("managementFeeRate", pool.config.management_fee_rate as u128),
                        ("reserved", pool.config.reserved as u128),
                    ] {
                        if name == name_to_find && value.try_as_integer_unsigned()? != prior {
                            println!(">>> {name}: {prior:#?} --> {value:#?}",);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// Download anchor account by name, using a map of names to pubkeys
async fn account_by_name<T: AccountDeserialize>(
    client: &Client,
    map: &HashMap<String, Pubkey>,
    name: &str,
) -> Result<T> {
    let addr = map
        .get(name)
        .context(format!("could not find account with name: {name}"))?;
    client.read_anchor_account::<T>(addr).await
}
