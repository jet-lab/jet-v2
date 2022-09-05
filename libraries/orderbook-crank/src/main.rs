use anchor_client::solana_sdk::signature::read_keypair_file;
use clap::{App, Arg};
use jet_bonds_orderbook_crank::Context;
use solana_clap_utils::{input_parsers::pubkey_of, input_validators::is_pubkey};

const DEFAULT_RPC: &str = "http://127.0.0.1:8899";

fn main() {
    let matches = App::new("bonds-orderbook-crank")
        .arg(
            Arg::with_name("url")
                .short("u")
                .long("url")
                .help("The RPC endpoint to use")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("signer-keypair-file")
                .short("s")
                .long("signer-keypair-file")
                .help("The keypair to use for signing consume events transactions")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("payer-keypair-file")
                .short("p")
                .long("payer-keypair-file")
                .help("The keypair to use to pay for the instruction calls")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("bond-manager-key")
                .long("bond-manager-key")
                .help("The pubkey of the bond manager account")
                .takes_value(true)
                .validator(is_pubkey)
                .required(true),
        )
        .get_matches();

    let endpoint = matches.value_of("url").unwrap_or(DEFAULT_RPC).to_string();
    let payer = read_keypair_file(matches.value_of("payer-keypair-file").unwrap()).unwrap();
    let signer = read_keypair_file(matches.value_of("signer-keypair-file").unwrap()).unwrap();
    let bond_manager_key =
        pubkey_of(&matches, "bond-manager-key").expect("Invalid bond manager pubkey.");

    Context {
        endpoint,
        signer,
        payer,
        bond_manager_key,
    }
    .run();
}
