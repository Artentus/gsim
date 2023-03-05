#![allow(dead_code)]

use std::cell::UnsafeCell;
use std::ops::*;

macro_rules! size_of {
    ($t:ty) => {
        std::mem::size_of::<$t>()
    };
}

macro_rules! bit_size_of {
    ($t:ty) => {
        size_of!($t) * 8
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub(crate) struct LogicStorage(u64);
assert_eq_size!(LogicStorage, u64);
assert_eq_align!(LogicStorage, u64);

impl LogicStorage {
    pub(crate) const ALL_ZERO: Self = Self(0);
    pub(crate) const ALL_ONE: Self = Self(!0);

    #[inline]
    pub(crate) fn mask(width: LogicWidth) -> Self {
        if width >= LogicWidth::MAX {
            Self::ALL_ONE
        } else {
            Self((1 << width.get()) - 1)
        }
    }

    #[inline]
    fn get_bit(&self, bit_index: LogicOffset) -> bool {
        ((self.0 >> bit_index.get()) & 0x1) != 0
    }
}

impl BitAnd for LogicStorage {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for LogicStorage {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitXor for LogicStorage {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl Not for LogicStorage {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Add for LogicStorage {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.wrapping_add(rhs.0))
    }
}

impl AddAssign for LogicStorage {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_add(rhs.0);
    }
}

impl Sub for LogicStorage {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.wrapping_sub(rhs.0))
    }
}

impl SubAssign for LogicStorage {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_sub(rhs.0);
    }
}

impl Neg for LogicStorage {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self(Self::ALL_ZERO.0.wrapping_sub(self.0))
    }
}

impl Shl<LogicOffset> for LogicStorage {
    type Output = Self;

    #[inline]
    fn shl(self, rhs: LogicOffset) -> Self::Output {
        Self(self.0 << rhs.get())
    }
}

impl ShlAssign<LogicOffset> for LogicStorage {
    #[inline]
    fn shl_assign(&mut self, rhs: LogicOffset) {
        self.0 <<= rhs.get();
    }
}

impl Shr<LogicOffset> for LogicStorage {
    type Output = Self;

    #[inline]
    fn shr(self, rhs: LogicOffset) -> Self::Output {
        Self(self.0 >> rhs.get())
    }
}

impl ShrAssign<LogicOffset> for LogicStorage {
    #[inline]
    fn shr_assign(&mut self, rhs: LogicOffset) {
        self.0 >>= rhs.get();
    }
}

const MAX_LOGIC_WIDTH: u8 = {
    const MAX_LOGIC_WIDTH: usize = bit_size_of!(LogicStorage);
    const_assert!(MAX_LOGIC_WIDTH <= (u8::MAX as usize));
    MAX_LOGIC_WIDTH as u8
};

/// The width in bits of a logic state, in the range 1 to 64 inclusive
pub type LogicWidth = bounded_integer::BoundedU8<1, MAX_LOGIC_WIDTH>;
assert_eq_size!(LogicWidth, u8);
assert_eq_align!(LogicWidth, u8);

const MAX_LOGIC_OFFSET: u8 = {
    const MAX_LOGIC_OFFSET: usize = bit_size_of!(LogicStorage) - 1;
    const_assert!(MAX_LOGIC_OFFSET < (u8::MAX as usize));
    MAX_LOGIC_OFFSET as u8
};

/// A bit offset in a logic state, in the range 0 to 63 inclusive
pub type LogicOffset = bounded_integer::BoundedU8<0, MAX_LOGIC_OFFSET>;
assert_eq_size!(LogicOffset, u8);
assert_eq_align!(LogicOffset, u8);

#[inline]
pub(crate) const fn width_to_offset(width: LogicWidth) -> Option<LogicOffset> {
    LogicOffset::new(width.get())
}

/// The logic state of a single bit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicBitState {
    /// The high impedance state
    HighZ,
    /// An undefined logic level
    Undefined,
    /// The low logic level
    Logic0,
    /// The high logic level
    Logic1,
}

impl LogicBitState {
    #[inline]
    fn from_bits(state_bit: bool, valid_bit: bool) -> Self {
        match (state_bit, valid_bit) {
            (false, false) => Self::HighZ,
            (true, false) => Self::Undefined,
            (false, true) => Self::Logic0,
            (true, true) => Self::Logic1,
        }
    }

    /// A character representing this logic state
    /// - `HighZ`: `'Z'`
    /// - `Undefined`: `'X'`
    /// - `Logic0`: `'0'`
    /// - `Logic1`: `'1'`
    #[inline]
    pub fn display_char(self) -> char {
        match self {
            LogicBitState::HighZ => 'Z',
            LogicBitState::Undefined => 'X',
            LogicBitState::Logic0 => '0',
            LogicBitState::Logic1 => '1',
        }
    }
}

/// Stores the logic state of up to 64 bits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct LogicState {
    //  state | valid | meaning
    // -------|-------|---------
    //    0   |   0   | High-Z
    //    1   |   0   | Undefined
    //    0   |   1   | Logic 0
    //    1   |   1   | Logic 1
    pub(crate) state: LogicStorage,
    pub(crate) valid: LogicStorage,
}

impl LogicState {
    /// A logic state representing high impedance on all bits
    pub const HIGH_Z: Self = Self {
        state: LogicStorage::ALL_ZERO,
        valid: LogicStorage::ALL_ZERO,
    };

    /// A logic state representing an undefined logic level on all bits
    pub const UNDEFINED: Self = Self {
        state: LogicStorage::ALL_ONE,
        valid: LogicStorage::ALL_ZERO,
    };

    /// A logic state representing a low logic level on all bits
    pub const LOGIC_0: Self = Self {
        state: LogicStorage::ALL_ZERO,
        valid: LogicStorage::ALL_ONE,
    };

    /// A logic state representing a high logic level on all bits
    pub const LOGIC_1: Self = Self {
        state: LogicStorage::ALL_ONE,
        valid: LogicStorage::ALL_ONE,
    };

    /// Creates a new logic state representing the given integer value
    #[inline]
    pub const fn new(value: u64) -> Self {
        Self {
            state: LogicStorage(value),
            valid: LogicStorage::ALL_ONE,
        }
    }

    /// Gets the logic state of a single bit
    pub fn get_bit_state(&self, bit_index: LogicOffset) -> LogicBitState {
        let state_bit = self.state.get_bit(bit_index);
        let valid_bit = self.valid.get_bit(bit_index);
        LogicBitState::from_bits(state_bit, valid_bit)
    }

    /// Creates a string representing the first `width` bits of this state
    pub fn display_string(&self, width: LogicWidth) -> String {
        (0..width.get())
            .rev()
            .map(|bit_index| {
                let bit_index = LogicOffset::new(bit_index).expect("invalid bit index");
                self.get_bit_state(bit_index).display_char()
            })
            .collect()
    }

    /// Tests this state for equality with another state while only considering the first `width` bits
    pub fn eq_width(&self, rhs: &Self, width: LogicWidth) -> bool {
        let mask = LogicStorage::mask(width);
        ((self.state & mask) == (rhs.state & mask)) && ((self.valid & mask) == (rhs.valid & mask))
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub(crate) struct LogicStateCell {
    inner: UnsafeCell<LogicState>,
}

impl LogicStateCell {
    #[inline]
    pub(crate) const fn new(value: LogicState) -> Self {
        Self {
            inner: UnsafeCell::new(value),
        }
    }

    #[inline]
    pub(crate) fn get(&self) -> LogicState {
        unsafe { *self.inner.get() }
    }

    #[inline]
    pub(crate) unsafe fn get_mut_unsafe(&self) -> &mut LogicState {
        &mut *self.inner.get()
    }

    #[inline]
    pub(crate) fn get_mut(&mut self) -> &mut LogicState {
        self.inner.get_mut()
    }
}

unsafe impl Sync for LogicStateCell {}
