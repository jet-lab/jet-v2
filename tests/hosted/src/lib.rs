use solana_sdk::signature::Keypair;

pub mod context;
pub mod load;
pub mod margin;
pub mod orchestrator;
pub mod setup_helper;
pub mod swap;
pub mod tokens;


pub fn clone(keypair: &Keypair) -> Keypair {
	Keypair::from_bytes(&keypair.to_bytes()).unwrap()
}
