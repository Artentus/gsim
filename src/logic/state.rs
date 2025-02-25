use crate::{BitWidth, bit_width};
use std::fmt;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::str::FromStr;

/// An error that can occur when parsing a logic state
#[derive(Debug, Clone)]
pub enum LogicStateFromStrError {
    /// The number of bits was not between 1 and 256 inclusive
    InvalidBitWidth,
    /// The string contained a character other than `x`, `X`, `z`, `Z`, `0` or `1`
    IllegalCharacter(u8),
}

/// The logic state of a single bit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LogicBitState {
    /// The low logic level
    Logic0 = 0b00,
    /// The high logic level
    Logic1 = 0b01,
    /// The high impedance state
    HighZ = 0b10,
    /// An undefined logic level
    Undefined = 0b11,
}

impl LogicBitState {
    #[inline]
    pub(crate) const fn from_bits(bits: u8) -> Self {
        match bits {
            0b00 => Self::Logic0,
            0b01 => Self::Logic1,
            0b10 => Self::HighZ,
            0b11 => Self::Undefined,
            _ => panic!("invalid bit pattern"),
        }
    }

    #[inline]
    pub(crate) const fn to_bits(self) -> u8 {
        self as u8
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

    /// - `b'Z'` | `b'z'` => `HighZ`
    /// - `b'X'` | `b'x'` => `Undefined`
    /// - `b'0'` => `Logic0`
    /// - `b'1'` => `Logic1`
    #[inline]
    pub const fn from_ascii_char(c: u8) -> Option<Self> {
        match c {
            b'Z' | b'z' => Some(Self::HighZ),
            b'X' | b'x' => Some(Self::Undefined),
            b'0' => Some(Self::Logic0),
            b'1' => Some(Self::Logic1),
            _ => None,
        }
    }

    /// - `HighZ` => `b'Z'`
    /// - `Undefined` => `b'X'`
    /// - `Logic0` => `b'0'`
    /// - `Logic1` => `b'1'`
    #[inline]
    pub const fn to_ascii_char(self) -> u8 {
        match self {
            Self::Logic0 => b'0',
            Self::Logic1 => b'1',
            Self::HighZ => b'Z',
            Self::Undefined => b'X',
        }
    }

    /// - `'Z'` | `'z'` => `HighZ`
    /// - `'X'` | `'x'` => `Undefined`
    /// - `'0'` => `Logic0`
    /// - `'1'` => `Logic1`
    #[inline]
    pub const fn from_char(c: char) -> Option<Self> {
        if c.is_ascii() {
            Self::from_ascii_char(c as u8)
        } else {
            None
        }
    }

    /// - `HighZ` => `'Z'`
    /// - `Undefined` => `'X'`
    /// - `Logic0` => `'0'`
    /// - `Logic1` => `'1'`
    #[inline]
    pub const fn to_char(self) -> char {
        match self {
            Self::Logic0 => '0',
            Self::Logic1 => '1',
            Self::HighZ => 'Z',
            Self::Undefined => 'X',
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

impl TryFrom<char> for LogicBitState {
    type Error = ();

    #[inline]
    fn try_from(value: char) -> Result<Self, Self::Error> {
        Self::from_char(value).ok_or(())
    }
}

impl From<LogicBitState> for char {
    #[inline]
    fn from(value: LogicBitState) -> Self {
        value.to_char()
    }
}

impl FromStr for LogicBitState {
    type Err = LogicStateFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.as_bytes();

        match s {
            [b'0'] => Ok(Self::Logic0),
            [b'1'] => Ok(Self::Logic1),
            [b'z'] | [b'Z'] => Ok(Self::HighZ),
            [b'x'] | [b'X'] => Ok(Self::Undefined),
            &[c] => Err(LogicStateFromStrError::IllegalCharacter(c)),
            _ => Err(LogicStateFromStrError::InvalidBitWidth),
        }
    }
}

impl fmt::Display for LogicBitState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::HighZ => "Z",
            Self::Undefined => "X",
            Self::Logic0 => "0",
            Self::Logic1 => "1",
        };

        f.write_str(s)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for LogicBitState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_char(self.to_char())
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

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "one of the characters ['0', '1', 'Z', 'z', 'X', 'x']")
            }

            fn visit_char<E>(self, c: char) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                LogicBitState::from_char(c).ok_or(E::invalid_value(Unexpected::Char(c), &self))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                v.parse()
                    .map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
            }
        }

        deserializer.deserialize_str(LogicBitStateVisitor)
    }
}

#[inline]
fn space_out_bits(v: u32) -> u64 {
    let v = v as u64;
    let v = (v | (v << 16)) & 0x0000FFFF0000FFFF;
    let v = (v | (v << 08)) & 0x00FF00FF00FF00FF;
    let v = (v | (v << 04)) & 0x0F0F0F0F0F0F0F0F;
    let v = (v | (v << 02)) & 0x3333333333333333;
    let v = (v | (v << 01)) & 0x5555555555555555;
    v
}

#[inline]
fn interleave_bits((a, b): (u32, u32)) -> u64 {
    let a = space_out_bits(a);
    let b = space_out_bits(b);
    a | (b << 1)
}

struct Interleave<'a> {
    inner: std::iter::Zip<
        std::iter::Copied<std::slice::Iter<'a, u32>>,
        std::iter::Copied<std::slice::Iter<'a, u32>>,
    >,
}

impl<'a> Interleave<'a> {
    #[inline]
    fn new(bit_plane_0: &'a [u32], bit_plane_1: &'a [u32]) -> Self {
        Self {
            inner: bit_plane_0.iter().copied().zip(bit_plane_1.iter().copied()),
        }
    }
}

impl Iterator for Interleave<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(interleave_bits)
    }
}

#[allow(missing_docs)]
#[allow(missing_debug_implementations)]
pub struct Bits<'a> {
    bit_width: u32,
    current: Option<u64>,
    inner: Interleave<'a>,
}

impl<'a> Bits<'a> {
    #[inline]
    fn new(bit_width: BitWidth, bit_plane_0: &'a [u32], bit_plane_1: &'a [u32]) -> Self {
        let mut inner = Interleave::new(bit_plane_0, bit_plane_1);

        Self {
            bit_width: bit_width.get(),
            current: inner.next(),
            inner,
        }
    }
}

impl Iterator for Bits<'_> {
    type Item = LogicBitState;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bit_width == 0 {
            return None;
        }

        if let Some(current) = &mut self.current {
            let bit_state = LogicBitState::from_bits((*current & 0x3) as u8);

            self.bit_width -= 1;
            *current >>= 2;

            if (self.bit_width % u32::BITS) == 0 {
                self.current = self.inner.next();
            }

            Some(bit_state)
        } else {
            None
        }
    }
}

impl std::iter::FusedIterator for Bits<'_> {}

const MAX_WORD_COUNT: usize = BitWidth::MAX.get().div_ceil(u32::BITS) as usize;

const ALL_ZERO: [u32; MAX_WORD_COUNT] = [u32::MIN; MAX_WORD_COUNT];
const ALL_ONE: [u32; MAX_WORD_COUNT] = [u32::MAX; MAX_WORD_COUNT];

#[derive(Debug, Clone, Copy)]
enum LogicStateRepr {
    Logic0 {
        bit_width: BitWidth,
    },
    Logic1 {
        bit_width: BitWidth,
    },
    HighZ {
        bit_width: BitWidth,
    },
    Undefined {
        bit_width: BitWidth,
    },
    TwoState {
        bit_width: BitWidth,
        bit_plane_0: [u32; 5],
    },
    FourState {
        bit_width: BitWidth,
        bit_plane_0: [u32; 2],
        bit_plane_1: [u32; 2],
    },
    Ptr {
        bit_width: BitWidth,
        bit_plane_0: NonNull<u32>,
        bit_plane_1: NonNull<u32>,
    },
}

impl LogicStateRepr {
    #[inline]
    const fn bit_width(&self) -> BitWidth {
        match *self {
            Self::Logic0 { bit_width }
            | Self::Logic1 { bit_width }
            | Self::HighZ { bit_width }
            | Self::Undefined { bit_width }
            | Self::TwoState { bit_width, .. }
            | Self::FourState { bit_width, .. }
            | Self::Ptr { bit_width, .. } => bit_width,
        }
    }

    #[inline]
    fn bit_planes(&self) -> (&[u32], &[u32]) {
        let word_len = self.bit_width().word_len() as usize;

        match self {
            LogicStateRepr::Logic0 { .. } => (&ALL_ZERO[..word_len], &ALL_ZERO[..word_len]),
            LogicStateRepr::Logic1 { .. } => (&ALL_ONE[..word_len], &ALL_ZERO[..word_len]),
            LogicStateRepr::HighZ { .. } => (&ALL_ZERO[..word_len], &ALL_ONE[..word_len]),
            LogicStateRepr::Undefined { .. } => (&ALL_ONE[..word_len], &ALL_ONE[..word_len]),
            LogicStateRepr::TwoState { bit_plane_0, .. } => {
                (&bit_plane_0[..word_len], &ALL_ZERO[..word_len])
            }
            LogicStateRepr::FourState {
                bit_plane_0,
                bit_plane_1,
                ..
            } => (&bit_plane_0[..word_len], &bit_plane_1[..word_len]),
            LogicStateRepr::Ptr {
                bit_plane_0,
                bit_plane_1,
                ..
            } => unsafe {
                (
                    std::slice::from_raw_parts(bit_plane_0.as_ptr(), word_len),
                    std::slice::from_raw_parts(bit_plane_1.as_ptr(), word_len),
                )
            },
        }
    }

    fn bit(&self, index: u32) -> Option<LogicBitState> {
        if index >= self.bit_width().get() {
            return None;
        }

        let word_index = (index / u32::BITS) as usize;
        let bit_index = (index % u32::BITS) as usize;

        let (bit_plane_0, bit_plane_1) = self.bit_planes();
        let bit_0 = ((bit_plane_0[word_index] >> bit_index) as u8) & 0b1;
        let bit_1 = ((bit_plane_1[word_index] >> bit_index) as u8) & 0b1;
        Some(LogicBitState::from_bits(bit_0 | (bit_1 << 1)))
    }

    fn bits(&self) -> Bits<'_> {
        let (bit_plane_0, bit_plane_1) = self.bit_planes();
        Bits::new(self.bit_width(), bit_plane_0, bit_plane_1)
    }
}

impl fmt::Display for LogicStateRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bit_width = self.bit_width().get() as usize;
        let mut buffer = [0u8; BitWidth::MAX.get() as usize];
        for (i, bit) in self.bits().enumerate() {
            buffer[bit_width - i - 1] = bit.to_ascii_char();
        }

        let s = unsafe { std::str::from_utf8_unchecked(&buffer[..bit_width]) };
        f.write_str(s)
    }
}

/// The state of a group of bits
#[repr(transparent)]
pub struct LogicState {
    repr: LogicStateRepr,
}

impl LogicState {
    /// A state with all bits having the value `0`
    #[inline]
    pub const fn logic_0(bit_width: BitWidth) -> Self {
        Self {
            repr: LogicStateRepr::Logic0 { bit_width },
        }
    }

    /// A state with all bits having the value `1`
    #[inline]
    pub const fn logic_1(bit_width: BitWidth) -> Self {
        Self {
            repr: LogicStateRepr::Logic1 { bit_width },
        }
    }

    /// A state with all bits being in high impedance
    #[inline]
    pub const fn high_z(bit_width: BitWidth) -> Self {
        Self {
            repr: LogicStateRepr::HighZ { bit_width },
        }
    }

    /// A state with all bits being undefined
    #[inline]
    pub const fn undefined(bit_width: BitWidth) -> Self {
        Self {
            repr: LogicStateRepr::Undefined { bit_width },
        }
    }

    /// Creates a state from a single bit
    #[inline]
    pub const fn from_bit(value: LogicBitState) -> Self {
        match value {
            LogicBitState::Logic0 => Self::logic_0(bit_width!(1)),
            LogicBitState::Logic1 => Self::logic_1(bit_width!(1)),
            LogicBitState::HighZ => Self::high_z(bit_width!(1)),
            LogicBitState::Undefined => Self::undefined(bit_width!(1)),
        }
    }

    /// Creates a state representing a boolean
    #[inline]
    pub const fn from_bool(value: bool) -> Self {
        if value {
            Self::logic_1(bit_width!(1))
        } else {
            Self::logic_0(bit_width!(1))
        }
    }

    /// Creates a state representing a number
    #[inline]
    pub const fn from_u32(value: u32, bit_width: BitWidth) -> Self {
        assert!(bit_width.get() <= u32::BITS);

        Self {
            repr: LogicStateRepr::TwoState {
                bit_width,
                bit_plane_0: [value, 0, 0, 0, 0],
            },
        }
    }

    /// Creates a state representing a number
    #[inline]
    pub const fn from_u64(value: u64, bit_width: BitWidth) -> Self {
        assert!(bit_width.get() <= u64::BITS);

        let low = value as u32;
        let high = (value >> 32) as u32;

        Self {
            repr: LogicStateRepr::TwoState {
                bit_width,
                bit_plane_0: [low, high, 0, 0, 0],
            },
        }
    }

    fn new_alloc(bit_width: BitWidth, bit_plane_0: Vec<u32>, bit_plane_1: Vec<u32>) -> Self {
        assert_eq!(bit_plane_0.len(), bit_plane_0.capacity());
        assert_eq!(bit_plane_1.len(), bit_plane_1.capacity());

        let word_len = bit_width.word_len() as usize;
        debug_assert_eq!(bit_plane_0.len(), word_len);
        debug_assert_eq!(bit_plane_1.len(), word_len);

        let mut bit_plane_0 = ManuallyDrop::new(bit_plane_0);
        let mut bit_plane_1 = ManuallyDrop::new(bit_plane_1);
        let bit_plane_0 = NonNull::new(bit_plane_0.as_mut_ptr()).unwrap();
        let bit_plane_1 = NonNull::new(bit_plane_1.as_mut_ptr()).unwrap();

        Self {
            repr: LogicStateRepr::Ptr {
                bit_width,
                bit_plane_0,
                bit_plane_1,
            },
        }
    }

    /// Creates a state representing a number (least significant word first)
    pub fn from_big_int(bit_width: BitWidth, value: &[u32]) -> Self {
        let word_count = bit_width.get().div_ceil(u32::BITS) as usize;
        if word_count <= 5 {
            let mut buffer = [0; 5];
            let copy_count = word_count.min(value.len());
            buffer[..copy_count].copy_from_slice(&value[..copy_count]);

            Self {
                repr: LogicStateRepr::TwoState {
                    bit_width,
                    bit_plane_0: buffer,
                },
            }
        } else {
            let mut bit_plane_0 = value.to_vec();
            bit_plane_0.resize(word_count, 0);
            bit_plane_0.shrink_to_fit();
            Self::new_alloc(bit_width, bit_plane_0, ALL_ZERO[..word_count].to_vec())
        }
    }

    /// Creates a state from raw bit planes (least significant words first)
    pub fn from_bit_planes(bit_width: BitWidth, bit_plane_0: &[u32], bit_plane_1: &[u32]) -> Self {
        let word_count = bit_width.get().div_ceil(u32::BITS) as usize;
        if word_count <= 2 {
            let mut bit_plane_0_buffer = [0; 2];
            let mut bit_plane_1_buffer = [0; 2];
            let copy_count = word_count.min(bit_plane_0.len());
            bit_plane_0_buffer[..copy_count].copy_from_slice(&bit_plane_0[..copy_count]);
            let copy_count = word_count.min(bit_plane_1.len());
            bit_plane_1_buffer[..copy_count].copy_from_slice(&bit_plane_1[..copy_count]);

            Self {
                repr: LogicStateRepr::FourState {
                    bit_width,
                    bit_plane_0: bit_plane_0_buffer,
                    bit_plane_1: bit_plane_1_buffer,
                },
            }
        } else {
            let mut bit_plane_0 = bit_plane_0.to_vec();
            bit_plane_0.resize(word_count, 0);
            bit_plane_0.shrink_to_fit();
            let mut bit_plane_1 = bit_plane_1.to_vec();
            bit_plane_1.resize(word_count, u32::MAX);
            bit_plane_1.shrink_to_fit();
            Self::new_alloc(bit_width, bit_plane_0, bit_plane_1)
        }
    }

    /// Creates a state with the specified bits (least significant bit first)
    pub fn from_bits(bits: &[LogicBitState]) -> Self {
        let len: u32 = bits.len().try_into().expect("invalid bit width");
        let bit_width = BitWidth::new(len).expect("invalid bit width");

        let mut bit_plane_0 = [0; MAX_WORD_COUNT];
        let mut bit_plane_1 = [0; MAX_WORD_COUNT];
        for (i, &bit) in bits.iter().enumerate() {
            let bits = bit.to_bits();
            let bit_0 = (bits & 0b1) as u32;
            let bit_1 = (bits >> 1) as u32;

            let word_index = i / (u32::BITS as usize);
            let bit_index = i % (u32::BITS as usize);
            bit_plane_0[word_index] |= bit_0 << bit_index;
            bit_plane_1[word_index] |= bit_1 << bit_index;
        }

        Self::from_bit_planes(bit_width, &bit_plane_0, &bit_plane_1)
    }

    /// The number of bits in this state
    #[inline]
    pub const fn bit_width(&self) -> BitWidth {
        self.repr.bit_width()
    }

    /// The raw bit planes of the state
    #[inline]
    pub fn bit_planes(&self) -> (&[u32], &[u32]) {
        self.repr.bit_planes()
    }

    /// Gets a bit at a specified index
    #[inline]
    pub fn bit(&self, index: u32) -> Option<LogicBitState> {
        self.repr.bit(index)
    }

    /// Iterates all bits in the state
    #[inline]
    pub fn bits(&self) -> Bits<'_> {
        self.repr.bits()
    }

    /// Turns the logic state into a borrowed form
    #[inline]
    pub const fn borrow(&self) -> LogicStateRef<'_> {
        LogicStateRef {
            repr: self.repr,
            _borrow: PhantomData,
        }
    }
}

impl From<LogicBitState> for LogicState {
    #[inline]
    fn from(value: LogicBitState) -> Self {
        Self::from_bit(value)
    }
}

impl From<bool> for LogicState {
    #[inline]
    fn from(value: bool) -> Self {
        Self::from_bool(value)
    }
}

impl fmt::Display for LogicState {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.repr, f)
    }
}

impl fmt::Debug for LogicState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.repr, f)
    }
}

fn parse_into(s: &[u8], bit_plane_0: &mut [u32], bit_plane_1: &mut [u32]) {
    for (i, &c) in s.iter().enumerate() {
        let word_index = i / (u32::BITS as usize);
        let bit_index = i % (u32::BITS as usize);

        let (bit_0, bit_1) = match c {
            b'0' => (0, 0),
            b'1' => (1, 0),
            b'z' | b'Z' => (0, 1),
            b'x' | b'X' => (1, 1),
            _ => unreachable!(),
        };

        bit_plane_0[word_index] |= bit_0 << bit_index;
        bit_plane_1[word_index] |= bit_1 << bit_index;
    }
}

impl FromStr for LogicState {
    type Err = LogicStateFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.as_bytes();
        let len: u32 = s
            .len()
            .try_into()
            .map_err(|_| LogicStateFromStrError::InvalidBitWidth)?;
        let bit_width = BitWidth::new(len).ok_or(LogicStateFromStrError::InvalidBitWidth)?;

        let mut contains_0 = false;
        let mut contains_1 = false;
        let mut contains_z = false;
        let mut contains_x = false;
        for &c in s {
            match c {
                b'0' => contains_0 = true,
                b'1' => contains_1 = true,
                b'z' | b'Z' => contains_z = true,
                b'x' | b'X' => contains_x = true,
                _ => return Err(LogicStateFromStrError::IllegalCharacter(c)),
            }
        }

        match (contains_0, contains_1, contains_z, contains_x) {
            (true, false, false, false) => Ok(Self::logic_0(bit_width)),
            (false, true, false, false) => Ok(Self::logic_1(bit_width)),
            (false, false, true, false) => Ok(Self::high_z(bit_width)),
            (false, false, false, true) => Ok(Self::undefined(bit_width)),
            (true, true, false, false) => {
                let word_len = bit_width.word_len() as usize;
                if word_len <= 5 {
                    let mut bit_plane_0 = [0; 5];
                    let mut bit_plane_1 = [0; 5];
                    parse_into(s, &mut bit_plane_0, &mut bit_plane_1);
                    assert_eq!(bit_plane_1, [0; 5]);

                    Ok(Self {
                        repr: LogicStateRepr::TwoState {
                            bit_width,
                            bit_plane_0,
                        },
                    })
                } else {
                    let mut bit_plane_0 = vec![0; word_len as usize];
                    let mut bit_plane_1 = vec![0; word_len as usize];
                    parse_into(s, &mut bit_plane_0, &mut bit_plane_1);
                    Ok(Self::new_alloc(bit_width, bit_plane_0, bit_plane_1))
                }
            }
            _ => {
                let word_len = bit_width.word_len() as usize;
                if word_len <= 2 {
                    let mut bit_plane_0 = [0; 2];
                    let mut bit_plane_1 = [0; 2];
                    parse_into(s, &mut bit_plane_0, &mut bit_plane_1);

                    Ok(Self {
                        repr: LogicStateRepr::FourState {
                            bit_width,
                            bit_plane_0,
                            bit_plane_1,
                        },
                    })
                } else {
                    let mut bit_plane_0 = vec![0; word_len as usize];
                    let mut bit_plane_1 = vec![0; word_len as usize];
                    parse_into(s, &mut bit_plane_0, &mut bit_plane_1);
                    Ok(Self::new_alloc(bit_width, bit_plane_0, bit_plane_1))
                }
            }
        }
    }
}

impl Drop for LogicState {
    fn drop(&mut self) {
        if let LogicStateRepr::Ptr {
            bit_width,
            bit_plane_0,
            bit_plane_1,
        } = self.repr
        {
            let word_len = bit_width.word_len() as usize;

            unsafe {
                // SAFETY: this is the only place where deallocation can occurr.
                let _ = Vec::from_raw_parts(bit_plane_0.as_ptr(), word_len, word_len);
                let _ = Vec::from_raw_parts(bit_plane_1.as_ptr(), word_len, word_len);
            }
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for LogicState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
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

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    f,
                    "a string consisting of only the chars ['0', '1', 'Z', 'z', 'X', 'x'] and of length {} to {}",
                    BitWidth::MIN,
                    BitWidth::MAX
                )
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                v.parse()
                    .map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
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
/// let state = bits![1, 0, X, Z];
/// assert_eq!(state.to_string(), "10XZ");
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
        let bits = const {
            let mut bits = [$($crate::bits!(@BIT $bit)),+];
            assert!(bits.len() >= ($crate::BitWidth::MIN.get() as usize));
            assert!(bits.len() <= ($crate::BitWidth::MAX.get() as usize));

            let mut i = 0;
            while i < (bits.len() / 2) {
                let bit = bits[i];
                bits[i] = bits[bits.len() - i - 1];
                bits[bits.len() - i - 1] = bit;
                i += 1;
            }

            bits
        };

        $crate::LogicState::from_bits(&bits)
    }}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CopyFromResult {
    Unchanged,
    Changed,
}

macro_rules! copy_from_impl {
    () => {
        pub(crate) fn copy_from<'other>(
            &mut self,
            other: impl IntoLogicStateRef<'other>,
        ) -> CopyFromResult {
            let other = other.into_logic_state_ref();
            assert_eq!(self.bit_width(), other.bit_width());

            let (dst_plane_0, dst_plane_1) = self.bit_planes_mut();
            let (src_plane_0, src_plane_1) = other.bit_planes();

            let mut changed = false;
            for (dst_word_0, dst_word_1, &src_word_0, &src_word_1) in
                itertools::izip!(dst_plane_0, dst_plane_1, src_plane_0, src_plane_1)
            {
                changed |= (src_word_0 != *dst_word_0) | (src_word_1 != *dst_word_1);
                *dst_word_0 = src_word_0;
                *dst_word_1 = src_word_1;
            }

            if changed {
                CopyFromResult::Changed
            } else {
                CopyFromResult::Unchanged
            }
        }

        pub(crate) fn copy_from_one(
            &mut self,
            [src_plane_0, src_plane_1]: [u32; 2],
        ) -> CopyFromResult {
            let (dst_plane_0, dst_plane_1) = self.bit_planes_mut();
            debug_assert_eq!(dst_plane_0.len(), 1);
            debug_assert_eq!(dst_plane_1.len(), 1);

            if (src_plane_0 != dst_plane_0[0]) || (src_plane_1 != dst_plane_1[0]) {
                dst_plane_0[0] = src_plane_0;
                dst_plane_1[0] = src_plane_1;

                CopyFromResult::Changed
            } else {
                CopyFromResult::Unchanged
            }
        }

        pub(crate) fn copy_from_bit(&mut self, src_bit: LogicBitState) -> CopyFromResult {
            let (dst_plane_0, dst_plane_1) = self.bit_planes_mut();
            debug_assert_eq!(dst_plane_0.len(), 1);
            debug_assert_eq!(dst_plane_1.len(), 1);

            let src_plane_0 = (src_bit.to_bits() & 0b1) as u32;
            let src_plane_1 = (src_bit.to_bits() >> 1) as u32;

            if (src_plane_0 != dst_plane_0[0]) || (src_plane_1 != dst_plane_1[0]) {
                dst_plane_0[0] = src_plane_0;
                dst_plane_1[0] = src_plane_1;

                CopyFromResult::Changed
            } else {
                CopyFromResult::Unchanged
            }
        }
    };
}

#[repr(C)]
pub(crate) struct InlineLogicState {
    bit_width: BitWidth,
    bit_plane_0: [u32; MAX_WORD_COUNT],
    bit_plane_1: [u32; MAX_WORD_COUNT],
}

impl InlineLogicState {
    #[inline]
    pub const fn logic_0(bit_width: BitWidth) -> Self {
        Self {
            bit_width,
            bit_plane_0: ALL_ZERO,
            bit_plane_1: ALL_ZERO,
        }
    }

    #[inline]
    pub const fn logic_1(bit_width: BitWidth) -> Self {
        Self {
            bit_width,
            bit_plane_0: ALL_ONE,
            bit_plane_1: ALL_ZERO,
        }
    }

    #[inline]
    pub const fn high_z(bit_width: BitWidth) -> Self {
        Self {
            bit_width,
            bit_plane_0: ALL_ZERO,
            bit_plane_1: ALL_ONE,
        }
    }

    #[inline]
    pub const fn undefined(bit_width: BitWidth) -> Self {
        Self {
            bit_width,
            bit_plane_0: ALL_ONE,
            bit_plane_1: ALL_ONE,
        }
    }

    #[inline]
    pub(crate) fn bit_width(&self) -> BitWidth {
        self.bit_width
    }

    #[inline]
    pub(crate) fn bit_planes(&self) -> (&[u32], &[u32]) {
        let word_len = self.bit_width.word_len() as usize;
        (&self.bit_plane_0[..word_len], &self.bit_plane_1[..word_len])
    }

    #[inline]
    pub(crate) fn set_logic_0(&mut self) {
        self.bit_plane_0 = ALL_ZERO;
        self.bit_plane_1 = ALL_ZERO;
    }

    #[inline]
    pub(crate) fn set_logic_1(&mut self) {
        self.bit_plane_0 = ALL_ONE;
        self.bit_plane_1 = ALL_ZERO;
    }

    #[inline]
    pub(crate) fn set_high_z(&mut self) {
        self.bit_plane_0 = ALL_ZERO;
        self.bit_plane_1 = ALL_ONE;
    }

    #[inline]
    pub(crate) fn set_undefined(&mut self) {
        self.bit_plane_0 = ALL_ONE;
        self.bit_plane_1 = ALL_ONE;
    }

    #[inline]
    pub(crate) fn bit_planes_mut(&mut self) -> (&mut [u32], &mut [u32]) {
        let word_len = self.bit_width.word_len() as usize;

        (
            &mut self.bit_plane_0[..word_len],
            &mut self.bit_plane_1[..word_len],
        )
    }

    pub(crate) fn bit(&self, index: u32) -> Option<LogicBitState> {
        if index >= self.bit_width().get() {
            return None;
        }

        let word_index = (index / u32::BITS) as usize;
        let bit_index = (index % u32::BITS) as usize;

        let (bit_plane_0, bit_plane_1) = self.bit_planes();
        let bit_0 = ((bit_plane_0[word_index] >> bit_index) as u8) & 0b1;
        let bit_1 = ((bit_plane_1[word_index] >> bit_index) as u8) & 0b1;
        Some(LogicBitState::from_bits(bit_0 | (bit_1 << 1)))
    }

    pub(crate) fn bits(&self) -> Bits<'_> {
        let (bit_plane_0, bit_plane_1) = self.bit_planes();
        Bits::new(self.bit_width(), bit_plane_0, bit_plane_1)
    }

    copy_from_impl!();

    #[inline]
    pub(crate) const fn borrow(&self) -> LogicStateRef<'_> {
        unsafe {
            LogicStateRef::new_ptr(
                self.bit_width,
                NonNull::new_unchecked(self.bit_plane_0.as_slice().as_ptr().cast_mut()),
                NonNull::new_unchecked(self.bit_plane_1.as_slice().as_ptr().cast_mut()),
            )
        }
    }

    #[inline]
    pub(crate) fn borrow_mut(&mut self) -> LogicStateMut<'_> {
        unsafe {
            LogicStateMut::new_ptr(
                self.bit_width,
                NonNull::new_unchecked(self.bit_plane_0.as_mut_slice().as_mut_ptr()),
                NonNull::new_unchecked(self.bit_plane_1.as_mut_slice().as_mut_ptr()),
            )
        }
    }
}

impl fmt::Display for InlineLogicState {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.borrow(), f)
    }
}

impl fmt::Debug for InlineLogicState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.borrow(), f)
    }
}

/// Borrowed form of [LogicState]
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct LogicStateRef<'a> {
    repr: LogicStateRepr,
    _borrow: PhantomData<&'a LogicState>,
}

impl LogicStateRef<'_> {
    /// SAFETY: caller must ensure the pointers are valid for the bit width and the chosen lifetime.
    #[inline]
    pub(crate) const unsafe fn new_ptr(
        bit_width: BitWidth,
        bit_plane_0: NonNull<u32>,
        bit_plane_1: NonNull<u32>,
    ) -> Self {
        Self {
            repr: LogicStateRepr::Ptr {
                bit_width,
                bit_plane_0,
                bit_plane_1,
            },
            _borrow: PhantomData,
        }
    }

    /// The number of bits in this state
    #[inline]
    pub const fn bit_width(&self) -> BitWidth {
        self.repr.bit_width()
    }

    /// The raw bit planes of the state
    #[inline]
    pub fn bit_planes(&self) -> (&[u32], &[u32]) {
        self.repr.bit_planes()
    }

    /// Gets a bit at a specified index
    #[inline]
    pub fn bit(&self, index: u32) -> Option<LogicBitState> {
        self.repr.bit(index)
    }

    /// Iterates all bits in the state
    #[inline]
    pub fn bits(&self) -> Bits<'_> {
        self.repr.bits()
    }

    /// Turns the logic state into an owned form
    pub fn to_owned(&self) -> LogicState {
        if let LogicStateRepr::Ptr {
            bit_width,
            bit_plane_0,
            bit_plane_1,
        } = self.repr
        {
            let word_len = bit_width.word_len() as usize;

            let (bit_plane_0, bit_plane_1) = unsafe {
                (
                    std::slice::from_raw_parts(bit_plane_0.as_ptr(), word_len),
                    std::slice::from_raw_parts(bit_plane_1.as_ptr(), word_len),
                )
            };

            let bit_plane_0 = bit_plane_0.to_vec();
            let bit_plane_1 = bit_plane_1.to_vec();
            LogicState::new_alloc(bit_width, bit_plane_0, bit_plane_1)
        } else {
            LogicState { repr: self.repr }
        }
    }
}

impl fmt::Display for LogicStateRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.repr, f)
    }
}

impl fmt::Debug for LogicStateRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.repr, f)
    }
}

#[repr(C)]
pub(crate) struct LogicStateMut<'a> {
    bit_width: BitWidth,
    bit_plane_0: NonNull<u32>,
    bit_plane_1: NonNull<u32>,
    _borrow: PhantomData<&'a mut LogicState>,
}

impl<'a> LogicStateMut<'a> {
    /// SAFETY: caller must ensure the pointers are valid for the bit width and the chosen lifetime.
    #[inline]
    pub(crate) unsafe fn new_ptr(
        bit_width: BitWidth,
        bit_plane_0: NonNull<u32>,
        bit_plane_1: NonNull<u32>,
    ) -> Self {
        Self {
            bit_width,
            bit_plane_0,
            bit_plane_1,
            _borrow: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn bit_width(&self) -> BitWidth {
        self.bit_width
    }

    #[inline]
    pub(crate) fn bit_planes(&self) -> (&[u32], &[u32]) {
        let word_len = self.bit_width.word_len() as usize;

        unsafe {
            (
                std::slice::from_raw_parts(self.bit_plane_0.as_ptr(), word_len),
                std::slice::from_raw_parts(self.bit_plane_1.as_ptr(), word_len),
            )
        }
    }

    #[inline]
    pub(crate) fn bit_planes_mut(&mut self) -> (&mut [u32], &mut [u32]) {
        let word_len = self.bit_width.word_len() as usize;

        unsafe {
            (
                std::slice::from_raw_parts_mut(self.bit_plane_0.as_ptr(), word_len),
                std::slice::from_raw_parts_mut(self.bit_plane_1.as_ptr(), word_len),
            )
        }
    }

    pub(crate) fn bit(&self, index: u32) -> Option<LogicBitState> {
        if index >= self.bit_width().get() {
            return None;
        }

        let word_index = (index / u32::BITS) as usize;
        let bit_index = (index % u32::BITS) as usize;

        let (bit_plane_0, bit_plane_1) = self.bit_planes();
        let bit_0 = ((bit_plane_0[word_index] >> bit_index) as u8) & 0b1;
        let bit_1 = ((bit_plane_1[word_index] >> bit_index) as u8) & 0b1;
        Some(LogicBitState::from_bits(bit_0 | (bit_1 << 1)))
    }

    pub(crate) fn bits(&self) -> Bits<'_> {
        let (bit_plane_0, bit_plane_1) = self.bit_planes();
        Bits::new(self.bit_width(), bit_plane_0, bit_plane_1)
    }

    copy_from_impl!();

    #[inline]
    pub(crate) fn into_shared(self) -> LogicStateRef<'a> {
        LogicStateRef {
            repr: LogicStateRepr::Ptr {
                bit_width: self.bit_width,
                bit_plane_0: self.bit_plane_0,
                bit_plane_1: self.bit_plane_1,
            },
            _borrow: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn as_shared<'b>(&'b self) -> LogicStateRef<'b> {
        LogicStateRef {
            repr: LogicStateRepr::Ptr {
                bit_width: self.bit_width,
                bit_plane_0: self.bit_plane_0,
                bit_plane_1: self.bit_plane_1,
            },
            _borrow: PhantomData,
        }
    }
}

impl fmt::Display for LogicStateMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.as_shared(), f)
    }
}

impl fmt::Debug for LogicStateMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.as_shared(), f)
    }
}

/// A value that can be converted into a borrowed [LogicState]
pub trait IntoLogicStateRef<'a> {
    /// Performs the conversion into a borrowed [LogicState]
    fn into_logic_state_ref(self) -> LogicStateRef<'a>;
}

impl<'a> IntoLogicStateRef<'a> for &'a LogicState {
    #[inline]
    fn into_logic_state_ref(self) -> LogicStateRef<'a> {
        self.borrow()
    }
}

impl<'a> IntoLogicStateRef<'a> for &'a InlineLogicState {
    #[inline]
    fn into_logic_state_ref(self) -> LogicStateRef<'a> {
        self.borrow()
    }
}

impl<'a> IntoLogicStateRef<'a> for LogicStateRef<'a> {
    #[inline]
    fn into_logic_state_ref(self) -> LogicStateRef<'a> {
        self
    }
}

impl<'a, 'b> IntoLogicStateRef<'b> for &'b LogicStateRef<'a>
where
    'a: 'b,
{
    #[inline]
    fn into_logic_state_ref(self) -> LogicStateRef<'b> {
        *self
    }
}

impl<'a> IntoLogicStateRef<'a> for LogicStateMut<'a> {
    #[inline]
    fn into_logic_state_ref(self) -> LogicStateRef<'a> {
        self.into_shared()
    }
}

impl<'a, 'b> IntoLogicStateRef<'b> for &'b LogicStateMut<'a>
where
    'a: 'b,
{
    #[inline]
    fn into_logic_state_ref(self) -> LogicStateRef<'b> {
        self.as_shared()
    }
}

macro_rules! eq_impl {
    ($t:ident$(<$($l:lifetime),+>)?) => {
        impl<$($($l,)+)? T> PartialEq<T> for $t$(<$($l),+>)?
        where
            for<'t> &'t T: IntoLogicStateRef<'t>,
        {
            fn eq(&self, other: &T) -> bool {
                let this = self.into_logic_state_ref();
                let other = other.into_logic_state_ref();

                match (this.repr, other.repr) {
                    (
                        LogicStateRepr::Logic0 { bit_width: width_a },
                        LogicStateRepr::Logic0 { bit_width: width_b },
                    )
                    | (
                        LogicStateRepr::Logic1 { bit_width: width_a },
                        LogicStateRepr::Logic1 { bit_width: width_b },
                    )
                    | (
                        LogicStateRepr::HighZ { bit_width: width_a },
                        LogicStateRepr::HighZ { bit_width: width_b },
                    )
                    | (
                        LogicStateRepr::Undefined { bit_width: width_a },
                        LogicStateRepr::Undefined { bit_width: width_b },
                    ) => width_a == width_b,

                    (LogicStateRepr::TwoState { .. }, _)
                    | (_, LogicStateRepr::TwoState { .. })
                    | (LogicStateRepr::FourState { .. }, _)
                    | (_, LogicStateRepr::FourState { .. })
                    | (LogicStateRepr::Ptr { .. }, _)
                    | (_, LogicStateRepr::Ptr { .. }) => {
                        let width_a = this.repr.bit_width();
                        let width_b = other.repr.bit_width();
                        if width_a != width_b {
                            return false;
                        }

                        let (bit_plane_a0, bit_plane_a1) = this.repr.bit_planes();
                        let (bit_plane_b0, bit_plane_b1) = other.repr.bit_planes();

                        let (&a0_last, a0_head) = bit_plane_a0.split_last().unwrap();
                        let (&a1_last, a1_head) = bit_plane_a1.split_last().unwrap();
                        let (&b0_last, b0_head) = bit_plane_b0.split_last().unwrap();
                        let (&b1_last, b1_head) = bit_plane_b1.split_last().unwrap();

                        let mask = 1u32.wrapping_shl(width_a.get()) - 1;
                        let mask = if mask == 0 { u32::MAX } else { mask };

                        (a0_head == b0_head)
                            && (a1_head == b1_head)
                            && ((a0_last & mask) == (b0_last & mask))
                            && ((a1_last & mask) == (b1_last & mask))
                    }

                    _ => false,
                }
            }
        }
    };
}

eq_impl!(LogicState);
eq_impl!(InlineLogicState);
eq_impl!(LogicStateRef<'a>);
eq_impl!(LogicStateMut<'a>);
