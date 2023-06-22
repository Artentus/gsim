#![allow(dead_code)]

mod ops;
use ops::*;

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

/// An integer type of the same bit width as LogicState.
pub type LogicSizeInteger = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub(crate) struct LogicStorage(LogicSizeInteger);
assert_eq_size!(LogicStorage, LogicSizeInteger);
assert_eq_align!(LogicStorage, LogicSizeInteger);

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

impl BitAndAssign for LogicStorage {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for LogicStorage {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for LogicStorage {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor for LogicStorage {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for LogicStorage {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
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

impl Mul for LogicStorage {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0.wrapping_mul(rhs.0))
    }
}

impl MulAssign for LogicStorage {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_mul(rhs.0);
    }
}

impl Div for LogicStorage {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0.wrapping_div(rhs.0))
    }
}

impl DivAssign for LogicStorage {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_div(rhs.0);
    }
}

impl Rem for LogicStorage {
    type Output = Self;

    #[inline]
    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0.wrapping_rem(rhs.0))
    }
}

impl RemAssign for LogicStorage {
    #[inline]
    fn rem_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_rem(rhs.0);
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

/// Stores the logic state of up to `MAX_LOGIC_WIDTH` bits
#[derive(Debug, Clone, Copy)]
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
    pub const fn from_int(value: LogicSizeInteger) -> Self {
        Self {
            state: LogicStorage(value),
            valid: LogicStorage::ALL_ONE,
        }
    }

    /// Creates a new logic state representing the given boolean value
    #[inline]
    pub const fn from_bool(value: bool) -> Self {
        Self {
            state: LogicStorage(if value { 1 } else { 0 }),
            valid: LogicStorage::ALL_ONE,
        }
    }

    /// Creates a new logic state from the given bits (most significant bit first)
    pub const fn from_bits(bits: &[LogicBitState]) -> Self {
        assert!(bits.len() <= (MAX_LOGIC_WIDTH as usize));

        let mut state = 0;
        let mut valid = 0;

        // TODO: write this as a for loop once they become stable in const fns
        let mut i = 0;
        while i < bits.len() {
            state <<= 1;
            valid <<= 1;

            let (bit_state, bit_valid) = match bits[i] {
                LogicBitState::HighZ => (0, 0),
                LogicBitState::Undefined => (1, 0),
                LogicBitState::Logic0 => (0, 1),
                LogicBitState::Logic1 => (1, 1),
            };

            state |= bit_state;
            valid |= bit_valid;

            i += 1;
        }

        Self {
            state: LogicStorage(state),
            valid: LogicStorage(valid),
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
    pub fn eq(self, rhs: Self, width: LogicWidth) -> bool {
        let mask = LogicStorage::mask(width);
        ((self.state & mask) == (rhs.state & mask)) && ((self.valid & mask) == (rhs.valid & mask))
    }

    /// Computes logical AND between this state and `rhs`
    #[inline]
    pub fn logic_and(self, rhs: Self) -> Self {
        logic_and(self, rhs)
    }

    /// Computes logical OR between this state and `rhs`
    #[inline]
    pub fn logic_or(self, rhs: Self) -> Self {
        logic_or(self, rhs)
    }

    /// Computes logical XOR between this state and `rhs`
    #[inline]
    pub fn logic_xor(self, rhs: Self) -> Self {
        logic_xor(self, rhs)
    }

    /// Computes logical NAND between this state and `rhs`
    #[inline]
    pub fn logic_nand(self, rhs: Self) -> Self {
        logic_nand(self, rhs)
    }

    /// Computes logical NOR between this state and `rhs`
    #[inline]
    pub fn logic_nor(self, rhs: Self) -> Self {
        logic_nor(self, rhs)
    }

    /// Computes logical XNOR between this state and `rhs`
    #[inline]
    pub fn logic_xnor(self, rhs: Self) -> Self {
        logic_xnor(self, rhs)
    }

    /// Computes logical NOT of this state
    #[inline]
    pub fn logic_not(self) -> Self {
        logic_not(self)
    }

    /// Computes the sum of this state and `rhs`
    #[inline]
    pub fn add(self, rhs: Self, width: LogicWidth) -> Self {
        add(self, rhs, width)
    }

    /// Computes the difference between this state and `rhs`
    #[inline]
    pub fn sub(self, rhs: Self, width: LogicWidth) -> Self {
        sub(self, rhs, width)
    }

    /// Computes the product of this state and `rhs`
    #[inline]
    pub fn mul(self, rhs: Self, width: LogicWidth) -> Self {
        mul(self, rhs, width)
    }

    /// Computes the quotient of this state and `rhs`
    #[inline]
    pub fn div(self, rhs: Self, width: LogicWidth) -> Self {
        div(self, rhs, width)
    }

    /// Computes the remainder of the quotient of this state and `rhs`
    #[inline]
    pub fn rem(self, rhs: Self, width: LogicWidth) -> Self {
        rem(self, rhs, width)
    }

    /// Turns all HIGH Z bits into UNDEFINED bits
    #[inline]
    pub fn high_z_to_undefined(self) -> Self {
        Self {
            state: self.state | !self.valid,
            valid: self.valid,
        }
    }
}

/// Constructs a logic state from a list of bits (most significant bit first)
///
/// Example:
/// ```
/// # use gsim::{bits, LogicState, LogicWidth};
/// # fn foo() -> LogicState {
/// bits!(1, 0, X, Z) // Turns into state `10XZ`
/// # }
/// # assert_eq!(foo().display_string(LogicWidth::new(4).unwrap()), "10XZ");
/// ```
#[macro_export]
macro_rules! bits {
    (@BIT Z) => { $crate::LogicBitState::HighZ };
    (@BIT z) => { $crate::LogicBitState::HighZ };
    (@BIT X) => { $crate::LogicBitState::Undefined };
    (@BIT x) => { $crate::LogicBitState::Undefined };
    (@BIT 0) => { $crate::LogicBitState::Logic0 };
    (@BIT 1) => { $crate::LogicBitState::Logic1 };
    ($($bit:tt),*) => {{
        const STATE: $crate::LogicState = $crate::LogicState::from_bits(&[$($crate::bits!(@BIT $bit)),*]);
        STATE
    }}
}

pub use bits;

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
    pub(crate) unsafe fn set_unsafe(&self, value: LogicState) -> bool {
        let ptr = self.inner.get();
        if (value.state != (*ptr).state) || (value.valid != (*ptr).valid) {
            *ptr = value;
            true
        } else {
            false
        }
    }

    #[inline]
    pub(crate) fn get_mut(&mut self) -> &mut LogicState {
        self.inner.get_mut()
    }
}

unsafe impl Sync for LogicStateCell {}
