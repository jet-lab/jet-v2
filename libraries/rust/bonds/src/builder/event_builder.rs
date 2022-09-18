use agnostic_orderbook::state::event_queue::{EventQueue, EventRef, FillEventRef, OutEventRef};
use jet_bonds::{
    margin::state::Obligation,
    orderbook::state::{CallbackFlags, CallbackInfo},
    tickets::state::SplitTicket,
};
use solana_sdk::pubkey::Pubkey;

use crate::builder::error::Result;

use super::ix_builder::bonds_pda;

/// Maximum byte size of the `ConsumeEventsInfo`, determined by solana transaction size
/// TODO: this is placeholder
const MAX_BYTES: usize = 1280;

/// Number of bytes in a loan account seed
const SEED_BYTES: usize = 8;

#[derive(Clone)]
pub enum EventAccountKeys {
    Fill(FillAccountsKeys),
    Out(OutAccountsKeys),
}

#[derive(Clone)]
pub struct LoanAccountKey {
    pub key: Pubkey,
    pub seed: Vec<u8>,
}

#[derive(Clone)]
pub struct FillAccountsKeys {
    pub maker: Pubkey,
    pub loan: Option<LoanAccountKey>,
    pub maker_adapter: Option<Pubkey>,
    pub taker_adapter: Option<Pubkey>,
}

#[derive(Clone)]
pub struct OutAccountsKeys {
    pub user: Pubkey,
    pub user_adapter_account: Option<Pubkey>,
}

#[derive(Default)]
pub struct ConsumeEventsInfo(Vec<EventAccountKeys>);

impl ConsumeEventsInfo {
    pub fn push(&mut self, keys: EventAccountKeys) {
        self.0.push(keys);
    }

    pub fn count_bytes(&self) -> usize {
        self.0
            .iter()
            .map(|e| match e {
                EventAccountKeys::Fill(FillAccountsKeys {
                    loan,
                    maker_adapter,
                    taker_adapter,
                    ..
                }) => {
                    let mut sum = 32usize;
                    if loan.is_some() {
                        sum += SEED_BYTES + 32;
                    }
                    if maker_adapter.is_some() {
                        sum += 32;
                    }
                    if taker_adapter.is_some() {
                        sum += 32;
                    }
                    sum
                }
                EventAccountKeys::Out(OutAccountsKeys {
                    user_adapter_account,
                    ..
                }) => {
                    let mut sum = 32usize;
                    if user_adapter_account.is_some() {
                        sum += 32;
                    }
                    sum
                }
            })
            .sum()
    }

    pub fn as_params(&self) -> (Vec<Pubkey>, u32, Vec<Vec<u8>>) {
        let mut keys = Vec::new();
        let mut seeds = Vec::new();

        for event in self.0.clone() {
            match event {
                EventAccountKeys::Fill(FillAccountsKeys {
                    maker,
                    loan,
                    maker_adapter,
                    taker_adapter,
                }) => {
                    keys.push(maker);
                    if let Some(acc) = loan {
                        keys.push(acc.key);
                        seeds.push(acc.seed.clone());
                    }
                    if let Some(key) = maker_adapter {
                        keys.push(key);
                    }
                    if let Some(key) = taker_adapter {
                        keys.push(key);
                    }
                }
                EventAccountKeys::Out(OutAccountsKeys {
                    user,
                    user_adapter_account,
                }) => {
                    keys.push(user);
                    if let Some(key) = user_adapter_account {
                        keys.push(key);
                    }
                }
            }
        }
        let len = keys.len() as u32;
        (keys, len, seeds)
    }
}

pub fn make_seed(rng: &mut impl rand::RngCore) -> Vec<u8> {
    let bytes = &mut [0u8; SEED_BYTES];
    rng.fill_bytes(bytes);
    bytes.to_vec()
}

pub fn build_consume_events_info(
    event_queue: EventQueue<'_, CallbackInfo>,
    rng: &mut impl rand::RngCore,
) -> Result<ConsumeEventsInfo> {
    let mut info = ConsumeEventsInfo::default();

    for event in event_queue.iter() {
        if info.count_bytes() > MAX_BYTES {
            break;
        }
        let keys = match event {
            EventRef::Fill(FillEventRef {
                maker_callback_info,
                taker_callback_info,
                ..
            }) => {
                let loan = if maker_callback_info
                    .flags
                    .contains(CallbackFlags::AUTO_STAKE)
                {
                    let seed = make_seed(rng);
                    let key =
                        bonds_pda(&SplitTicket::make_seeds(&maker_callback_info.owner, &seed));
                    Some(LoanAccountKey { key, seed })
                } else if maker_callback_info.flags.contains(CallbackFlags::NEW_DEBT) {
                    let seed = make_seed(rng);
                    let key = bonds_pda(&Obligation::make_seeds(
                        &maker_callback_info.fill_account,
                        &seed,
                    ));
                    Some(LoanAccountKey { key, seed })
                } else {
                    None
                };

                EventAccountKeys::Fill(FillAccountsKeys {
                    maker: Pubkey::new_from_array(maker_callback_info.fill_account),
                    loan,
                    maker_adapter: maker_callback_info.adapter(),
                    taker_adapter: taker_callback_info.adapter(),
                })
            }
            EventRef::Out(OutEventRef { callback_info, .. }) => {
                EventAccountKeys::Out(OutAccountsKeys {
                    user: Pubkey::new_from_array(callback_info.fill_account),
                    user_adapter_account: callback_info.adapter(),
                })
            }
        };
        info.push(keys)
    }

    Ok(info)
}
