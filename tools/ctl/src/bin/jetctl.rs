use clap::Parser;
use jet_margin_sdk::ix_builder::staking::{derive_stake_account, derive_stake_pool};
use jetctl::CliOpts;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    use jet_instructions::staking::STAKING_PROGRAM;
    use solana_sdk::{pubkey, pubkey::Pubkey};

    let user = Pubkey::default();
    let pool = pubkey!("4o7XLNe2NYtcxhFpiXYKSobgodsuQvHgxKriDiYqE2tP");
    let mut seedvec = Vec::new();
    seedvec.extend(pool.as_ref());
    seedvec.extend(user.as_ref());

    let a = Pubkey::find_program_address(&[&seedvec], &STAKING_PROGRAM).0;
    let b = derive_stake_account(&pool, &user);

    println!("{a} {b}");

    if let Err(e) = jetctl::run(CliOpts::parse()).await {
        println!("error: ");

        for err in e.chain() {
            println!("{err}");
        }

        println!("{}", e.backtrace());
    }
    Ok(())
}
