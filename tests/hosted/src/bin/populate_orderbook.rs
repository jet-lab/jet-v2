use anchor_lang::AccountDeserialize;
use anyhow::Result;
use jet_bonds::control::state::BondManager;
use jet_margin_sdk::bonds::{BondsIxBuilder, OrderParams};
use solana_client::{rpc_client::RpcClient, rpc_config::RpcSendTransactionConfig};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};

const ENDPOINT: &str = "https://api.devnet.solana.com";

const FAUCET_PID: Pubkey = pubkey!("4bXpkKSV8swHSnwqtzuboGPaPDeEgAn4Vt8GfarV5rZt");
const DEVNET_USDC: Pubkey = pubkey!("4ruM7B4Hz4MUxy7DSFBRK9zCFLvkbLccB6S3zJ7t2525");
const DEVNET_USDC_FAUCET: Pubkey = pubkey!("MV2QoKwWmRQnu8HY56Hsmfhb6aC6L6mLirmQ5Houo9m");

const TOKEN_AMOUNT: u64 = 10_000_000_000;
const TICKET_AMOUNT: u64 = 5_000_000_000;

lazy_static::lazy_static! {
    static ref PAYER: String = shellexpand::env("$PWD/tests/keypairs/payer.json")
    .unwrap().to_string();
    static ref BOB: String = shellexpand::env("$PWD/tests/keypairs/payer.json")
    .unwrap().to_string();
    static ref ALICE: String = shellexpand::env("$PWD/tests/keypairs/payer.json")
    .unwrap().to_string();
}

fn map_keypair_file(path: String) -> Result<Keypair> {
    solana_clap_utils::keypair::keypair_from_path(&Default::default(), &path, "", false)
        .map_err(|_| anyhow::Error::msg("failed to read keypair"))
}

struct Client {
    conn: RpcClient,
    ix: BondsIxBuilder,
    signer: Keypair,
}

impl Client {
    pub fn new(conn: RpcClient, signer: Keypair, mint: Pubkey, seed: [u8; 32]) -> Result<Self> {
        let mut ix = BondsIxBuilder::new_from_seed(&mint, seed, signer.pubkey())
            .with_payer(&signer.pubkey());
        let bond_manager = {
            let data = conn.get_account_data(&ix.manager())?;

            BondManager::try_deserialize(&mut data.as_slice())?
        };

        ix = ix.with_orderbook_accounts(
            Some(bond_manager.bids),
            Some(bond_manager.asks),
            Some(bond_manager.event_queue),
        );

        Ok(Self { conn, ix, signer })
    }
    pub fn sign_send_transaction(
        &self,
        instructions: &[Instruction],
        add_signers: &[&Keypair],
    ) -> Result<Signature> {
        let mut keypairs = vec![&self.signer];
        keypairs.extend_from_slice(add_signers);

        let recent_blockhash = self.conn.get_latest_blockhash()?;
        let mut tx =
            Transaction::new_unsigned(Message::new(instructions, Some(&self.signer.pubkey())));
        for signer in tx.message().clone().signer_keys() {
            for kp in keypairs.clone() {
                if &kp.pubkey() == signer {
                    tx.partial_sign(&[kp], recent_blockhash);
                }
            }
        }

        self.conn
            .send_transaction_with_config(
                &tx,
                RpcSendTransactionConfig {
                    skip_preflight: true,
                    ..Default::default()
                },
            )
            .map_err(anyhow::Error::from)
    }
}

struct User<'a> {
    client: &'a Client,
    kp: Keypair,
}

impl<'a> User<'a> {
    pub fn new(client: &'a Client, kp: Keypair) -> Result<Self> {
        Ok(Self { client, kp })
    }

    pub fn key(&self) -> Pubkey {
        self.kp.pubkey()
    }

    pub fn token_wallet(&self) -> Pubkey {
        get_associated_token_address(&self.key(), &self.client.ix.token_mint())
    }
    pub fn ticket_wallet(&self) -> Pubkey {
        get_associated_token_address(&self.key(), &self.client.ix.ticket_mint())
    }

    pub fn init_and_fund(&self, token_amount: u64, ticket_amount: u64) -> Result<()> {
        let init_token = create_associated_token_account(
            &self.client.signer.pubkey(),
            &self.key(),
            &self.client.ix.token_mint(),
        );
        let init_ticket = create_associated_token_account(
            &self.client.signer.pubkey(),
            &self.key(),
            &self.client.ix.ticket_mint(),
        );

        let fund_token = airdrop_ix(
            &self.token_wallet(),
            &DEVNET_USDC_FAUCET,
            &DEVNET_USDC,
            token_amount,
        );
        let fund_ticket =
            self.client
                .ix
                .convert_tokens(Some(&self.key()), None, None, None, ticket_amount)?;

        self.send_instructions(&[init_token, init_ticket, fund_token, fund_ticket])?;
        println!("funding success!");
        Ok(())
    }

    pub fn lend_order(&self, params: OrderParams) -> Result<()> {
        let lend = self
            .client
            .ix
            .lend_order(&self.key(), None, None, params, vec![])?;

        self.send_instructions(&[lend])
    }
    pub fn borrow_order(&self, params: OrderParams) -> Result<()> {
        let borrow = self
            .client
            .ix
            .sell_tickets_order(&self.key(), None, None, params)?;

        self.send_instructions(&[borrow])
    }

    fn send_instructions(&self, instructions: &[Instruction]) -> Result<()> {
        for ix in instructions {
            dbg!(self
                .client
                .sign_send_transaction(&[ix.clone()], &[&self.kp])?);
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let conn = RpcClient::new_with_commitment(ENDPOINT, CommitmentConfig::confirmed());
    let wallet = map_keypair_file(PAYER.clone())?;
    let alice_kp = map_keypair_file(ALICE.clone())?;
    let bob_kp = map_keypair_file(BOB.clone())?;

    let client = Client::new(conn, wallet, DEVNET_USDC, Pubkey::default().to_bytes())?;

    let alice = User::new(&client, alice_kp)?;
    let bob = User::new(&client, bob_kp)?;

    alice.init_and_fund(TOKEN_AMOUNT, TICKET_AMOUNT)?;
    bob.init_and_fund(TOKEN_AMOUNT, TICKET_AMOUNT)?;

    // let asks_data = &mut client.conn.get_account_data(&client.ix.asks()?)?;
    // let asks = agnostic_orderbook::state::critbit::Slab::<jet_bonds::orderbook::state::CallbackInfo>::from_buffer(
    //         asks_data,
    //         agnostic_orderbook::state::AccountTag::Asks,
    //     )?;
    // let bids_data = &mut client.conn.get_account_data(&client.ix.bids()?)?;
    // let bids = agnostic_orderbook::state::critbit::Slab::<jet_bonds::orderbook::state::CallbackInfo>::from_buffer(
    //         bids_data,
    //         agnostic_orderbook::state::AccountTag::Bids,
    //     )?;

    // dbg!(bids.into_iter(true).next().is_some());
    // dbg!(asks.into_iter(true).next().is_some());

    Ok(())
}

fn airdrop_ix(token_account: &Pubkey, faucet: &Pubkey, mint: &Pubkey, amount: u64) -> Instruction {
    let mut data = vec![1];
    data.extend_from_slice(&amount.to_le_bytes());

    let pk_nonce = Pubkey::find_program_address(&[b"faucet"], &FAUCET_PID).0;
    let keys = vec![
        AccountMeta::new_readonly(pk_nonce, false),
        AccountMeta::new(*mint, false),
        AccountMeta::new(*token_account, false),
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(*faucet, false),
    ];
    Instruction::new_with_bytes(FAUCET_PID, &data, keys)
}
