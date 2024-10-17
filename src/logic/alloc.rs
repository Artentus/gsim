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
    word_start: u32,
    word_end: u32,
    bit_planes: [BitPlanesView; N],
    _borrow: PhantomData<&'a LogicStateAllocator<T, N>>,
}

unsafe impl<T: Id, const N: usize> Send for LogicStateView<'_, T, N> {}
unsafe impl<T: Id, const N: usize> Sync for LogicStateView<'_, T, N> {}

pub(crate) struct LogicStateViewMut<'a, T: Id, const N: usize> {
    word_start: u32,
    word_end: u32,
    bit_planes: [BitPlanesView; N],
    _borrow: PhantomData<&'a mut LogicStateAllocator<T, N>>,
}

unsafe impl<T: Id, const N: usize> Send for LogicStateViewMut<'_, T, N> {}

macro_rules! view_get_unchecked_body {
    (|$this:ident, $id:ident, $bit_width:ident| -> $logic_state:ident) => {{
        debug_assert!($id.to_bits() >= $this.word_start);
        debug_assert!(($id.to_bits() + $bit_width.word_len()) <= $this.word_end);

        unsafe {
            // SAFETY:
            //   The pointers are valid as long as `&self` is valid, which is
            //   expressed through the lifetime bounds of the function.
            $this.bit_planes.map(|bit_planes| {
                $logic_state::new_ptr(
                    $bit_width,
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
    pub(crate) unsafe fn get_unchecked(
        &self,
        id: T,
        bit_width: BitWidth,
    ) -> [LogicStateRef<'_>; N] {
        view_get_unchecked_body!(|self, id, bit_width| -> LogicStateRef)
    }

    #[inline]
    pub(crate) fn get(&self, id: T, bit_width: BitWidth) -> Option<[LogicStateRef<'_>; N]> {
        if (id.to_bits() < self.word_start)
            || ((id.to_bits() + bit_width.word_len()) > self.word_end)
        {
            return None;
        }

        unsafe {
            // SAFETY: we just checked
            Some(self.get_unchecked(id, bit_width))
        }
    }
}

impl<T: Id, const N: usize> LogicStateViewMut<'_, T, N> {
    /// SAFETY: caller must ensure the ID is valid.
    #[inline]
    pub(crate) unsafe fn get_unchecked(
        &self,
        id: T,
        bit_width: BitWidth,
    ) -> [LogicStateRef<'_>; N] {
        view_get_unchecked_body!(|self, id, bit_width| -> LogicStateRef)
    }

    /// SAFETY: caller must ensure the ID is valid.
    #[inline]
    pub(crate) unsafe fn get_unchecked_mut(
        &mut self,
        id: T,
        bit_width: BitWidth,
    ) -> [LogicStateMut<'_>; N] {
        view_get_unchecked_body!(|self, id, bit_width| -> LogicStateMut)
    }

    #[inline]
    pub(crate) fn get(&self, id: T, bit_width: BitWidth) -> Option<[LogicStateRef<'_>; N]> {
        if (id.to_bits() < self.word_start)
            || ((id.to_bits() + bit_width.word_len()) > self.word_end)
        {
            return None;
        }

        unsafe {
            // SAFETY: we just checked
            Some(self.get_unchecked(id, bit_width))
        }
    }

    #[inline]
    pub(crate) fn get_mut(&mut self, id: T, bit_width: BitWidth) -> Option<[LogicStateMut<'_>; N]> {
        if (id.to_bits() < self.word_start)
            || ((id.to_bits() + bit_width.word_len()) > self.word_end)
        {
            return None;
        }

        unsafe {
            // SAFETY: we just checked
            Some(self.get_unchecked_mut(id, bit_width))
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
            bit_planes: [const { BitPlanes::new() }; N],
            _t: PhantomData,
        }
    }

    #[inline]
    fn reserve(&mut self, new_word_len: u32) -> Result<(), OutOfMemoryError> {
        if new_word_len > self.word_cap {
            let new_word_cap = new_word_len.saturating_mul(2);

            unsafe {
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
            word_start: 0,
            word_end: self.word_len,
            bit_planes: self.bit_planes.map(BitPlanes::view),
            _borrow: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn view_mut(&mut self) -> LogicStateViewMut<T, N> {
        assert!(self.word_len > 0);

        LogicStateViewMut {
            word_start: 0,
            word_end: self.word_len,
            bit_planes: self.bit_planes.map(BitPlanes::view),
            _borrow: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn range(&self, start: T, end: T, end_width: BitWidth) -> LogicStateView<T, N> {
        assert!(start.to_bits() <= end.to_bits());
        assert!((end.to_bits() + end_width.word_len()) <= self.word_len);

        LogicStateView {
            word_start: start.to_bits(),
            word_end: end.to_bits() + end_width.word_len(),
            bit_planes: self.bit_planes.map(BitPlanes::view),
            _borrow: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn range_mut(
        &mut self,
        start: T,
        end: T,
        end_width: BitWidth,
    ) -> LogicStateViewMut<T, N> {
        assert!(start.to_bits() <= end.to_bits());
        assert!((end.to_bits() + end_width.word_len()) <= self.word_len);

        LogicStateViewMut {
            word_start: start.to_bits(),
            word_end: end.to_bits() + end_width.word_len(),
            bit_planes: self.bit_planes.map(BitPlanes::view),
            _borrow: PhantomData,
        }
    }

    /// SAFETY: caller must ensure this range is only borrowed once at a time.
    #[inline]
    pub(crate) unsafe fn range_unsafe(
        &self,
        start: T,
        end: T,
        end_width: BitWidth,
    ) -> LogicStateViewMut<T, N> {
        assert!(start.to_bits() <= end.to_bits());
        assert!((end.to_bits() + end_width.word_len()) <= self.word_len);

        LogicStateViewMut {
            word_start: start.to_bits(),
            word_end: end.to_bits() + end_width.word_len(),
            bit_planes: self.bit_planes.map(BitPlanes::view),
            _borrow: PhantomData,
        }
    }
}

macro_rules! alloc_get_unchecked_body {
    (|$this:ident, $id:ident, $bit_width:ident| -> $logic_state:ident) => {{
        debug_assert!(($id.to_bits() + $bit_width.word_len()) <= $this.word_len);

        unsafe {
            // SAFETY:
            //   The pointers are valid as long as `&self` is valid, which is
            //   expressed through the lifetime bounds of the function.
            $this.bit_planes.map(|bit_planes| {
                $logic_state::new_ptr(
                    $bit_width,
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
    pub(crate) unsafe fn get_unchecked(
        &self,
        id: T,
        bit_width: BitWidth,
    ) -> [LogicStateRef<'_>; N] {
        alloc_get_unchecked_body!(|self, id, bit_width| -> LogicStateRef)
    }

    /// SAFETY: caller must ensure the ID is valid.
    #[inline]
    pub(crate) unsafe fn get_unchecked_mut(
        &mut self,
        id: T,
        bit_width: BitWidth,
    ) -> [LogicStateMut<'_>; N] {
        alloc_get_unchecked_body!(|self, id, bit_width| -> LogicStateMut)
    }

    /// SAFETY: caller must ensure the ID is valid and only borrowed once at a time.
    #[inline]
    pub(crate) unsafe fn get_unchecked_unsafe(
        &self,
        id: T,
        bit_width: BitWidth,
    ) -> [LogicStateMut<'_>; N] {
        alloc_get_unchecked_body!(|self, id, bit_width| -> LogicStateMut)
    }

    #[inline]
    pub(crate) fn get(&self, id: T, bit_width: BitWidth) -> Option<[LogicStateRef<'_>; N]> {
        if (id.to_bits() + bit_width.word_len()) > self.word_len {
            return None;
        }

        unsafe {
            // SAFETY: we just checked
            Some(self.get_unchecked(id, bit_width))
        }
    }

    #[inline]
    pub(crate) fn get_mut(&mut self, id: T, bit_width: BitWidth) -> Option<[LogicStateMut<'_>; N]> {
        if (id.to_bits() + bit_width.word_len()) > self.word_len {
            return None;
        }

        unsafe {
            // SAFETY: we just checked
            Some(self.get_unchecked_mut(id, bit_width))
        }
    }

    /// SAFETY: caller must ensure the ID is only borrowed once at a time.
    #[inline]
    pub(crate) fn get_unsafe(&self, id: T, bit_width: BitWidth) -> Option<[LogicStateMut<'_>; N]> {
        if (id.to_bits() + bit_width.word_len()) > self.word_len {
            return None;
        }

        unsafe {
            // SAFETY: we just checked
            Some(self.get_unchecked_unsafe(id, bit_width))
        }
    }
}

pub(crate) type WireStateView<'a> = LogicStateView<'a, WireStateId, 2>;
pub(crate) type WireStateViewMut<'a> = LogicStateViewMut<'a, WireStateId, 2>;
pub(crate) type WireStateAllocator = LogicStateAllocator<WireStateId, 2>;

pub(crate) type OutputStateView<'a> = LogicStateView<'a, OutputStateId, 1>;
pub(crate) type OutputStateViewMut<'a> = LogicStateViewMut<'a, OutputStateId, 1>;
pub(crate) type OutputStateAllocator = LogicStateAllocator<OutputStateId, 1>;
