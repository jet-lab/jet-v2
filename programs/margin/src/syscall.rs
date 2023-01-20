use anchor_lang::{
    prelude::{Clock, SolanaSysvar},
    solana_program::instruction,
};

#[inline]
#[cfg(not(feature = "mock_syscall"))]
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

#[cfg(feature = "mock_syscall")]
pub use thread_local_mock::test_sys as sys;

#[cfg(feature = "mock_syscall")]
pub mod thread_local_mock {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    pub enum SyscallProvider<T> {
        /// Return the provided value.
        Mock(Box<dyn Fn() -> T>),
        /// Attempts to query the solana runtime, as if there were no mock.
        SolanaRuntime,
        /// Guarantees that the syscall can complete withour error even if the
        /// solana runtime is unavailable. You cannot rely on a consistent or
        /// correct value being returned, only that the syscall itself will not
        /// fail.
        Stub,
    }

    impl<T> Default for SyscallProvider<T> {
        /// This makes it so that the compiler feature is additive, in the sense
        /// that enabling this feature will not alter the behavior of any code
        /// that can be compiled without this feature being enabled (aside from
        /// the performance cost of dynamic dispatch). You will only see changes
        /// in behavior when you actually use the code that is added by the
        /// feature.
        /// https://doc.rust-lang.org/cargo/reference/features.html?highlight=additive#feature-unification
        fn default() -> Self {
            Self::SolanaRuntime
        }
    }

    pub fn test_sys() -> Rc<RefCell<TestSys>> {
        SYS.with(|t| t.clone())
    }

    /// Disables all mocks/stubs and switches everything back to using the
    /// solana runtime.
    pub fn restore() {
        *test_sys().borrow_mut() = TestSys {
            stack_height: SyscallProvider::SolanaRuntime,
            clock: SyscallProvider::SolanaRuntime,
        };
    }

    /// Stubs out all syscalls with a meaningless but reliable return.
    pub fn stub() {
        *test_sys().borrow_mut() = TestSys {
            stack_height: SyscallProvider::Stub,
            clock: SyscallProvider::Stub,
        }
    }

    /// Mocks syscalls so they produce values by executing the expression.
    #[macro_export]
    macro_rules! mock_sys {
        ($($name:ident = $evaluator:expr);+$(;)?) => {{
            use $crate::syscall::thread_local_mock::*;
            $(test_sys().borrow_mut().$name = SyscallProvider::Mock(Box::new(move || $evaluator));)+
        }};
    }

    thread_local! {
        pub static SYS: Rc<RefCell<TestSys>> = Rc::new(RefCell::new(TestSys::default()));
    }

    #[derive(Default)]
    pub struct TestSys {
        pub stack_height: SyscallProvider<usize>,
        pub clock: SyscallProvider<u64>,
    }

    impl Sys for Rc<RefCell<TestSys>> {
        fn get_stack_height(&self) -> usize {
            match &self.borrow().stack_height {
                SyscallProvider::Mock(height) => height(),
                SyscallProvider::SolanaRuntime => RealSys.get_stack_height(),
                SyscallProvider::Stub => 0,
            }
        }

        fn unix_timestamp(&self) -> u64 {
            match &self.borrow().clock {
                SyscallProvider::Mock(time) => time(),
                SyscallProvider::SolanaRuntime => RealSys.unix_timestamp(),
                SyscallProvider::Stub => 1_600_000_000,
            }
        }
    }
}
