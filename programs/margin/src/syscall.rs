use anchor_lang::{
    prelude::{Clock, SolanaSysvar},
    solana_program::instruction,
};

#[inline]
#[cfg(all(target_arch = "bpf", not(test), not(mock_syscall)))]
pub fn sys() -> RealSys {
    RealSys
}

pub struct RealSys;
impl Sys for RealSys {}

pub trait Sys {
    #[inline]
    fn get_stack_height(&self) -> usize {
        instruction::get_stack_height()
    }

    /// Get the current timestamp in seconds since Unix epoch
    ///
    /// The function returns a [anchor_lang::prelude::Clock] value in the bpf arch,
    /// and first checks if there is a [Clock] in other archs, returning the system
    /// time if there is no clock (e.g. if not running in a simulator with its clock).
    #[inline]
    fn unix_timestamp(&self) -> u64 {
        Clock::get().unwrap().unix_timestamp as u64
    }
}

#[cfg(any(not(target_arch = "bpf"), test, mock_syscall))]
pub use thread_local_mock::sys;

#[cfg(any(not(target_arch = "bpf"), test, mock_syscall))]
pub mod thread_local_mock {
    use anchor_lang::prelude::SolanaSysvar;

    use super::*;
    use std::{
        cell::RefCell,
        rc::Rc,
        time::{SystemTime, UNIX_EPOCH},
    };

    pub fn sys() -> Rc<RefCell<TestSys>> {
        SYS.with(|t| t.clone())
    }

    pub fn mock_stack_height(height: Option<usize>) {
        sys().borrow_mut().mock_stack_height = height;
    }

    pub fn mock_clock(unix_timestamp: Option<u64>) {
        sys().borrow_mut().mock_clock = unix_timestamp;
    }

    thread_local! {
        pub static SYS: Rc<RefCell<TestSys>> = Rc::new(RefCell::new(TestSys::default()));
    }

    #[derive(Default)]
    pub struct TestSys {
        pub mock_stack_height: Option<usize>,
        pub mock_clock: Option<u64>,
    }

    impl Sys for Rc<RefCell<TestSys>> {
        fn get_stack_height(&self) -> usize {
            self.borrow()
                .mock_stack_height
                .unwrap_or_else(|| RealSys.get_stack_height())
        }

        fn unix_timestamp(&self) -> u64 {
            // The mocked clock gets top priority if set. Otherwise, actually
            // try to get the solana clock, in case it's available in a
            // simulation, then fall back to the system clock.
            if let Some(mocked) = self.borrow().mock_clock {
                mocked
            } else if let Ok(clock) = Clock::get() {
                clock.unix_timestamp as u64
            } else {
                let time = SystemTime::now();
                time.duration_since(UNIX_EPOCH).unwrap().as_secs()
            }
        }
    }
}
