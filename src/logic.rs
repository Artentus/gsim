#![allow(dead_code)]

use crate::{inline_vec, SafeDivCeil};
use std::num::NonZeroU8;
use std::ops::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct LogicStorage(u32);
assert_eq_size!(LogicStorage, u32);
assert_eq_align!(LogicStorage, u32);

impl LogicStorage {
    pub(crate) const ALL_ZERO: Self = Self(0);
    pub(crate) const ALL_ONE: Self = Self(!0);

    #[inline]
    pub(crate) const fn new(value: u32) -> Self {
        Self(value)
    }

    #[inline]
    pub(crate) const fn mask(width: AtomWidth) -> Self {
        Self(((1u64 << width.get()) - 1) as u32)
    }

    #[inline]
    pub(crate) fn get_bit(&self, bit_index: AtomOffset) -> bool {
        ((self.0 >> bit_index.get()) & 0x1) != 0
    }

    #[inline]
    pub(crate) const fn get(&self) -> u32 {
        self.0
    }
}

const_assert_eq!(
    LogicStorage::mask(AtomWidth::MAX).0,
    LogicStorage::ALL_ONE.0
);

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

impl Shl<AtomOffset> for LogicStorage {
    type Output = Self;

    #[inline]
    fn shl(self, rhs: AtomOffset) -> Self::Output {
        Self(self.0 << rhs.get())
    }
}

impl ShlAssign<AtomOffset> for LogicStorage {
    #[inline]
    fn shl_assign(&mut self, rhs: AtomOffset) {
        self.0 <<= rhs.get();
    }
}

impl Shr<AtomOffset> for LogicStorage {
    type Output = Self;

    #[inline]
    fn shr(self, rhs: AtomOffset) -> Self::Output {
        Self(self.0 >> rhs.get())
    }
}

impl ShrAssign<AtomOffset> for LogicStorage {
    #[inline]
    fn shr_assign(&mut self, rhs: AtomOffset) {
        self.0 >>= rhs.get();
    }
}

impl LogicStorage {
    #[inline]
    pub(crate) fn ashr(self, rhs: AtomOffset) -> Self {
        Self(((self.0 as i32) >> rhs.get()) as u32)
    }

    #[inline]
    pub(crate) fn widening_shl(self, shamnt: AtomOffset) -> (Self, Self) {
        let full = (self.0 as u64) << shamnt.get();
        let low = full as u32;
        let high = (full >> 32) as u32;
        (Self(low), Self(high))
    }

    #[inline]
    pub(crate) fn widening_shr(self, shamnt: AtomOffset) -> (Self, Self) {
        let full = ((self.0 as u64) << 32) >> shamnt.get();
        let high = (full >> 32) as u32;
        let low = full as u32;
        (Self(high), Self(low))
    }

    #[inline]
    pub(crate) fn carrying_add(self, rhs: Self, c_in: bool) -> (Self, bool) {
        let (r1, c1) = self.0.overflowing_add(rhs.0);
        let (r2, c2) = r1.overflowing_add(c_in as u32);
        (Self(r2), c1 | c2)
    }

    #[inline]
    pub(crate) fn widening_mul(self, rhs: Self, width: AtomWidth) -> (Self, Self) {
        let result = (self.0 as u64) * (rhs.0 as u64);
        (Self(result as u32), Self((result >> width.get()) as u32))
    }

    #[inline]
    pub(crate) fn lts(self, rhs: Self, width: AtomWidth) -> bool {
        let lhs = self.0 as i32;
        let rhs = rhs.0 as i32;
        let shift_amount = Atom::BITS.get() - width.get();
        (lhs << shift_amount) < (rhs << shift_amount)
    }

    #[inline]
    pub(crate) fn gts(self, rhs: Self, width: AtomWidth) -> bool {
        let lhs = self.0 as i32;
        let rhs = rhs.0 as i32;
        let shift_amount = Atom::BITS.get() - width.get();
        (lhs << shift_amount) > (rhs << shift_amount)
    }

    #[inline]
    pub(crate) fn les(self, rhs: Self, width: AtomWidth) -> bool {
        let lhs = self.0 as i32;
        let rhs = rhs.0 as i32;
        let shift_amount = Atom::BITS.get() - width.get();
        (lhs << shift_amount) <= (rhs << shift_amount)
    }

    #[inline]
    pub(crate) fn ges(self, rhs: Self, width: AtomWidth) -> bool {
        let lhs = self.0 as i32;
        let rhs = rhs.0 as i32;
        let shift_amount = Atom::BITS.get() - width.get();
        (lhs << shift_amount) >= (rhs << shift_amount)
    }

    #[inline]
    pub(crate) fn keep_trailing_ones(self) -> Self {
        Self(((1u64 << self.0.trailing_ones()) - 1) as u32)
    }
}

/// The logic state of a single bit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LogicBitState {
    /// The high impedance state
    HighZ = 0b00,
    /// An undefined logic level
    Undefined = 0b01,
    /// The low logic level
    Logic0 = 0b10,
    /// The high logic level
    Logic1 = 0b11,
}

impl LogicBitState {
    #[inline]
    pub(crate) const fn from_bits(state_bit: bool, valid_bit: bool) -> Self {
        match (state_bit, valid_bit) {
            (false, false) => Self::HighZ,
            (true, false) => Self::Undefined,
            (false, true) => Self::Logic0,
            (true, true) => Self::Logic1,
        }
    }

    #[inline]
    pub(crate) const fn to_bits(self) -> (bool, bool) {
        let state = (self as u8) & 0x1;
        let valid = (self as u8) >> 1;
        (state > 0, valid > 0)
    }

    /// Creates a logic bit state representing a boolean value
    #[inline]
    pub const fn from_bool(value: bool) -> Self {
        match value {
            false => Self::Logic0,
            true => Self::Logic1,
        }
    }

    /// The boolean value this logic bit state represents, if any
    #[inline]
    pub const fn to_bool(self) -> Option<bool> {
        match self {
            Self::HighZ | Self::Undefined => None,
            Self::Logic0 => Some(false),
            Self::Logic1 => Some(true),
        }
    }

    #[inline]
    pub(crate) const fn splat(self) -> Atom {
        match self {
            Self::HighZ => Atom::HIGH_Z,
            Self::Undefined => Atom::UNDEFINED,
            Self::Logic0 => Atom::LOGIC_0,
            Self::Logic1 => Atom::LOGIC_1,
        }
    }

    /// - `b'Z'` | `b'z'` => `HighZ`
    /// - `b'X'` | `b'x'` => `Undefined`
    /// - `b'0'` => `Logic0`
    /// - `b'1'` => `Logic1`
    #[inline]
    pub const fn parse_byte(c: u8) -> Option<Self> {
        match c {
            b'Z' | b'z' => Some(Self::HighZ),
            b'X' | b'x' => Some(Self::Undefined),
            b'0' => Some(Self::Logic0),
            b'1' => Some(Self::Logic1),
            _ => None,
        }
    }

    /// - `'Z'` | `'z'` => `HighZ`
    /// - `'X'` | `'x'` => `Undefined`
    /// - `'0'` => `Logic0`
    /// - `'1'` => `Logic1`
    #[inline]
    pub const fn parse(c: char) -> Option<Self> {
        if c.is_ascii() {
            Self::parse_byte(c as u8)
        } else {
            None
        }
    }
}

impl From<bool> for LogicBitState {
    #[inline]
    fn from(value: bool) -> Self {
        Self::from_bool(value)
    }
}

impl TryFrom<LogicBitState> for bool {
    type Error = ();

    #[inline]
    fn try_from(value: LogicBitState) -> Result<Self, Self::Error> {
        value.to_bool().ok_or(())
    }
}

impl std::fmt::Display for LogicBitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LogicBitState::HighZ => "Z",
            LogicBitState::Undefined => "X",
            LogicBitState::Logic0 => "0",
            LogicBitState::Logic1 => "1",
        };

        f.write_str(s)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for LogicBitState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::*;

        struct LogicBitStateVisitor;

        impl Visitor<'_> for LogicBitStateVisitor {
            type Value = LogicBitState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("one of the chars ['Z', 'z', 'X', 'x', '0', '1']")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if v.len() == 1 {
                    LogicBitState::parse_byte(v.as_bytes()[0])
                } else {
                    None
                }
                .ok_or_else(|| E::invalid_value(Unexpected::Str(v), &self))
            }
        }

        deserializer.deserialize_str(LogicBitStateVisitor)
    }
}

const MAX_ATOM_WIDTH: u8 = Atom::BITS.get();
pub(crate) type AtomWidth = bounded_integer::BoundedU8<1, MAX_ATOM_WIDTH>;
assert_eq_size!(AtomWidth, u8);
assert_eq_align!(AtomWidth, u8);

const MAX_ATOM_OFFSET: u8 = Atom::BITS.get() - 1;
pub(crate) type AtomOffset = bounded_integer::BoundedU8<0, MAX_ATOM_OFFSET>;
assert_eq_size!(AtomOffset, u8);
assert_eq_align!(AtomOffset, u8);

/// The smallest unit of logic state in the simulator
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct Atom {
    //  state | valid | meaning
    // -------|-------|---------
    //    0   |   0   | High-Z
    //    1   |   0   | Undefined
    //    0   |   1   | Logic 0
    //    1   |   1   | Logic 1
    pub(crate) state: LogicStorage,
    pub(crate) valid: LogicStorage,
}

const_assert_eq!(
    std::mem::size_of::<Atom>() * 8,
    (Atom::BITS.get() as usize) * 2,
);

impl Atom {
    pub(crate) const BITS: NonZeroU8 = unsafe { NonZeroU8::new_unchecked(32) };

    pub(crate) const HIGH_Z: Self = Self {
        state: LogicStorage::ALL_ZERO,
        valid: LogicStorage::ALL_ZERO,
    };

    pub(crate) const UNDEFINED: Self = Self {
        state: LogicStorage::ALL_ONE,
        valid: LogicStorage::ALL_ZERO,
    };

    pub(crate) const LOGIC_0: Self = Self {
        state: LogicStorage::ALL_ZERO,
        valid: LogicStorage::ALL_ONE,
    };

    pub(crate) const LOGIC_1: Self = Self {
        state: LogicStorage::ALL_ONE,
        valid: LogicStorage::ALL_ONE,
    };

    #[inline]
    pub(crate) const fn from_int(value: u32) -> Self {
        Self {
            state: LogicStorage(value),
            valid: LogicStorage::ALL_ONE,
        }
    }

    #[inline]
    pub(crate) const fn from_bool(value: bool) -> Self {
        Self {
            state: LogicStorage(value as u32),
            valid: LogicStorage::ALL_ONE,
        }
    }

    #[inline]
    pub(crate) const fn from_bit(value: LogicBitState) -> Self {
        let (state, valid) = value.to_bits();

        Self {
            state: LogicStorage(state as u32),
            valid: LogicStorage(valid as u32),
        }
    }

    fn from_bits(bits: &[LogicBitState]) -> Self {
        debug_assert!(bits.len() > 0);
        debug_assert!(bits.len() <= (Self::BITS.get() as usize));

        let mut state = 0;
        let mut valid = 0;

        for i in 0..bits.len() {
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
        }

        Self {
            state: LogicStorage(state),
            valid: LogicStorage(valid),
        }
    }

    fn parse(s: &[u8]) -> Option<Self> {
        debug_assert!(s.len() > 0);
        debug_assert!(s.len() <= (Self::BITS.get() as usize));

        let mut state = 0;
        let mut valid = 0;

        for &c in s {
            state <<= 1;
            valid <<= 1;

            let bit = LogicBitState::parse_byte(c)?;
            let (bit_state, bit_valid) = match bit {
                LogicBitState::HighZ => (0, 0),
                LogicBitState::Undefined => (1, 0),
                LogicBitState::Logic0 => (0, 1),
                LogicBitState::Logic1 => (1, 1),
            };

            state |= bit_state;
            valid |= bit_valid;
        }

        Some(Self {
            state: LogicStorage(state),
            valid: LogicStorage(valid),
        })
    }

    #[inline]
    pub(crate) fn is_valid(&self, width: AtomWidth) -> bool {
        let mask = LogicStorage::mask(width);
        (self.valid | !mask) == LogicStorage::ALL_ONE
    }

    #[inline]
    pub(crate) fn get_bit_state(&self, bit_index: AtomOffset) -> LogicBitState {
        let state_bit = self.state.get_bit(bit_index);
        let valid_bit = self.valid.get_bit(bit_index);
        LogicBitState::from_bits(state_bit, valid_bit)
    }

    #[inline]
    pub(crate) const fn from_state_valid(state: u32, valid: u32) -> Self {
        Self {
            state: LogicStorage(state),
            valid: LogicStorage(valid),
        }
    }

    #[inline]
    pub(crate) const fn to_state_valid(self) -> (u32, u32) {
        (self.state.0, self.valid.0)
    }

    #[inline]
    pub(crate) fn eq(self, other: Self, width: AtomWidth) -> bool {
        let mask = LogicStorage::mask(width);
        ((self.state & mask) == (other.state & mask))
            && ((self.valid & mask) == (other.valid & mask))
    }

    #[inline]
    pub(crate) fn high_z_to_undefined(self) -> Self {
        Self {
            state: self.state | !self.valid,
            valid: self.valid,
        }
    }
}

impl std::fmt::Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in (0..Self::BITS.get()).rev() {
            let i = AtomOffset::new(i).unwrap();
            let bit = self.get_bit_state(i);
            write!(f, "{bit}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) enum LogicStateRepr {
    HighZ,
    Undefined,
    Logic0,
    Logic1,
    Int(LogicStorage),
    Bits(inline_vec!(Atom)),
}

/// A logic state of arbitrary bit width
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct LogicState(pub(crate) LogicStateRepr);

impl LogicState {
    /// A logic state representing high impedance on all bits
    pub const HIGH_Z: Self = Self(LogicStateRepr::HighZ);

    /// A logic state representing an undefined logic level on all bits
    pub const UNDEFINED: Self = Self(LogicStateRepr::Undefined);

    /// A logic state representing a low logic level on all bits
    pub const LOGIC_0: Self = Self(LogicStateRepr::Logic0);

    /// A logic state representing a high logic level on all bits
    pub const LOGIC_1: Self = Self(LogicStateRepr::Logic1);

    /// Creates a new logic state representing the given integer value
    ///
    /// Bits past the first 32 are implicitely assigned the value 0
    #[inline]
    pub const fn from_int(value: u32) -> Self {
        Self(LogicStateRepr::Int(LogicStorage(value)))
    }

    /// Creates a new logic state representing the given boolean value
    ///
    /// Bits past the first one are implicitely assigned the value 0
    #[inline]
    pub const fn from_bool(value: bool) -> Self {
        Self(LogicStateRepr::Int(LogicStorage(value as u32)))
    }

    /// Creates a new logic state from the given bits (most significant bit first)
    ///
    /// Bits past the specified ones are implicitely assigned the value Z
    pub fn from_bits(bits: &[LogicBitState]) -> Self {
        let width = u8::try_from(bits.len())
            .ok()
            .and_then(NonZeroU8::new)
            .expect("invalid bit width");

        let head_width = (width.get() % Atom::BITS.get()) as usize;
        let list_len = (width.get().div_ceil(Atom::BITS.get())) as usize;

        let mut list = smallvec::smallvec![Atom::HIGH_Z; list_len];
        let mut i = list_len;

        if head_width > 0 {
            let head_bits = &bits[..head_width];
            let head = Atom::from_bits(head_bits);

            i -= 1;
            list[i] = head;
        }

        let tail_bits = &bits[head_width..];
        let mut chunks = tail_bits.chunks_exact(Atom::BITS.get() as usize);
        for item_bits in chunks.by_ref() {
            let item = Atom::from_bits(item_bits);

            i -= 1;
            list[i] = item;
        }
        debug_assert_eq!(chunks.remainder().len(), 0);

        Self(LogicStateRepr::Bits(list))
    }

    /// Constructs a logic state from a string of bits (most significant bit first)
    ///
    /// Bits past the specified ones are implicitely assigned the value Z
    ///
    /// Example:
    /// ```
    /// use gsim::LogicState;
    /// use std::num::NonZeroU8;
    ///
    /// let state = LogicState::parse("10XZ").unwrap();
    /// assert_eq!(state.display_string(NonZeroU8::new(4).unwrap()), "10XZ");
    /// ```
    pub fn parse(s: &str) -> Option<Self> {
        let width = u8::try_from(s.len())
            .ok()
            .and_then(NonZeroU8::new)
            .expect("invalid bit width");

        if !s.is_ascii() {
            return None;
        }

        let head_width = (width.get() % Atom::BITS.get()) as usize;
        let list_len = (width.get().div_ceil(Atom::BITS.get())) as usize;

        let mut list = smallvec::smallvec![Atom::HIGH_Z; list_len];
        let mut i = list_len;

        if head_width > 0 {
            let head_str = &s.as_bytes()[..head_width];
            let head = Atom::parse(head_str)?;

            i -= 1;
            list[i] = head;
        }

        let tail_str = &s.as_bytes()[head_width..];
        let mut chunks = tail_str.chunks_exact(Atom::BITS.get() as usize);
        for item_bits in chunks.by_ref() {
            let item = Atom::parse(item_bits)?;

            i -= 1;
            list[i] = item;
        }
        debug_assert_eq!(chunks.remainder().len(), 0);

        Some(Self(LogicStateRepr::Bits(list)))
    }

    /// Gets the logic state of a single bit
    pub fn get_bit_state(&self, bit_index: u8) -> LogicBitState {
        match self.0 {
            LogicStateRepr::HighZ => LogicBitState::HighZ,
            LogicStateRepr::Undefined => LogicBitState::Undefined,
            LogicStateRepr::Logic0 => LogicBitState::Logic0,
            LogicStateRepr::Logic1 => LogicBitState::Logic1,
            LogicStateRepr::Int(value) => {
                if let Some(bit_index) = u8::try_from(bit_index).ok().and_then(AtomOffset::new) {
                    let state_bit = value.get_bit(bit_index);
                    LogicBitState::from_bits(state_bit, true)
                } else {
                    LogicBitState::Logic0
                }
            }
            LogicStateRepr::Bits(ref list) => {
                let item_index = (bit_index / Atom::BITS.get()) as usize;
                let bit_index = AtomOffset::new(bit_index % Atom::BITS.get()).unwrap();

                let item = list[item_index];
                item.get_bit_state(bit_index)
            }
        }
    }

    /// Creates a string representing the first `width` bits of this state
    pub fn display_string(&self, width: NonZeroU8) -> String {
        use std::fmt::Write;

        let mut s = String::with_capacity(width.get() as usize);
        for i in (0..width.get()).rev() {
            let bit = self.get_bit_state(i);
            write!(s, "{bit}").unwrap();
        }
        s
    }

    #[inline]
    pub(crate) fn iter_atoms(&self) -> LogicStateIter<'_> {
        match self.0 {
            LogicStateRepr::HighZ => LogicStateIter::HighZ,
            LogicStateRepr::Undefined => LogicStateIter::Undefined,
            LogicStateRepr::Logic0 => LogicStateIter::Logic0,
            LogicStateRepr::Logic1 => LogicStateIter::Logic1,
            LogicStateRepr::Int(value) => LogicStateIter::Int(Some(value)),
            LogicStateRepr::Bits(ref atoms) => LogicStateIter::Bits { atoms, current: 0 },
        }
    }

    /// Tests the first `width` bits of this state and another for equality
    pub fn eq(&self, other: &Self, width: NonZeroU8) -> bool {
        match (&self.0, &other.0) {
            (LogicStateRepr::HighZ, LogicStateRepr::HighZ)
            | (LogicStateRepr::Undefined, LogicStateRepr::Undefined)
            | (LogicStateRepr::Logic0, LogicStateRepr::Logic0)
            | (LogicStateRepr::Logic1, LogicStateRepr::Logic1) => return true,
            (LogicStateRepr::HighZ, LogicStateRepr::Undefined)
            | (LogicStateRepr::HighZ, LogicStateRepr::Logic0)
            | (LogicStateRepr::HighZ, LogicStateRepr::Logic1)
            | (LogicStateRepr::Undefined, LogicStateRepr::HighZ)
            | (LogicStateRepr::Undefined, LogicStateRepr::Logic0)
            | (LogicStateRepr::Undefined, LogicStateRepr::Logic1)
            | (LogicStateRepr::Logic0, LogicStateRepr::HighZ)
            | (LogicStateRepr::Logic0, LogicStateRepr::Undefined)
            | (LogicStateRepr::Logic0, LogicStateRepr::Logic1)
            | (LogicStateRepr::Logic1, LogicStateRepr::HighZ)
            | (LogicStateRepr::Logic1, LogicStateRepr::Undefined)
            | (LogicStateRepr::Logic1, LogicStateRepr::Logic0) => return false,
            _ => (),
        }

        let count = width.safe_div_ceil(Atom::BITS).get() as usize;
        let mut total_width = width.get();
        for (this, other) in self.iter_atoms().zip(other.iter_atoms()).take(count) {
            let width = AtomWidth::new(total_width).unwrap_or(AtomWidth::MAX);
            total_width -= width.get();

            if !this.eq(other, width) {
                return false;
            }
        }

        true
    }
}

pub(crate) enum LogicStateIter<'a> {
    HighZ,
    Undefined,
    Logic0,
    Logic1,
    Int(Option<LogicStorage>),
    Bits { atoms: &'a [Atom], current: usize },
}

impl Iterator for LogicStateIter<'_> {
    type Item = Atom;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            LogicStateIter::HighZ => Some(Atom::HIGH_Z),
            LogicStateIter::Undefined => Some(Atom::UNDEFINED),
            LogicStateIter::Logic0 => Some(Atom::LOGIC_0),
            LogicStateIter::Logic1 => Some(Atom::LOGIC_1),
            LogicStateIter::Int(value) => Some(Atom {
                state: value.take().unwrap_or(LogicStorage::ALL_ZERO),
                valid: LogicStorage::ALL_ONE,
            }),
            LogicStateIter::Bits { atoms, current } => {
                let atom = atoms.get(*current).copied();
                if atom.is_some() {
                    *current += 1;
                }
                atom
            }
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for LogicState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::*;

        struct LogicStateVisitor;

        impl Visitor<'_> for LogicStateVisitor {
            type Value = LogicState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string consisting of only the chars ['Z', 'z', 'X', 'x', '0', '1'] and length 1 to 32")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                LogicState::parse(v).ok_or_else(|| E::invalid_value(Unexpected::Str(v), &self))
            }
        }

        deserializer.deserialize_str(LogicStateVisitor)
    }
}

/// Constructs a logic state from a list of bits (most significant bit first)
///
/// Example:
/// ```
/// use gsim::bits;
/// use std::num::NonZeroU8;
///
/// let state = bits!(1, 0, X, Z);
/// assert_eq!(state.display_string(NonZeroU8::new(4).unwrap()), "10XZ");
/// ```
#[macro_export]
macro_rules! bits {
    (@BIT Z) => { $crate::LogicBitState::HighZ };
    (@BIT z) => { $crate::LogicBitState::HighZ };
    (@BIT X) => { $crate::LogicBitState::Undefined };
    (@BIT x) => { $crate::LogicBitState::Undefined };
    (@BIT 0) => { $crate::LogicBitState::Logic0 };
    (@BIT 1) => { $crate::LogicBitState::Logic1 };
    ($($bit:tt),+) => {{
        $crate::LogicState::from_bits(&[$($crate::bits!(@BIT $bit)),+])
    }}
}

pub use bits;
