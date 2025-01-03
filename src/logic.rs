#![allow(dead_code)]

mod alloc;
pub use alloc::*;

mod state;
pub use state::*;

use crate::{inline_vec, SafeDivCeil};
use smallvec::{smallvec, SmallVec};
use std::fmt;
use std::num::NonZeroU8;
use std::ops::*;

/// The number of bits in a wire or register
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct BitWidth(u8);

impl BitWidth {
    /// A width of 1
    pub const MIN: Self = Self(u8::MIN);
    /// A width of 256
    pub const MAX: Self = Self(u8::MAX);

    /// Creates a new bit width from an integer
    #[inline]
    pub const fn new(value: u32) -> Option<Self> {
        if value > 0 {
            let value = value - 1;
            if value <= (u8::MAX as u32) {
                return Some(Self(value as u8));
            }
        }

        None
    }

    /// Gets the width as an integer
    #[inline]
    pub const fn get(self) -> u32 {
        (self.0 as u32) + 1
    }

    #[inline]
    pub(crate) const fn word_len(self) -> u32 {
        self.get().div_ceil(u32::BITS)
    }

    #[inline]
    pub(crate) const fn last_word_width(self) -> Self {
        Self(self.0 % (u32::BITS as u8))
    }

    #[inline]
    pub(crate) const fn last_word_mask(self) -> u32 {
        ((1u64 << self.last_word_width().get()) - 1) as u32
    }
}

impl TryFrom<u32> for BitWidth {
    type Error = ();

    #[inline]
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(())
    }
}

impl From<BitWidth> for u32 {
    #[inline]
    fn from(value: BitWidth) -> Self {
        value.get()
    }
}

impl fmt::Debug for BitWidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.get(), f)
    }
}

impl fmt::Display for BitWidth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.get(), f)
    }
}

/// Creates a [BitWidth] from a literal in a constant expression
#[macro_export]
macro_rules! bit_width {
    ($value:expr) => {
        const {
            match $crate::BitWidth::new($value) {
                Some(bit_width) => bit_width,
                None => panic!("invalid bit width"),
            }
        }
    };
}

/*
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
#[repr(C, align(8))]
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
        debug_assert!(!bits.is_empty());
        debug_assert!(bits.len() <= (Self::BITS.get() as usize));

        let mut state = 0;
        let mut valid = 0;

        for bit in bits {
            state <<= 1;
            valid <<= 1;

            let (bit_state, bit_valid) = match bit {
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

    fn parse(s: &[u8]) -> Result<Self, ParseError> {
        debug_assert!(!s.is_empty());
        debug_assert!(s.len() <= (Self::BITS.get() as usize));

        let mut state = 0;
        let mut valid = 0;

        for &c in s {
            state <<= 1;
            valid <<= 1;

            let bit = LogicBitState::from_ascii_char(c).ok_or(ParseError::IllegalCharacter(c))?;
            let (bit_state, bit_valid) = match bit {
                LogicBitState::HighZ => (0, 0),
                LogicBitState::Undefined => (1, 0),
                LogicBitState::Logic0 => (0, 1),
                LogicBitState::Logic1 => (1, 1),
            };

            state |= bit_state;
            valid |= bit_valid;
        }

        Ok(Self {
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

    #[inline]
    pub(crate) fn undefined_to_logic_0(self) -> Self {
        Self {
            state: self.state & self.valid,
            valid: self.state | self.valid,
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
pub enum FromBigIntError {
    /// The number of words was not between 1 and 8 inclusive
    InvalidWordCount,
}

#[derive(Debug, Clone)]
pub enum FromBitsError {
    /// The number of bits was not between 1 and 255 inclusive
    InvalidBitCount,
}

#[derive(Debug, Clone)]
pub enum ParseError {
    /// The string contained a character other than `x`, `X`, `z`, `Z`, `0` or `1`
    IllegalCharacter(u8),
    /// The number of bits was not between 1 and 255 inclusive
    InvalidBitCount,
}

#[derive(Debug, Clone)]
pub enum ToIntError {
    /// The specified width was bigger than 32
    InvalidWidth,
    /// The first `width` bits of the logic state are not representable by an integer
    Unrepresentable,
}

#[derive(Debug, Clone)]
pub enum ToBoolError {
    /// The first bit of the logic state is not representable by a boolean
    Unrepresentable,
}

#[derive(Debug, Clone)]
pub enum ToBigIntError {
    /// The first `width` bits of the logic state are not representable by an integer
    Unrepresentable,
}

pub(crate) const MAX_ATOM_COUNT: usize = NonZeroU8::MAX.get().div_ceil(Atom::BITS.get()) as usize;

//#[derive(Debug, Clone)]
//pub(crate) enum LogicStateRepr {
//    HighZ,
//    Undefined,
//    Logic0,
//    Logic1,
//    Int(inline_vec!(u32)),
//    Bits(inline_vec!(Atom)),
//}

assert_eq_size!(LogicStateRepr, [u32; 4]);
assert_eq_align!(LogicStateRepr, usize);

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
        use crate::InlineCount;

        let mut array = [0; u32::INLINE_COUNT];
        array[0] = value;

        // TODO: if smallvec ever supports it, construct a length 1 vector instead
        let vec = SmallVec::from_const(array);
        Self(LogicStateRepr::Int(vec))
    }

    /// Creates a new logic state representing the given boolean value
    ///
    /// Bits past the first one are implicitely assigned the value 0
    #[inline]
    pub const fn from_bool(value: bool) -> Self {
        Self::from_int(value as u32)
    }

    /// Creates a new logic state representing the given integer value
    ///
    /// Integer words are given in little endian order, bits past the end are implicitely assigned the value 0
    #[inline]
    pub fn from_big_int<T: Into<inline_vec!(u32)>>(value: T) -> Result<Self, FromBigIntError> {
        let list = value.into();

        if (1..=MAX_ATOM_COUNT).contains(&list.len()) {
            Ok(Self(LogicStateRepr::Int(list)))
        } else {
            Err(FromBigIntError::InvalidWordCount)
        }
    }

    /// Creates a new logic state from the given bit planes
    ///
    /// Bit plane words are given in little endian order
    ///
    /// Bits in the planes correspond to logic state like so
    ///  plane 0 | plane 1 | meaning
    /// ---------|---------|---------
    ///     0    |    0    | High-Z
    ///     1    |    0    | Undefined
    ///     0    |    1    | Logic 0
    ///     1    |    1    | Logic 1
    pub fn from_bit_planes(plane_0: &[u32], plane_1: &[u32]) -> Self {
        Self(LogicStateRepr::Bits(
            plane_0
                .iter()
                .zip(plane_1)
                .map(|(&state, &valid)| Atom::from_state_valid(state, valid))
                .collect(),
        ))
    }

    /// Creates a new logic state from the given bits (most significant bit first)
    ///
    /// Bits past the specified ones are implicitely assigned the value Z
    pub fn from_bits(bits: &[LogicBitState]) -> Result<Self, FromBitsError> {
        let width = u8::try_from(bits.len())
            .and_then(NonZeroU8::try_from)
            .map_err(|_| FromBitsError::InvalidBitCount)?;

        let head_width = (width.get() % Atom::BITS.get()) as usize;
        let list_len = (width.get().div_ceil(Atom::BITS.get())) as usize;

        let mut list = smallvec![Atom::HIGH_Z; list_len];
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

        Ok(Self(LogicStateRepr::Bits(list)))
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
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let s = s.as_bytes();

        let width = u8::try_from(s.len())
            .and_then(NonZeroU8::try_from)
            .map_err(|_| ParseError::InvalidBitCount)?;
        let head_width = (width.get() % Atom::BITS.get()) as usize;
        let list_len = (width.get().div_ceil(Atom::BITS.get())) as usize;

        let mut list = smallvec![Atom::HIGH_Z; list_len];
        let mut i = list_len;

        if head_width > 0 {
            let head_str = &s[..head_width];
            let head = Atom::parse(head_str)?;

            i -= 1;
            list[i] = head;
        }

        let tail_str = &s[head_width..];
        let mut chunks = tail_str.chunks_exact(Atom::BITS.get() as usize);
        for item_bits in chunks.by_ref() {
            let item = Atom::parse(item_bits)?;

            i -= 1;
            list[i] = item;
        }
        debug_assert_eq!(chunks.remainder().len(), 0);

        Ok(Self(LogicStateRepr::Bits(list)))
    }

    /// Converts the first `width` bits of the logic state into an integer
    pub fn to_int(&self, width: NonZeroU8) -> Result<u32, ToIntError> {
        let Some(width) = AtomWidth::new(width.get()) else {
            return Err(ToIntError::InvalidWidth);
        };
        let mask = LogicStorage::mask(width);

        match &self.0 {
            LogicStateRepr::HighZ | LogicStateRepr::Undefined => Err(ToIntError::Unrepresentable),
            LogicStateRepr::Logic0 => Ok(0),
            LogicStateRepr::Logic1 => Ok(mask.get()),
            LogicStateRepr::Int(list) => Ok(list[0] & mask.get()),
            LogicStateRepr::Bits(list) => {
                if list[0].is_valid(width) {
                    Ok((list[0].state & mask).get())
                } else {
                    Err(ToIntError::Unrepresentable)
                }
            }
        }
    }

    /// Converts the first bit of the logic state into a boolean
    pub fn to_bool(&self) -> Result<bool, ToBoolError> {
        match &self.0 {
            LogicStateRepr::HighZ | LogicStateRepr::Undefined => Err(ToBoolError::Unrepresentable),
            LogicStateRepr::Logic0 => Ok(false),
            LogicStateRepr::Logic1 => Ok(true),
            LogicStateRepr::Int(list) => Ok((list[0] & 0x1) > 0),
            LogicStateRepr::Bits(list) => match list[0].get_bit_state(AtomOffset::MIN) {
                LogicBitState::HighZ | LogicBitState::Undefined => {
                    Err(ToBoolError::Unrepresentable)
                }
                LogicBitState::Logic0 => Ok(false),
                LogicBitState::Logic1 => Ok(true),
            },
        }
    }

    /// Converts the first `width` bits of the logic state into an integer
    ///
    /// Integer words are given in little endian order
    pub fn to_big_int(&self, width: NonZeroU8) -> Result<inline_vec!(u32), ToBigIntError> {
        let word_count = width.safe_div_ceil(Atom::BITS).get() as usize;

        let mut words = match &self.0 {
            LogicStateRepr::HighZ | LogicStateRepr::Undefined => {
                return Err(ToBigIntError::Unrepresentable)
            }
            LogicStateRepr::Logic0 => smallvec![u32::MIN; word_count],
            LogicStateRepr::Logic1 => smallvec![u32::MAX; word_count],
            LogicStateRepr::Int(list) => list[..word_count].iter().copied().collect(),
            LogicStateRepr::Bits(list) => {
                let mut total_width = width.get();

                list[..word_count]
                    .iter()
                    .map(|atom| {
                        let width = AtomWidth::new(total_width).unwrap_or(AtomWidth::MAX);
                        total_width -= width.get();

                        if atom.is_valid(width) {
                            Ok(atom.state.get())
                        } else {
                            Err(ToBigIntError::Unrepresentable)
                        }
                    })
                    .collect::<Result<_, _>>()?
            }
        };

        let last_width = width.get() % Atom::BITS.get();
        if let Some(last_width) = AtomWidth::new(last_width) {
            *words.last_mut().unwrap() &= LogicStorage::mask(last_width).get();
        }

        Ok(words)
    }

    /// Converts the first `width` bits of the logic state into bit planes
    ///
    /// Bit plane words are given in little endian order
    ///
    /// Bits in the planes correspond to logic state like so
    ///  plane 0 | plane 1 | meaning
    /// ---------|---------|---------
    ///     0    |    0    | High-Z
    ///     1    |    0    | Undefined
    ///     0    |    1    | Logic 0
    ///     1    |    1    | Logic 1
    pub fn to_bit_planes(&self, width: NonZeroU8) -> (inline_vec!(u32), inline_vec!(u32)) {
        let word_count = width.safe_div_ceil(Atom::BITS).get() as usize;

        let (mut plane_0_words, mut plane_1_words) = match &self.0 {
            LogicStateRepr::HighZ => (
                smallvec![u32::MIN; word_count],
                smallvec![u32::MIN; word_count],
            ),
            LogicStateRepr::Undefined => (
                smallvec![u32::MAX; word_count],
                smallvec![u32::MIN; word_count],
            ),
            LogicStateRepr::Logic0 => (
                smallvec![u32::MIN; word_count],
                smallvec![u32::MAX; word_count],
            ),
            LogicStateRepr::Logic1 => (
                smallvec![u32::MAX; word_count],
                smallvec![u32::MAX; word_count],
            ),
            LogicStateRepr::Int(list) => (
                list[..word_count].iter().copied().collect(),
                smallvec![u32::MAX; word_count],
            ),
            LogicStateRepr::Bits(list) => (
                list[..word_count].iter().map(|atom| atom.state.0).collect(),
                list[..word_count].iter().map(|atom| atom.valid.0).collect(),
            ),
        };

        let last_width = width.get() % Atom::BITS.get();
        if let Some(last_width) = AtomWidth::new(last_width) {
            *plane_0_words.last_mut().unwrap() &= LogicStorage::mask(last_width).get();
            *plane_1_words.last_mut().unwrap() &= LogicStorage::mask(last_width).get();
        }

        (plane_0_words, plane_1_words)
    }

    /// Gets the logic state of a single bit
    pub fn get_bit_state(&self, bit_index: u8) -> LogicBitState {
        match self.0 {
            LogicStateRepr::HighZ => LogicBitState::HighZ,
            LogicStateRepr::Undefined => LogicBitState::Undefined,
            LogicStateRepr::Logic0 => LogicBitState::Logic0,
            LogicStateRepr::Logic1 => LogicBitState::Logic1,
            LogicStateRepr::Int(ref list) => {
                let item_index = (bit_index / Atom::BITS.get()) as usize;
                let bit_index = AtomOffset::new(bit_index % Atom::BITS.get()).unwrap();

                if let Some(&item) = list.get(item_index) {
                    let item = LogicStorage(item);
                    let state_bit = item.get_bit(bit_index);
                    LogicBitState::from_bits(state_bit, true)
                } else {
                    LogicBitState::Logic0
                }
            }
            LogicStateRepr::Bits(ref list) => {
                let item_index = (bit_index / Atom::BITS.get()) as usize;
                let bit_index = AtomOffset::new(bit_index % Atom::BITS.get()).unwrap();

                if let Some(item) = list.get(item_index) {
                    item.get_bit_state(bit_index)
                } else {
                    LogicBitState::HighZ
                }
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
            LogicStateRepr::Int(ref list) => LogicStateIter::Int { list, current: 0 },
            LogicStateRepr::Bits(ref list) => LogicStateIter::Bits { list, current: 0 },
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

    pub(crate) fn undefined_to_logic_0(self) -> Self {
        Self(match self.0 {
            LogicStateRepr::HighZ => LogicStateRepr::HighZ,
            LogicStateRepr::Undefined => LogicStateRepr::Logic0,
            LogicStateRepr::Logic0 => LogicStateRepr::Logic0,
            LogicStateRepr::Logic1 => LogicStateRepr::Logic1,
            LogicStateRepr::Int(value) => LogicStateRepr::Int(value),
            LogicStateRepr::Bits(mut atoms) => {
                atoms
                    .iter_mut()
                    .for_each(|atom| *atom = atom.undefined_to_logic_0());
                LogicStateRepr::Bits(atoms)
            }
        })
    }
}

pub(crate) enum LogicStateIter<'a> {
    HighZ,
    Undefined,
    Logic0,
    Logic1,
    Int { list: &'a [u32], current: usize },
    Bits { list: &'a [Atom], current: usize },
}

impl Iterator for LogicStateIter<'_> {
    type Item = Atom;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            LogicStateIter::HighZ => Some(Atom::HIGH_Z),
            LogicStateIter::Undefined => Some(Atom::UNDEFINED),
            LogicStateIter::Logic0 => Some(Atom::LOGIC_0),
            LogicStateIter::Logic1 => Some(Atom::LOGIC_1),
            LogicStateIter::Int { list, current } => {
                let item = list.get(*current).copied();
                if item.is_some() {
                    *current += 1;
                }
                item.map(Atom::from_int)
            }
            LogicStateIter::Bits { list, current } => {
                let item = list.get(*current).copied();
                if item.is_some() {
                    *current += 1;
                }
                item
            }
        }
    }
}
 */
