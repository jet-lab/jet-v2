use std::{fs::OpenOptions, io::Write, sync::Arc};

use anyhow::Result;
use hosted_tests::bonds::TestManager;
use hosted_tests::margin::MarginClient;
use jet_margin_sdk::ix_builder::get_metadata_address;
use jet_simulation::solana_rpc_api::RpcConnection;
use solana_sdk::signer::Signer;

lazy_static::lazy_static! {
    static ref CONFIG_PATH: String = shellexpand::env("$PWD/tests/integration/bonds/config.json").unwrap().to_string();
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let rpc = Arc::new(RpcConnection::new_local_funded().await?);

    let margin = MarginClient::new(rpc.clone());
    margin.init_globals().await?;
    margin.create_airspace_if_missing(false).await?;
    margin.create_authority_if_missing().await?;
    margin
        .register_adapter_if_unregistered(&jet_bonds::ID)
        .await?;

    let x = TestManager::new(
        rpc,
        &keys::mint(),
        &keys::event_queue(),
        &keys::bids(),
        &keys::asks(),
        keys::usdc_price().pubkey(),
    )
    .await?
    .with_margin()
    .await?;
    x.pause_orders().await?;

    {
        let json = format!(
            "{{ 
    \"jetBondsPid\": \"{}\",
    \"bondManager\": \"{}\",
    \"bondsMetadata\": \"{}\"
}}",
            jet_bonds::ID,
            x.ix_builder.manager(),
            get_metadata_address(&jet_bonds::ID),
        );
        let mut io = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(CONFIG_PATH.to_string())
            .unwrap();
        io.write_all(json.as_bytes()).unwrap();
    }

    println!("deployed bond manager to {:?}", x.ix_builder.manager());

    Ok(())
}

mod keys {
    json_keypairs! {
        // payer = "[222,147,115,219,200,207,183,34,103,192,44,23,43,203,127,70,67,170,118,146,40,128,166,176,91,184,240,89,157,92,138,41,12,48,55,127,230,6,125,75,21,171,39,213,6,155,83,215,2,250,164,163,97,165,211,0,204,244,39,28,66,112,134,180]";
        // authority = "[39,147,77,63,116,164,246,7,32,209,175,208,128,14,177,244,45,71,65,156,25,123,37,149,13,154,122,109,65,99,210,163,119,197,146,64,183,117,85,212,178,252,172,16,127,0,85,40,51,163,146,80,31,186,233,84,244,109,213,213,255,149,121,207]";
        // crank = "[78,122,206,47,0,102,125,42,154,126,250,137,110,198,174,2,137,75,111,54,34,93,221,115,77,222,133,247,129,233,156,0,50,26,219,183,209,148,208,168,131,217,2,159,31,202,77,155,22,129,62,12,119,47,130,91,28,192,91,204,32,21,101,165]";
        mint = "[246,43,252,198,120,201,142,112,177,111,236,88,172,135,87,184,164,70,237,7,72,62,112,62,26,76,246,196,206,41,214,63,105,138,110,229,84,226,231,32,107,197,42,155,38,138,222,153,230,189,220,238,198,171,252,15,180,216,131,6,122,162,129,153]";
        event_queue = "[94,75,127,91,165,7,129,112,195,242,198,228,161,243,228,13,175,213,152,141,87,63,215,122,244,13,68,36,166,238,59,116,72,80,134,219,183,121,88,125,92,49,111,20,66,30,171,185,93,158,56,137,132,172,109,91,108,136,215,56,12,149,85,4]";
        asks = "[141,46,10,183,108,199,29,225,29,29,79,221,122,71,28,133,182,245,47,17,101,231,6,38,125,150,148,161,131,96,28,132,195,111,31,15,79,201,185,178,150,94,2,203,120,181,183,93,52,59,81,229,164,62,136,115,7,250,184,73,142,99,59,15]";
        bids = "[16,106,193,60,13,228,72,228,213,162,191,66,14,80,153,128,225,183,237,191,150,198,34,125,254,145,173,242,168,71,19,43,142,97,100,204,81,253,220,145,191,229,103,250,132,174,223,78,92,123,252,104,172,20,109,24,208,100,43,194,195,152,113,235]";
        usdc_price = "[231,220,159,197,166,68,121,194,19,184,120,144,110,156,147,220,188,5,234,113,170,160,71,229,29,253,14,164,90,77,167,167,219,80,133,1,153,205,101,100,36,39,115,198,170,188,11,154,6,92,113,91,80,75,84,217,121,214,59,97,134,32,57,185]";
    }

    macro_rules! json_keypairs {
        ($($name:ident = $json:literal;)+) => {
            $(pub fn $name() -> solana_sdk::signature::Keypair {
                key_strings::get(key_strings::$name)
            })+
            mod key_strings {
                $(#[allow(non_upper_case_globals)] pub const $name: &str = $json;)+
                pub fn get(s: &str) -> solana_sdk::signature::Keypair {
                    solana_sdk::signature::read_keypair(&mut s.as_bytes().clone()).unwrap()
                }
            }
        };
    }
    use json_keypairs;
}
