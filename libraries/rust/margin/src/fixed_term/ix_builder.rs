use agnostic_orderbook::state::{event_queue::EventQueue, AccountTag};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

use jet_fixed_term::orderbook::state::CallbackInfo;

use super::error::{FixedTermMarketIxError, Result};

pub use jet_instructions::fixed_term::*;

pub fn init_account(payer: &Pubkey, slab: &Pubkey, space: usize, rent: u64) -> Instruction {
    solana_sdk::system_instruction::create_account(
        payer,
        slab,
        rent,
        space as u64,
        &jet_fixed_term::ID,
    )
}

/// Convenience struct for passing around an `EventQueue`
#[derive(Clone)]
pub struct OwnedEventQueue(Vec<u8>);

impl OwnedEventQueue {
    pub fn inner(&mut self) -> Result<EventQueue<CallbackInfo>> {
        EventQueue::from_buffer(&mut self.0, AccountTag::EventQueue)
            .map_err(|e| FixedTermMarketIxError::Deserialization(e.to_string()))
    }

    pub fn is_empty(&mut self) -> Result<bool> {
        Ok(self.inner()?.iter().next().is_none())
    }
}

impl<T: Into<Vec<u8>>> From<T> for OwnedEventQueue {
    fn from(data: T) -> Self {
        Self(data.into())
    }
}
