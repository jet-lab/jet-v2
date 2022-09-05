use agnostic_orderbook::state::event_queue::{EventQueue, EventRef, FillEventRef, OutEventRef};
use anchor_client::{
    anchor_lang::AccountDeserialize,
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair,
        signature::Signature, signer::Signer, transaction::Transaction,
    },
};
use anyhow::Result;
use jet_bonds::{
    control::state::BondManager,
    orderbook::{
        instructions::lender_borrower,
        state::{CallbackFlags, CallbackInfo},
    },
};
use jet_bonds_lib::builder::BondsIxBuilder;
use rand::{rngs::OsRng, RngCore};

/// Maximum number of accounts a single consume_events transaction can support
const MAX_ACCOUNTS: usize = 12;

pub struct Context {
    pub endpoint: String,
    pub signer: Keypair,
    pub payer: Keypair,
    pub bond_manager_key: Pubkey,
}

impl Context {
    pub fn run(self) -> Result<()> {
        let connection =
            RpcClient::new_with_commitment(self.endpoint.clone(), CommitmentConfig::confirmed());

        let event_queue_key = {
            let data = connection.get_account(&self.bond_manager_key).unwrap().data;
            BondManager::try_deserialize(&mut data.as_slice())
                .unwrap()
                .event_queue
        };

        let builder = BondsIxBuilder::new(self.bond_manager_key)
            .with_crank(&self.signer.pubkey())
            .with_payer(&self.payer.pubkey())
            .with_orderbook_accounts(None, None, Some(event_queue_key));

        loop {
            let res = consume_events_iteration(
                &connection,
                &builder,
                &event_queue_key,
                &self.signer,
                &self.payer,
            );
            match res {
                Ok(_) => (),
                Err(e) => {
                    if e.to_string() == "No events" {
                        continue;
                    }

                    println!("{e:#?}")
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn consume_events_iteration(
    connection: &RpcClient,
    ix_builder: &BondsIxBuilder,
    eq_key: &Pubkey,
    signer: &Keypair,
    payer: &Keypair,
) -> Result<Signature> {
    //query the state of the event queue
    let mut event_queue_data = connection.get_account_data(eq_key)?;
    let event_queue = EventQueue::from_buffer(
        &mut event_queue_data,
        agnostic_orderbook::state::AccountTag::EventQueue,
    )?;

    // no events, keep looping
    if event_queue.iter().next().is_none() {
        return Err(anyhow::Error::msg("No events"));
    }

    // Build and send transaction

    let (event_accounts, num_events, seeds) =
        populate_event_accounts(event_queue, &mut OsRng::default());
    let remaining_accounts = event_accounts.iter().collect::<Vec<&Pubkey>>();

    let ix = ix_builder.consume_events(remaining_accounts, num_events as u32, seeds)?;
    let recent_blockhash = connection.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[signer, payer],
        recent_blockhash,
    );

    Ok(connection.send_and_confirm_transaction(&tx)?)
}

pub fn populate_event_accounts(
    event_queue: EventQueue<'_, CallbackInfo>,
    rng: &mut impl RngCore,
) -> (Vec<Pubkey>, usize, Vec<Vec<u8>>) {
    let mut event_accounts = Vec::<Pubkey>::new();
    let mut num_events = 0;
    let mut seeds = Vec::new();
    for event in event_queue.iter() {
        if event_accounts.len() > MAX_ACCOUNTS {
            break;
        }
        match event {
            EventRef::Fill(FillEventRef {
                event,
                maker_callback_info,
                taker_callback_info,
            }) => {
                let (lender_info, borrower_info) =
                    lender_borrower(event.taker_side, maker_callback_info, taker_callback_info);
                event_accounts.push(Pubkey::new_from_array(lender_info.user_vault));
                event_accounts.push(Pubkey::new_from_array(borrower_info.user_vault));

                if borrower_info.flags.contains(CallbackFlags::NEW_DEBT) {
                    event_accounts.push(Pubkey::new_from_array(borrower_info.account_key));
                }
                if lender_info.flags.contains(CallbackFlags::AUTO_STAKE) {
                    let mut bytes = [0u8; 4];
                    rng.try_fill_bytes(&mut bytes).unwrap();
                    let (ticket_key, _) = Pubkey::find_program_address(
                        &[
                            jet_bonds::seeds::SPLIT_TICKET,
                            Pubkey::new_from_array(lender_info.account_key).as_ref(),
                            &bytes,
                        ],
                        &jet_bonds::ID,
                    );
                    seeds.push(bytes.to_vec());
                    event_accounts.push(ticket_key);
                }
                if lender_info.adapter_account_key != Pubkey::default().to_bytes() {
                    event_accounts.push(Pubkey::new_from_array(lender_info.adapter_account_key));
                }
                if borrower_info.flags.contains(CallbackFlags::NEW_DEBT) {
                    let mut bytes = [0u8; 4];
                    rng.try_fill_bytes(&mut bytes).unwrap();
                    let (obligation_key, _) = Pubkey::find_program_address(
                        &[
                            jet_bonds::seeds::OBLIGATION,
                            Pubkey::new_from_array(borrower_info.account_key).as_ref(),
                            &bytes,
                        ],
                        &jet_bonds::ID,
                    );
                    seeds.push(bytes.to_vec());
                    event_accounts.push(obligation_key);
                }
                if borrower_info.adapter_account_key != Pubkey::default().to_bytes() {
                    event_accounts.push(Pubkey::new_from_array(borrower_info.adapter_account_key));
                }
            }
            EventRef::Out(OutEventRef { callback_info, .. }) => {
                let user_key = Pubkey::new_from_array(callback_info.account_key);
                event_accounts.extend_from_slice(&[user_key]);
                if callback_info.adapter_account_key != Pubkey::default().to_bytes() {
                    event_accounts.push(Pubkey::new_from_array(callback_info.adapter_account_key));
                }
            }
        }
        num_events += 1;
    }
    (event_accounts, num_events, seeds)
}
