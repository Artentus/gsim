#![allow(dead_code)]

mod alloc;
pub(crate) use alloc::*;

mod state;
pub use state::*;

use std::fmt;

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

    #[inline]
    pub(crate) const fn clog2(self) -> Option<Self> {
        Self::new(self.get().next_power_of_two().ilog2())
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
