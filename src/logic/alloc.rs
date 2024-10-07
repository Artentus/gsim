use super::{BitWidth, LogicStateMut, LogicStateRef};
use crate::id::{def_id_type, Id};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr::{self, NonNull};

#[derive(Debug, Clone, Copy)]
pub(crate) struct OutOfMemoryError;

impl From<std::collections::TryReserveError> for OutOfMemoryError {
    #[inline]
    fn from(_: std::collections::TryReserveError) -> Self {
        Self
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct RawBuffer<T>(NonNull<T>);

impl<T> RawBuffer<T> {
    #[inline]
    const fn new() -> Self {
        Self(NonNull::dangling())
    }

    /// SAFETY: caller must take care not to cause double frees.
    #[inline]
    unsafe fn dealloc(&mut self, word_len: u32, word_cap: u32) {
        if word_cap > 0 {
            unsafe {
                let _ = Vec::from_raw_parts(self.0.as_ptr(), word_len as usize, word_cap as usize);
            }
        }
    }

    #[inline]
    unsafe fn realloc(
        &mut self,
        word_len: u32,
        word_cap: u32,
        new_word_cap: u32,
    ) -> Result<(), OutOfMemoryError> {
        let mut new_plane = Vec::<T>::new();
        new_plane.try_reserve_exact(new_word_cap as usize)?;
        assert_eq!(new_plane.capacity(), new_word_cap as usize);

        let mut new_plane = ManuallyDrop::new(new_plane);
        let new_plane = NonNull::new(new_plane.as_mut_ptr()).unwrap();

        unsafe {
            // SAFETY:
            //   - The new allocation does not overlap the old one.
            //   - Both allocations are bigger than `word_len`.

            ptr::copy_nonoverlapping(self.0.as_ptr(), new_plane.as_ptr(), word_len as usize);
        }

        unsafe {
            // SAFETY:
            //   The old allocation is currently allocated, and after this function there
            //   will be no pointer to it anymore so it can never be deallocated again.

            self.dealloc(word_len, word_cap);
        }

        self.0 = new_plane;
        Ok(())
    }

    #[inline]
    unsafe fn init(&mut self, word_len: u32, word_count: u32) {
        unsafe {
            self.0
                .as_ptr()
                .add(word_len as usize)
                .write_bytes(0, word_count as usize);
        }
    }
}

def_id_type!(pub(crate) WireStateId);
def_id_type!(pub(crate) OutputStateId);

/*
              STORAGE FORMAT

     ID 0      ID 1                ID 3
      |         |---------|         |
      v         v         v         v
 -----------------------------------------
 |  Width  |  Width  |    -    |  Width  |
 -----------------------------------------
 | Plane 0 | Plane 0 | Plane 0 | Plane 0 |
 -----------------------------------------
 | Plane 1 | Plane 1 | Plane 1 | Plane 1 |
 -----------------------------------------

*/

#[derive(Clone, Copy)]
struct BitPlanesView {
    bit_plane_0: NonNull<u32>,
    bit_plane_1: NonNull<u32>,
}

pub(crate) struct LogicStateView<'a, T: Id, const N: usize> {
    first_id: u32,
    last_id: u32,
    bit_width: NonNull<BitWidth>,
    bit_planes: [BitPlanesView; N],
    _borrow: PhantomData<&'a LogicStateAllocator<T, N>>,
}

unsafe impl<T: Id, const N: usize> Send for LogicStateView<'_, T, N> {}
unsafe impl<T: Id, const N: usize> Sync for LogicStateView<'_, T, N> {}

pub(crate) struct LogicStateViewMut<'a, T: Id, const N: usize> {
    first_id: u32,
    last_id: u32,
    bit_width: NonNull<BitWidth>,
    bit_planes: [BitPlanesView; N],
    _borrow: PhantomData<&'a mut LogicStateAllocator<T, N>>,
}

unsafe impl<T: Id, const N: usize> Send for LogicStateViewMut<'_, T, N> {}

macro_rules! view_get_unchecked_body {
    (|$this:ident, $id:ident| -> $logic_state:ident) => {{
        debug_assert!($id.to_bits() >= $this.first_id);
        debug_assert!($id.to_bits() <= $this.last_id);

        unsafe {
            // SAFETY: the bounds check above ensures this is a read of valid memory.
            let bit_width = $this.bit_width.as_ptr().add($id.to_bits() as usize).read();

            // SAFETY:
            //   The pointers are valid as long as `&self` is valid, which is
            //   expressed through the lifetime bounds of the function.
            $this.bit_planes.map(|bit_planes| {
                $logic_state::new_ptr(
                    bit_width,
                    bit_planes.bit_plane_0.add($id.to_bits() as usize),
                    bit_planes.bit_plane_1.add($id.to_bits() as usize),
                )
            })
        }
    }};
}

impl<T: Id, const N: usize> LogicStateView<'_, T, N> {
    /// SAFETY: caller must ensure the ID is valid.
    #[inline]
    pub(crate) unsafe fn get_unchecked(&self, id: T) -> [LogicStateRef<'_>; N] {
        view_get_unchecked_body!(|self, id| -> LogicStateRef)
    }

    #[inline]
    pub(crate) fn get(&self, id: T) -> Option<[LogicStateRef<'_>; N]> {
        if (id.to_bits() < self.first_id) || (id.to_bits() > self.last_id) {
            return None;
        }

        unsafe {
            // SAFETY: we just checked
            Some(self.get_unchecked(id))
        }
    }
}

impl<T: Id, const N: usize> LogicStateViewMut<'_, T, N> {
    /// SAFETY: caller must ensure the ID is valid.
    #[inline]
    pub(crate) unsafe fn get_unchecked(&self, id: T) -> [LogicStateRef<'_>; N] {
        view_get_unchecked_body!(|self, id| -> LogicStateRef)
    }

    /// SAFETY: caller must ensure the ID is valid.
    #[inline]
    pub(crate) unsafe fn get_unchecked_mut(&mut self, id: T) -> [LogicStateMut<'_>; N] {
        view_get_unchecked_body!(|self, id| -> LogicStateMut)
    }

    #[inline]
    pub(crate) fn get(&self, id: T) -> Option<[LogicStateRef<'_>; N]> {
        if (id.to_bits() < self.first_id) || (id.to_bits() > self.last_id) {
            return None;
        }

        unsafe {
            // SAFETY: we just checked
            Some(self.get_unchecked(id))
        }
    }

    #[inline]
    pub(crate) fn get_mut(&mut self, id: T) -> Option<[LogicStateMut<'_>; N]> {
        if (id.to_bits() < self.first_id) || (id.to_bits() > self.last_id) {
            return None;
        }

        unsafe {
            // SAFETY: we just checked
            Some(self.get_unchecked_mut(id))
        }
    }
}

#[derive(Clone, Copy)]
struct BitPlanes {
    bit_plane_0: RawBuffer<u32>,
    bit_plane_1: RawBuffer<u32>,
}

impl BitPlanes {
    #[inline]
    const fn new() -> Self {
        Self {
            bit_plane_0: RawBuffer::new(),
            bit_plane_1: RawBuffer::new(),
        }
    }

    #[inline]
    fn view(self) -> BitPlanesView {
        BitPlanesView {
            bit_plane_0: self.bit_plane_0.0,
            bit_plane_1: self.bit_plane_1.0,
        }
    }
}

pub(crate) struct LogicStateAllocator<T: Id, const N: usize> {
    word_len: u32,
    word_cap: u32,
    bit_width: RawBuffer<BitWidth>,
    bit_planes: [BitPlanes; N],
    _t: PhantomData<fn(T) -> T>,
}

unsafe impl<T: Id, const N: usize> Send for LogicStateAllocator<T, N> {}
unsafe impl<T: Id, const N: usize> Sync for LogicStateAllocator<T, N> {}

impl<T: Id, const N: usize> LogicStateAllocator<T, N> {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            word_len: 0,
            word_cap: 0,
            bit_width: RawBuffer::new(),
            bit_planes: [const { BitPlanes::new() }; N],
            _t: PhantomData,
        }
    }

    #[inline]
    fn reserve(&mut self, new_word_len: u32) -> Result<(), OutOfMemoryError> {
        if new_word_len > self.word_cap {
            let new_word_cap = new_word_len.saturating_mul(2);

            unsafe {
                self.bit_width
                    .realloc(self.word_len, self.word_cap, new_word_cap);

                for bit_planes in &mut self.bit_planes {
                    bit_planes
                        .bit_plane_0
                        .realloc(self.word_len, self.word_cap, new_word_cap);

                    bit_planes
                        .bit_plane_1
                        .realloc(self.word_len, self.word_cap, new_word_cap);
                }
            }

            self.word_cap = new_word_cap;
        }

        Ok(())
    }

    pub(crate) fn alloc(&mut self, bit_width: BitWidth) -> Result<T, OutOfMemoryError> {
        let word_count = bit_width.get().div_ceil(u32::BITS);
        let new_word_len = self
            .word_len
            .checked_add(word_count)
            .ok_or(OutOfMemoryError)?;

        self.reserve(new_word_len)?;

        unsafe {
            // SAFETY:
            //   - The call to `reserve` made sure the arrays are large enough to hold `new_word_len` elements.
            //   - The memory past `self.word_len` is currently uninitialized.
            //   - `0` is a valid bit pattern for all of these types (`BitWidth` is `repr(trasparent)` of `u8`).

            // This write is to ensure calling `get` with an invalid index doesn't read from uninitialized memory.
            self.bit_width.init(self.word_len, word_count);

            self.bit_width
                .0
                .as_ptr()
                .add(self.word_len as usize)
                .write(bit_width);

            for bit_planes in &mut self.bit_planes {
                bit_planes.bit_plane_0.init(self.word_len, word_count);
                bit_planes.bit_plane_1.init(self.word_len, word_count);
            }
        }

        let id = self.word_len;
        self.word_len = new_word_len;
        Ok(T::from_bits(id))
    }
}

impl<T: Id, const N: usize> Drop for LogicStateAllocator<T, N> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.bit_width.dealloc(self.word_len, self.word_cap);
            for bit_planes in &mut self.bit_planes {
                bit_planes.bit_plane_0.dealloc(self.word_len, self.word_cap);
                bit_planes.bit_plane_1.dealloc(self.word_len, self.word_cap);
            }
        }
    }
}

impl<T: Id, const N: usize> LogicStateAllocator<T, N> {
    #[inline]
    pub(crate) fn view(&self) -> LogicStateView<T, N> {
        assert!(self.word_len > 0);

        LogicStateView {
            first_id: 0,
            last_id: self.word_len - 1,
            bit_width: self.bit_width.0,
            bit_planes: self.bit_planes.map(BitPlanes::view),
            _borrow: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn view_mut(&mut self) -> LogicStateViewMut<T, N> {
        assert!(self.word_len > 0);

        LogicStateViewMut {
            first_id: 0,
            last_id: self.word_len - 1,
            bit_width: self.bit_width.0,
            bit_planes: self.bit_planes.map(BitPlanes::view),
            _borrow: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn range(&self, start: T, end: T) -> LogicStateView<T, N> {
        assert!(self.word_len > 0);
        assert!(start.to_bits() <= end.to_bits());
        assert!(end.to_bits() < self.word_len);

        LogicStateView {
            first_id: start.to_bits(),
            last_id: end.to_bits(),
            bit_width: self.bit_width.0,
            bit_planes: self.bit_planes.map(BitPlanes::view),
            _borrow: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn range_mut(&mut self, start: T, end: T) -> LogicStateViewMut<T, N> {
        assert!(self.word_len > 0);
        assert!(start.to_bits() <= end.to_bits());
        assert!(end.to_bits() < self.word_len);

        LogicStateViewMut {
            first_id: start.to_bits(),
            last_id: end.to_bits(),
            bit_width: self.bit_width.0,
            bit_planes: self.bit_planes.map(BitPlanes::view),
            _borrow: PhantomData,
        }
    }

    /// SAFETY: caller must ensure this range is only borrowed once at a time.
    #[inline]
    pub(crate) unsafe fn range_unsafe(&self, start: T, end: T) -> LogicStateViewMut<T, N> {
        assert!(self.word_len > 0);
        assert!(start.to_bits() <= end.to_bits());
        assert!(end.to_bits() < self.word_len);

        LogicStateViewMut {
            first_id: start.to_bits(),
            last_id: end.to_bits(),
            bit_width: self.bit_width.0,
            bit_planes: self.bit_planes.map(BitPlanes::view),
            _borrow: PhantomData,
        }
    }
}

macro_rules! alloc_get_unchecked_body {
    (|$this:ident, $id:ident| -> $logic_state:ident) => {{
        debug_assert!($id.to_bits() < $this.word_len);

        unsafe {
            // SAFETY: the bounds check above ensures this is a read of valid memory.
            let bit_width = $this
                .bit_width
                .0
                .as_ptr()
                .add($id.to_bits() as usize)
                .read();

            // SAFETY:
            //   The pointers are valid as long as `&self` is valid, which is
            //   expressed through the lifetime bounds of the function.
            $this.bit_planes.map(|bit_planes| {
                $logic_state::new_ptr(
                    bit_width,
                    bit_planes.bit_plane_0.0.add($id.to_bits() as usize),
                    bit_planes.bit_plane_1.0.add($id.to_bits() as usize),
                )
            })
        }
    }};
}

impl<T: Id, const N: usize> LogicStateAllocator<T, N> {
    /// SAFETY: caller must ensure the ID is valid.
    #[inline]
    pub(crate) unsafe fn get_unchecked(&self, id: T) -> [LogicStateRef<'_>; N] {
        alloc_get_unchecked_body!(|self, id| -> LogicStateRef)
    }

    /// SAFETY: caller must ensure the ID is valid.
    #[inline]
    pub(crate) unsafe fn get_unchecked_mut(&mut self, id: T) -> [LogicStateMut<'_>; N] {
        alloc_get_unchecked_body!(|self, id| -> LogicStateMut)
    }

    /// SAFETY: caller must ensure the ID is valid and only borrowed once at a time.
    #[inline]
    pub(crate) unsafe fn get_unchecked_unsafe(&self, id: T) -> [LogicStateMut<'_>; N] {
        alloc_get_unchecked_body!(|self, id| -> LogicStateMut)
    }

    #[inline]
    pub(crate) fn get(&self, id: T) -> Option<[LogicStateRef<'_>; N]> {
        if id.to_bits() >= self.word_len {
            return None;
        }

        unsafe {
            // SAFETY: we just checked
            Some(self.get_unchecked(id))
        }
    }

    #[inline]
    pub(crate) fn get_mut(&mut self, id: T) -> Option<[LogicStateMut<'_>; N]> {
        if id.to_bits() >= self.word_len {
            return None;
        }

        unsafe {
            // SAFETY: we just checked
            Some(self.get_unchecked_mut(id))
        }
    }

    /// SAFETY: caller must ensure the ID is only borrowed once at a time.
    #[inline]
    pub(crate) fn get_unsafe(&self, id: T) -> Option<[LogicStateMut<'_>; N]> {
        if id.to_bits() >= self.word_len {
            return None;
        }

        unsafe {
            // SAFETY: we just checked
            Some(self.get_unchecked_unsafe(id))
        }
    }
}

pub(crate) type WireStateView<'a> = LogicStateView<'a, WireStateId, 2>;
pub(crate) type WireStateViewMut<'a> = LogicStateViewMut<'a, WireStateId, 2>;
pub(crate) type WireStateAllocator = LogicStateAllocator<WireStateId, 2>;

pub(crate) type OutputStateView<'a> = LogicStateView<'a, OutputStateId, 1>;
pub(crate) type OutputStateViewMut<'a> = LogicStateViewMut<'a, OutputStateId, 1>;
pub(crate) type OutputStateAllocator = LogicStateAllocator<OutputStateId, 1>;
