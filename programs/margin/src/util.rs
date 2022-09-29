// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright (C) 2022 JET PROTOCOL HOLDINGS, LLC.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#[cfg(not(target_arch = "bpf"))]
use std::time::{SystemTime, UNIX_EPOCH};

use anchor_lang::{
    prelude::{msg, Clock, SolanaSysvar},
    solana_program::instruction::TRANSACTION_LEVEL_STACK_HEIGHT,
};
use bytemuck::{Pod, Zeroable};

use crate::{
    syscall::{sys, Sys},
    AccountPosition, ErrorCode,
};

/// Get the current timestamp in seconds since Unix epoch
///
/// The function returns a [anchor_lang::prelude::Clock] value in the bpf arch,
/// and first checks if there is a [Clock] in other archs, returning the system
/// time if there is no clock (e.g. if not running in a simulator with its clock).
pub fn get_timestamp() -> u64 {
    #[cfg(target_arch = "bpf")]
    {
        Clock::get().unwrap().unix_timestamp as u64
    }
    #[cfg(not(target_arch = "bpf"))]
    {
        // Get the clock in case it's available in a simulation,
        // then fall back to the system clock
        if let Ok(clock) = Clock::get() {
            clock.unix_timestamp as u64
        } else {
            let time = SystemTime::now();
            time.duration_since(UNIX_EPOCH).unwrap().as_secs()
        }
    }
}

pub trait Require<T> {
    fn require(self) -> std::result::Result<T, ErrorCode>;
    fn require_ref(&self) -> std::result::Result<&T, ErrorCode>;
    fn require_mut(&mut self) -> std::result::Result<&mut T, ErrorCode>;
}

impl<T: ErrorIfMissing> Require<T> for Option<T> {
    fn require(self) -> std::result::Result<T, ErrorCode> {
        self.ok_or(T::ERROR)
    }

    fn require_ref(&self) -> std::result::Result<&T, ErrorCode> {
        self.as_ref().ok_or(T::ERROR)
    }

    fn require_mut(&mut self) -> std::result::Result<&mut T, ErrorCode> {
        self.as_mut().ok_or(T::ERROR)
    }
}

pub trait ErrorIfMissing {
    const ERROR: ErrorCode;
}

impl ErrorIfMissing for &mut AccountPosition {
    const ERROR: ErrorCode = ErrorCode::PositionNotRegistered;
}

impl ErrorIfMissing for &AccountPosition {
    const ERROR: ErrorCode = ErrorCode::PositionNotRegistered;
}

macro_rules! log_on_error {
    ($result:expr, $($args:tt)*) => {{
        if $result.is_err() {
            msg!($($args)*);
        }
        $result
    }};
}
pub(crate) use log_on_error;

/// Data made available to invoked programs by the margin program. Put data here if:
/// - adapters need a guarantee that the margin program is the actual source of the data, or
/// - the data is needed by functions defined in margin that are called by adapters
/// Note: The security of the margin program cannot rely on function calls that happen within
/// adapters, because adapters can falsify the arguments to those functions.
/// Rather, this data should only be used to enable adapters to protect themselves, in which case
/// it would be in their best interest to pass along the actual state from the margin account.
#[derive(Pod, Zeroable, Copy, Clone, Debug, Default)]
#[repr(transparent)]
pub struct Invocation {
    /// The stack heights from where the margin program invoked an adapter.
    caller_heights: BitSet,
}

impl Invocation {
    /// Call this immediately before invoking another program to indicate that
    /// an invocation originated from the current stack height.
    pub(crate) fn start(&mut self) {
        self.caller_heights.insert(sys().get_stack_height() as u8);
    }

    /// Call this immediately after invoking another program to clear the
    /// indicator that an invocation originated from the current stack height.
    pub(crate) fn end(&mut self) {
        self.caller_heights.remove(sys().get_stack_height() as u8);
    }

    /// Returns ok if the current instruction was directly invoked by a cpi
    /// that marked the start.
    pub fn verify_directly_invoked(&self) -> Result<(), ErrorCode> {
        if !self.directly_invoked() {
            msg!(
                "Current stack height: {}. Invocations: {:?} (indexed from {})",
                sys().get_stack_height(),
                self,
                TRANSACTION_LEVEL_STACK_HEIGHT
            );
            return Err(ErrorCode::IndirectInvocation);
        }

        Ok(())
    }

    /// Returns true if the current instruction was directly invoked by a cpi
    /// that marked the start.
    pub fn directly_invoked(&self) -> bool {
        let height = sys().get_stack_height();
        height != 0 && self.caller_heights.contains(height as u8 - 1)
    }
}

#[derive(Pod, Zeroable, Copy, Clone, Default, PartialEq, Eq)]
#[repr(transparent)]
struct BitSet(u8);
impl BitSet {
    fn insert(&mut self, n: u8) {
        if n > 7 {
            panic!("attempted to set value outside bounds: {}", n);
        }
        self.0 |= 1 << n;
    }

    fn remove(&mut self, n: u8) {
        self.0 &= !(1 << n);
    }

    fn contains(&self, n: u8) -> bool {
        self.0 >> n & 1 == 1
    }
}

impl std::fmt::Debug for BitSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_tuple("BitSet")
            .field(&format_args!("{:#010b}", &self.0))
            .finish()
    }
}

#[cfg(test)]
mod test {
    /// potentially useful methods that are well tested.
    /// if useful, move to main impl. if annoying, delete
    impl BitSet {
        fn set(&mut self, n: u8, state: bool) {
            if state {
                self.insert(n)
            } else {
                self.remove(n)
            }
        }

        fn max(&self) -> Option<u32> {
            if self.0 == 0 {
                None
            } else {
                Some(7 - self.0.leading_zeros())
            }
        }

        fn min(&self) -> Option<u32> {
            if self.0 == 0 {
                None
            } else {
                Some(7 - self.0.trailing_zeros())
            }
        }
    }

    use anchor_lang::solana_program::instruction::TRANSACTION_LEVEL_STACK_HEIGHT;
    use itertools::Itertools;

    use crate::syscall::thread_local_mock::mock_stack_height;

    use super::*;

    const MAX_DEPTH: u8 = 5 + (TRANSACTION_LEVEL_STACK_HEIGHT as u8);

    #[test]
    fn never_report_if_none_marked() {
        let subject = Invocation::default();
        for i in 0..MAX_DEPTH {
            mock_stack_height(Some(i as usize));
            assert!(!subject.directly_invoked())
        }
    }

    /// Tests the typical case of margin at the top level
    #[test]
    fn happy_path() {
        let mut subject = Invocation::default();
        // mark start
        mock_stack_height(Some(1));
        subject.start();

        // actual invocation
        assert!(!subject.directly_invoked());
        mock_stack_height(Some(2));
        assert!(subject.directly_invoked());

        // too nested levels
        mock_stack_height(Some(3));
        assert!(!subject.directly_invoked());
        mock_stack_height(Some(4));
        assert!(!subject.directly_invoked());
        mock_stack_height(Some(5));
        assert!(!subject.directly_invoked());

        // same level as actual after done
        mock_stack_height(Some(1));
        subject.end();
        mock_stack_height(Some(2));
        assert!(!subject.directly_invoked());
    }

    /// Verify every scenario where margin invokes only once within the call stack
    /// This is redundant with check_all_heights_with_any_marks, but it has less risk
    /// of introducing bugs in the test code.
    #[test]
    fn check_all_heights_with_one_mark() {
        for mark_at in 0..MAX_DEPTH + 1 {
            let mut subject = Invocation::default();
            mock_stack_height(Some(mark_at as usize));
            subject.start();
            for check_at in 0..MAX_DEPTH + 1 {
                mock_stack_height(Some(check_at as usize));
                assert_eq!(
                    mark_at.checked_add(1).unwrap() == check_at,
                    subject.directly_invoked()
                )
            }
            mock_stack_height(Some(mark_at as usize));
            subject.end();
            for check_at in 0..MAX_DEPTH + 1 {
                mock_stack_height(Some(check_at as usize));
                assert!(!subject.directly_invoked());
            }
        }
    }

    /// Verify that directly_invoked returns the right value for every combination
    /// of invocations at every height before, during, and after the invocation
    #[test]
    fn check_all_heights_with_any_marks() {
        for size in 0..MAX_DEPTH + 2 {
            for combo in (0..MAX_DEPTH + 1).into_iter().combinations(size.into()) {
                let mut subject = Invocation::default();
                for depth in combo.clone() {
                    mock_stack_height(Some(depth as usize));
                    assert!(!subject.directly_invoked())
                }
                for depth in combo.clone() {
                    mock_stack_height(Some(depth as usize));
                    subject.start();
                }
                for depth in 0..MAX_DEPTH + 1 {
                    mock_stack_height(Some(depth as usize));
                    assert_eq!(
                        depth != 0 && combo.contains(&(depth - 1)),
                        subject.directly_invoked()
                    )
                }
                for depth in combo {
                    mock_stack_height(Some(depth as usize));
                    subject.end();
                    mock_stack_height(Some((depth + 1) as usize));
                    assert!(!subject.directly_invoked())
                }
                for depth in 0..MAX_DEPTH + 1 {
                    mock_stack_height(Some(depth as usize));
                    assert!(!subject.directly_invoked())
                }
            }
        }
    }

    #[test]
    fn bitset_insert() {
        bitset_manipulation(BitSet::insert, true);
    }

    #[test]
    fn bitset_remove() {
        bitset_manipulation(BitSet::remove, false);
    }

    /// For every possible initial state, `mutator` makes `contains` return
    /// `state` without changing the value for any other bit.
    fn bitset_manipulation(mutator: fn(&mut BitSet, u8) -> (), contains: bool) {
        for byte in 0..u8::MAX {
            for n in 0..8 {
                let mut ba = BitSet(byte);
                // `insert` or `remove` applies the desired state
                mutator(&mut ba, n);
                assert_eq!(contains, ba.contains(n));
                for i in 0..8u8 {
                    if i as u8 != n {
                        // other bits are unchanged
                        assert_eq!(BitSet(byte).contains(i), ba.contains(i));
                    }
                }
                ba.set(n, BitSet(byte).contains(n));
                // set restores the original bit
                assert_eq!(BitSet(byte), ba);
            }
        }
    }

    #[test]
    fn bitset_extrema() {
        assert_eq!(BitSet(0).max(), None);
        assert_eq!(BitSet(0).min(), None);
        for extremum in 0..8 {
            let top = 2u8.checked_pow(extremum + 1).unwrap_or(u8::MAX);
            for byte in 2u8.pow(extremum as u32)..top {
                assert_eq!(extremum as u32, BitSet(byte).max().unwrap());
                assert_eq!(extremum as u32, BitSet(byte.reverse_bits()).min().unwrap());
            }
        }
    }
}
