use anchor_lang::AccountDeserialize;
use anyhow::Result;
use clap::{Parser, Subcommand};
use jet_fixed_term::{
    control::state::Market as FixedTermMarket,
    margin::state::{MarginUser, TermLoan},
    tickets::state::TermDeposit,
};
use jet_margin_sdk::fixed_term::{
    derive::{term_deposit, term_loan},
    ix::recover_uninitialized,
    FixedTermIxBuilder, InitializeMarketParams, OrderbookAddresses, OwnedEventQueue,
    FIXED_TERM_PROGRAM,
};
use jet_program_common::{GOVERNOR_DEVNET, GOVERNOR_MAINNET};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::serde_as;
use solana_account_decoder::UiAccountEncoding;
use solana_clap_utils::keypair::signer_from_path;
use solana_client::{
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::RpcFilterType,
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signer::Signer};

use crate::{
    client::{Client, NetworkKind, Plan},
    governance::resolve_payer,
};

const MANAGER_VERSION: u64 = 0;

#[derive(Debug, Clone, Parser, Serialize, Deserialize)]
pub struct MarketParameters {
    #[clap(long)]
    pub borrow_tenor: u64,

    #[clap(long)]
    pub lend_tenor: u64,

    #[clap(long)]
    pub origination_fee: u64,

    #[clap(long)]
    pub min_order_size: u64,

    #[clap(long)]
    pub seed: Vec<u8>,

    #[clap(long)]
    pub token_mint: Pubkey,

    #[clap(long)]
    pub token_oracle: Pubkey,

    #[clap(long)]
    pub ticket_oracle: Pubkey,

    #[clap(long)]
    pub event_queue: String,

    #[clap(long)]
    pub bids: String,

    #[clap(long)]
    pub asks: String,
}

#[serde_as]
#[derive(Debug, Subcommand, Deserialize)]
#[serde(tag = "fixed-term-display")]
pub enum FixedTermDisplayCmd {
    /// Display Fixed Term markets
    Markets {
        /// optional, fetch a specific market by key
        #[clap(long)]
        pubkey: Option<Pubkey>,

        /// also deserialize and display events waiting in the event queue
        #[clap(long)]
        display_events: bool,
    },

    /// Display Fixed Term Margin Users
    Users {
        /// optional, fetch a specific MarginUser by pubkey
        #[clap(long)]
        pubkey: Option<Pubkey>,

        /// optional, only fetch users from a particular market
        #[clap(long)]
        market: Option<Pubkey>,

        /// display term loans assosciated with user
        #[clap(long)]
        loans: bool,

        /// display term deposits assosciated with user
        #[clap(long)]
        deposits: bool,
    },

    Accounts {
        /// filter by market
        #[clap(long)]
        market: Option<Pubkey>,

        /// display all term deposits
        #[clap(long)]
        deposits: bool,

        /// display all term loans
        #[clap(long)]
        loans: bool,
    },
}

fn map_seed(seed: Vec<u8>) -> [u8; 32] {
    let mut buf = [0u8; 32];
    let mut iter = seed.into_iter();

    // clippy go away, I cant use `write` on a fixed array
    #[allow(clippy::needless_range_loop)]
    for i in 0..buf.len() {
        match iter.next() {
            Some(b) => buf[i] = b,
            None => break,
        }
    }

    buf
}

pub async fn process_recover_uninitialized(client: &Client, recipient: Pubkey) -> Result<Plan> {
    let ft_accounts = client
        .rpc()
        .get_program_accounts(&FIXED_TERM_PROGRAM)
        .await?;
    let mut plan = client.plan()?;

    let governor = match client.network_kind {
        NetworkKind::Localnet => client.signer()?,
        NetworkKind::Devnet => GOVERNOR_DEVNET,
        NetworkKind::Mainnet => GOVERNOR_MAINNET,
    };

    for (address, account) in ft_accounts {
        if account.data[..8] != [0u8; 8] {
            continue;
        }

        plan = plan.instructions(
            [],
            [format!("recover {address}")],
            [recover_uninitialized(governor, address, recipient)],
        );
    }

    Ok(plan.build())
}

pub async fn process_create_fixed_term_market<'a>(
    client: &Client,
    params: MarketParameters,
) -> Result<Plan> {
    let payer = resolve_payer(client)?;
    let seed = map_seed(params.seed);
    let [eq, bids, asks] = [
        signer_from_path(
            &Default::default(),
            &params.event_queue,
            "event_queue",
            &mut None,
        )
        .map_err(|e| {
            anyhow::Error::msg(format!(
                "failed to resolve signer for event queue. Error: {e:?}"
            ))
        })?,
        signer_from_path(&Default::default(), &params.bids, "bids", &mut None).map_err(|e| {
            anyhow::Error::msg(format!("failed to resolve signer for bids. Error: {e:?}"))
        })?,
        signer_from_path(&Default::default(), &params.asks, "asks", &mut None).map_err(|e| {
            anyhow::Error::msg(format!("failed to resolve signer for asks. Error: {e:?}"))
        })?,
    ];
    let fixed_term_market = FixedTermIxBuilder::new_from_seed(
        client.signer()?,
        &Pubkey::default(),
        &params.token_mint,
        seed,
        payer,
        params.token_oracle,
        params.ticket_oracle,
        None,
        OrderbookAddresses {
            bids: bids.pubkey(),
            asks: asks.pubkey(),
            event_queue: eq.pubkey(),
        },
    );

    let mut steps = vec![];
    let mut instructions = vec![];
    if client.account_exists(&fixed_term_market.market()).await? {
        println!(
            "the fixed term market [{}] already exists. Skipping initialization instruction",
            fixed_term_market.market()
        );
    } else if !client.account_exists(&params.token_mint).await? {
        println!("the token {} does not exist", params.token_mint);
        return Ok(Plan::default());
    } else {
        if let Some(init_ata) = fixed_term_market.init_default_fee_destination(&payer) {
            instructions.push(init_ata);
        }
        let init_market = fixed_term_market.initialize_market(
            payer,
            InitializeMarketParams {
                version_tag: MANAGER_VERSION,
                seed,
                borrow_tenor: params.borrow_tenor,
                lend_tenor: params.lend_tenor,
                origination_fee: params.origination_fee,
            },
        );
        steps.push(format!(
            "initialize-market for token [{}]",
            params.token_mint
        ));
        instructions.push(init_market);
    }
    if client
        .account_exists(&fixed_term_market.orderbook_state())
        .await?
    {
        println!(
            "the market [{}] is already fully initialized",
            fixed_term_market.market()
        );
        return Ok(Plan::default());
    }
    let init_orderbook = fixed_term_market.initialize_orderbook(payer, params.min_order_size);
    steps.push(format!(
        "initialize-order-book for fixed term market {}",
        fixed_term_market.market()
    ));
    instructions.push(init_orderbook);

    let signers: Vec<Box<dyn Signer>> = vec![eq, bids, asks];

    Ok(client
        .plan()?
        .instructions(signers, steps, instructions)
        .build())
}

pub async fn process_display_fixed_term_accounts(
    client: &Client,
    cmd: FixedTermDisplayCmd,
) -> Result<()> {
    use FixedTermDisplayCmd::*;
    match cmd {
        Markets {
            pubkey,
            display_events,
        } => {
            let markets = if let Some(key) = pubkey {
                let market: FixedTermMarket = client.read_anchor_account(&key).await?;
                vec![(key, market)]
            } else {
                get_fixed_term_accounts(client).await?
            };

            println!(
                "Displaying information for {} fixed term markets",
                markets.len()
            );

            for market in markets {
                let mut ser = serde_json::Serializer::new(vec![]);
                market.1.serialize(&mut ser)?;
                let json: Value = serde_json::from_slice(ser.into_inner().as_slice())?;

                println!("Market [{}]", market.0);
                println!("{:#}", json);

                if display_events {
                    let buff = client.rpc().get_account_data(&market.1.event_queue).await?;
                    let mut eq = OwnedEventQueue::from(buff);
                    println!("Pending events: ");
                    for event in eq.inner()?.iter() {
                        println!("{event:?}");
                    }
                }
            }
        }
        Users {
            pubkey,
            market,
            loans,
            deposits,
        } => {
            let users = if let Some(key) = pubkey {
                let user = client.read_anchor_account::<MarginUser>(&key).await?;
                vec![(key, user)]
            } else if let Some(key) = market {
                get_fixed_term_accounts::<MarginUser>(client)
                    .await?
                    .into_iter()
                    .filter(|(_, u)| u.market == key)
                    .collect()
            } else {
                get_fixed_term_accounts(client).await?
            };

            println!("Displaying users: ");
            for user in users {
                println!("User: [{}]", user.0);
                println!("{}", user.1);

                if loans {
                    let keys = user
                        .1
                        .debt()
                        .active_loans()
                        .map(|n| term_loan(&user.1.market, &user.0, n))
                        .collect::<Vec<_>>();
                    let loans = client
                        .rpc()
                        .get_multiple_accounts(keys.as_slice())
                        .await?
                        .into_iter()
                        .zip(keys.into_iter())
                        .filter_map(|(a, k)| {
                            match TermLoan::try_deserialize(&mut a?.data.as_ref()) {
                                Ok(loan) => Some((k, loan)),
                                _ => None,
                            }
                        })
                        .collect::<Vec<_>>();
                    println!("Displaying loans for user [{}]", user.0);
                    for loan in loans {
                        println!("Loan: [{}]", loan.0);
                        println!("{:#?}", loan.1);
                    }
                }

                if deposits {
                    let keys = user
                        .1
                        .assets()
                        .active_deposits()
                        .map(|n| term_deposit(&user.1.market, &user.1.margin_account, n))
                        .collect::<Vec<_>>();
                    let deposits = client
                        .rpc()
                        .get_multiple_accounts(keys.as_slice())
                        .await?
                        .into_iter()
                        .zip(keys.into_iter())
                        .filter_map(|(a, k)| {
                            match TermDeposit::try_deserialize(&mut a?.data.as_ref()) {
                                Ok(deposit) => Some((k, deposit)),
                                _ => None,
                            }
                        })
                        .collect::<Vec<_>>();

                    println!("Displaying deposits for user [{}]", user.0);
                    for deposit in deposits {
                        println!("Deposit: [{}]", deposit.0);
                        println!("{:#?}", deposit.1);
                    }
                }
            }
        }
        Accounts {
            market,
            deposits,
            loans,
        } => {
            if deposits {
                let mut deposits = get_fixed_term_accounts::<TermDeposit>(client).await?;
                if let Some(market) = market {
                    deposits.retain(|d| d.1.market == market);
                }
                for deposit in deposits {
                    println!("TermDeposit: [{}]\n{:#?}", deposit.0, deposit.1);
                }
            }
            if loans {
                let mut loans = get_fixed_term_accounts::<TermLoan>(client).await?;
                if let Some(market) = market {
                    loans.retain(|l| l.1.market == market);
                }
                for loan in loans {
                    println!("TermLoan: [{}]\n{:#?}", loan.0, loan.1);
                }
            }
        }
    }

    Ok(())
}

async fn get_fixed_term_accounts<T: AccountDeserialize>(
    client: &Client,
) -> Result<Vec<(Pubkey, T)>> {
    Ok(client
        .rpc()
        .get_program_accounts_with_config(
            &jet_fixed_term::ID,
            RpcProgramAccountsConfig {
                filters: Some(vec![RpcFilterType::DataSize(
                    std::mem::size_of::<T>() as u64 + 8,
                )]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    data_slice: None,
                    commitment: Some(CommitmentConfig::processed()),
                    min_context_slot: None,
                },
                with_context: None,
            },
        )
        .await?
        .into_iter()
        .filter_map(|(k, a)| match T::try_deserialize(&mut a.data.as_ref()) {
            Ok(market) => Some((k, market)),
            _ => None,
        })
        .collect())
}
