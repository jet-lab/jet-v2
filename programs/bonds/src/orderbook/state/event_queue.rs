use std::{cell::RefCell, rc::Rc};

use agnostic_orderbook::state::event_queue::{EventQueueHeader, FillEvent, OutEvent};
use anchor_lang::prelude::*;
use bytemuck::{CheckedBitPattern, NoUninit};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use super::CallbackInfo;

#[account(zero_copy)]
pub struct EventAdapterMetadata {
    /// Signing authority over this Adapter
    pub owner: Pubkey,
    /// The `BondManager` this adapter belongs to
    pub manager: Pubkey,
    /// The `OrderbookUser` account this adapter is registered for
    pub orderbook_user: Pubkey,
}

impl EventAdapterMetadata {
    pub const LEN: usize = std::mem::size_of::<Self>();

    pub fn space(num_events: u32) -> usize {
        num_events as usize * (FillEvent::LEN + 2 * CallbackInfo::LEN)
            + Self::LEN
            + EventQueueHeader::LEN
            + 16 // anchor discriminator and agnostic-orderbook tag
    }
}

#[derive(FromPrimitive, Clone, Copy, CheckedBitPattern, NoUninit)]
#[repr(u8)]
pub(crate) enum EventTag {
    Fill,
    Out,
}

pub(crate) type GenericEvent = FillEvent;

pub trait Event {
    fn to_generic(&mut self) -> &GenericEvent;
}

impl Event for FillEvent {
    fn to_generic(&mut self) -> &GenericEvent {
        self.tag = EventTag::Fill as u8;
        self
    }
}

impl Event for OutEvent {
    fn to_generic(&mut self) -> &GenericEvent {
        self.tag = EventTag::Out as u8;
        bytemuck::cast_ref(self)
    }
}

pub enum OrderbookEvent {
    Fill {
        event: FillEvent,
        maker_info: CallbackInfo,
        taker_info: CallbackInfo,
    },
    Out {
        event: OutEvent,
        info: CallbackInfo,
    },
}

#[derive(Clone)]
pub struct EventQueue<'a> {
    data: Rc<RefCell<&'a mut [u8]>>,
    header: EventQueueHeader,
    capacity: usize,
    event_ptr: usize,
    callback_ptr: usize,
    is_adapter: bool,
}

impl<'a> EventQueue<'a> {
    const ADAPTER_OFFSET: usize = EventAdapterMetadata::LEN + 8;

    /// Checks should be done before instanciating an AdapterEventQueue to assert proper ownership
    pub fn from_data(data: Rc<RefCell<&'a mut [u8]>>) -> Result<Self> {
        let buf = &data.borrow();
        let capacity =
            (buf.len() - 8 - EventQueueHeader::LEN) / (FillEvent::LEN + 2 * CallbackInfo::LEN);

        let header_ptr = 8;
        let event_ptr = header_ptr + EventQueueHeader::LEN;
        let callback_ptr = event_ptr + capacity * FillEvent::LEN;

        let header = EventQueueHeader::deserialize(&mut &buf[header_ptr..])?;

        Ok(Self {
            data: data.clone(),
            event_ptr,
            capacity,
            callback_ptr,
            header,
            is_adapter: false,
        })
    }

    pub fn new_adapter(data: Rc<RefCell<&'a mut [u8]>>) -> Result<Self> {
        let buf = &data.borrow();
        let capacity = (buf.len() - 8 - EventQueueHeader::LEN - Self::ADAPTER_OFFSET)
            / (FillEvent::LEN + 2 * CallbackInfo::LEN);

        let header_ptr = 8 + Self::ADAPTER_OFFSET;
        let event_ptr = header_ptr + EventQueueHeader::LEN;
        let callback_ptr = event_ptr + capacity * FillEvent::LEN;

        let header = EventQueueHeader::deserialize(&mut &buf[header_ptr..])?;

        Ok(Self {
            data: data.clone(),
            capacity,
            event_ptr,
            callback_ptr,
            header,
            is_adapter: true,
        })
    }

    /// Pushes the given event to the back of the queue
    pub fn push_event<E: Event>(
        &mut self,
        mut event: E,
        maker_callback_info: Option<&CallbackInfo>,
        taker_callback_info: Option<&CallbackInfo>,
    ) -> std::result::Result<(), Error> {
        let mut buf = self.data.borrow_mut();
        let generic_event = event.to_generic();
        let event_idx = (self.header.head as usize + self.header.count as usize) % self.capacity;

        let events: &mut [FillEvent] =
            bytemuck::cast_slice_mut(&mut buf[self.event_ptr..self.callback_ptr]);
        events[event_idx] = *generic_event;

        self.header.count += 1;

        let callback_infos: &mut [CallbackInfo] =
            bytemuck::cast_slice_mut(&mut buf[self.callback_ptr..]);
        if let Some(c) = maker_callback_info {
            callback_infos[event_idx * 2] = *c;
        }

        if let Some(c) = taker_callback_info {
            callback_infos[event_idx * 2 + 1] = *c;
        }

        Ok(())
    }

    /// Attempts to remove the number of events from the top of the queue
    pub fn pop_events(&mut self, num_events: u32) -> Result<()> {
        let capped_number_of_entries_to_pop = std::cmp::min(self.header.count, num_events as u64);
        self.header.count -= capped_number_of_entries_to_pop;
        self.header.head =
            (self.header.head + capped_number_of_entries_to_pop) % (self.capacity as u64);
        Ok(())
    }

    fn get_event(&self, event_idx: usize) -> OrderbookEvent {
        let buf = self.data.borrow();
        let events: &[FillEvent] = bytemuck::cast_slice(&buf[self.event_ptr..self.callback_ptr]);
        let callback: &[CallbackInfo] = bytemuck::cast_slice(&buf[self.callback_ptr..]);

        let event = &events[event_idx];
        match EventTag::from_u8(event.tag).unwrap() {
            EventTag::Fill => OrderbookEvent::Fill {
                event: *event,
                maker_info: callback[2 * event_idx],
                taker_info: callback[2 * event_idx + 1],
            },
            EventTag::Out => OrderbookEvent::Out {
                event: *bytemuck::cast_ref(event),
                info: callback[2 * event_idx],
            },
        }
    }

    pub fn iter(&self) -> QueueIterator<'a> {
        QueueIterator {
            queue: self.clone(),
            current_index: 0,
            remaining: self.header.count,
        }
    }
}

impl<'a> Drop for EventQueue<'a> {
    fn drop(&mut self) {
        let mut buf = self.data.borrow_mut();

        let offset = match self.is_adapter {
            true => Self::ADAPTER_OFFSET + 8,
            false => 8,
        };
        self.header
            .serialize(&mut (&mut buf[offset..] as &mut [u8]))
            .unwrap();
    }
}

/// Utility struct for iterating over a queue
pub struct QueueIterator<'a> {
    queue: EventQueue<'a>,
    current_index: usize,
    remaining: u64,
}

impl<'a> Iterator for QueueIterator<'a> {
    type Item = OrderbookEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let event_idx =
            (self.queue.header.head as usize + self.current_index) % self.queue.capacity;
        self.current_index += 1;
        self.remaining -= 1;
        Some(self.queue.get_event(event_idx))
    }
}
