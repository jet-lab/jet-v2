use anchor_lang::solana_program::instruction;

#[inline]
#[cfg(not(test))]
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
}

#[cfg(test)]
pub use thread_local_mock::sys;

#[cfg(test)]
pub mod thread_local_mock {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    pub fn sys() -> Rc<RefCell<TestSys>> {
        SYS.with(|t| t.clone())
    }

    pub fn mock_stack_height(height: Option<usize>) {
        sys().borrow_mut().mock_stack_height = height;
    }

    thread_local! {
        pub static SYS: Rc<RefCell<TestSys>> = Rc::new(RefCell::new(TestSys::default()));
    }

    #[derive(Default)]
    pub struct TestSys {
        pub mock_stack_height: Option<usize>,
    }

    impl Sys for Rc<RefCell<TestSys>> {
        fn get_stack_height(&self) -> usize {
            self.borrow()
                .mock_stack_height
                .unwrap_or_else(|| RealSys.get_stack_height())
        }
    }
}
