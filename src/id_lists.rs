use super::{AllocationSize, Atom, Component, SafeDivCeil, Wire};
use std::num::NonZeroU8;
use std::ops::{Index, IndexMut};
use sync_unsafe_cell::SyncUnsafeCell;

// SAFETY:
// Accessing this data is on the hot path of the simulation,
// so it is important to optimize it as much as possible.
// Therefore in release mode we turn off all bounds checks
// and assume our invariants hold. This is technically not
// safe so proper testing in debug mode is required.

#[cfg(not(debug_assertions))]
macro_rules! get_expect {
    ($s:expr, $i:expr, $err:literal $(,)?) => {
        unsafe { $s.get_unchecked($i) }
    };
}

#[cfg(debug_assertions)]
macro_rules! get_expect {
    ($s:expr, $i:expr, $err:literal $(,)?) => {
        $s.get($i).expect($err)
    };
}

#[cfg(not(debug_assertions))]
macro_rules! get_mut_expect {
    ($s:expr, $i:expr, $err:literal $(,)?) => {
        unsafe { $s.get_unchecked_mut($i) }
    };
}

#[cfg(debug_assertions)]
macro_rules! get_mut_expect {
    ($s:expr, $i:expr, $err:literal $(,)?) => {
        $s.get_mut($i).expect($err)
    };
}

#[cfg(not(debug_assertions))]
macro_rules! get_flatten_expect {
    ($s:expr, $i:expr, $err:literal $(,)?) => {
        unsafe { $s.get_unchecked($i).unwrap_unchecked() }
    };
}

#[cfg(debug_assertions)]
macro_rules! get_flatten_expect {
    ($s:expr, $i:expr, $err:literal $(,)?) => {
        $s.get($i).copied().flatten().expect($err)
    };
}

const INVALID_ID: u32 = u32::MAX;

pub(crate) trait Id: Copy + Eq {
    /// An invalid ID
    const INVALID: Self;

    /// Checks whether this ID is invalid
    #[inline]
    fn is_invalid(self) -> bool {
        self == Self::INVALID
    }

    fn to_u32(self) -> u32;
    fn from_u32(val: u32) -> Self;
}

macro_rules! def_id_type {
    ($(#[$attr:meta])* $id_vis:vis $id_name:ident) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        $id_vis struct $id_name(u32);
        assert_eq_size!($id_name, u32);
        assert_eq_align!($id_name, u32);

        impl Default for $id_name {
            #[inline]
            fn default() -> Self {
                Self::INVALID
            }
        }

        impl Id for $id_name {
            const INVALID: Self = Self(INVALID_ID);

            #[inline]
            fn to_u32(self) -> u32 {
                self.0
            }

            #[inline]
            fn from_u32(val: u32) -> Self {
                Self(val)
            }
        }
    };
}

macro_rules! def_id_list {
    ($list_name:ident<$id_name:ident, $t:ty>) => {
        #[repr(transparent)]
        pub(crate) struct $list_name(Vec<SyncUnsafeCell<$t>>);

        #[allow(dead_code)]
        impl $list_name {
            #[inline]
            pub(crate) const fn new() -> Self {
                Self(Vec::new())
            }

            #[inline]
            pub(crate) fn alloc_size(&self) -> AllocationSize {
                AllocationSize(self.0.capacity() * std::mem::size_of::<$t>())
            }

            #[inline]
            pub(crate) fn push(&mut self, item: $t) -> Option<$id_name> {
                let current_len = u32::try_from(self.0.len())
                    .ok()
                    .filter(|&len| len < INVALID_ID)?;

                self.0.push(SyncUnsafeCell::new(item));
                Some($id_name(current_len))
            }

            #[inline]
            pub(crate) fn shrink_to_fit(&mut self) {
                self.0.shrink_to_fit()
            }

            /// SAFETY: caller must ensure there are no other references to the item with this ID
            #[inline]
            pub(crate) unsafe fn get_unsafe(&self, id: $id_name) -> &mut $t {
                unsafe { &mut *self.0[id.0 as usize].get() }
            }

            #[inline]
            pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut $t> {
                self.0.iter_mut().map(SyncUnsafeCell::get_mut)
            }

            #[inline]
            pub(crate) fn ids(&self) -> impl Iterator<Item = $id_name> + '_ {
                self.0.iter().enumerate().map(|(i, _)| $id_name(i as u32))
            }
        }

        impl Index<$id_name> for $list_name {
            type Output = $t;

            #[inline]
            fn index(&self, id: $id_name) -> &Self::Output {
                unsafe {
                    // SAFETY: since we have a shared reference to `self`, no mutable references exist
                    &*self.0[id.0 as usize].get()
                }
            }
        }

        impl IndexMut<$id_name> for $list_name {
            #[inline]
            fn index_mut(&mut self, id: $id_name) -> &mut Self::Output {
                self.0[id.0 as usize].get_mut()
            }
        }

        impl Index<($id_name, u8)> for $list_name {
            type Output = $t;

            #[inline]
            fn index(&self, (id, offset): ($id_name, u8)) -> &Self::Output {
                unsafe {
                    // SAFETY: since we have a shared reference to `self`, no mutable references exist
                    &*self.0[(id.0 as usize) + (offset as usize)].get()
                }
            }
        }

        impl IndexMut<($id_name, u8)> for $list_name {
            #[inline]
            fn index_mut(&mut self, (id, offset): ($id_name, u8)) -> &mut Self::Output {
                self.0[(id.0 as usize) + (offset as usize)].get_mut()
            }
        }
    };
}

def_id_type!(
    /// A unique identifier for a wire inside a simulation
    pub WireId
);

def_id_list!(WireList<WireId, Wire>);

impl WireList {
    #[inline]
    pub(crate) fn wire_count(&self) -> usize {
        self.0.len()
    }
}

def_id_type!(
    /// A unique identifier for a component inside a simulation
    pub ComponentId
);

def_id_list!(ComponentList<ComponentId, Component>);

impl ComponentList {
    pub(crate) fn component_counts(&self) -> (usize, usize) {
        let mut small_count = 0;
        let mut large_count = 0;

        for comp in self.0.iter() {
            let comp = unsafe {
                // SAFETY: since we have a shared reference to `self`, no mutable references exist
                &*comp.get()
            };

            match comp {
                Component::Small { .. } => small_count += 1,
                Component::Large { .. } => large_count += 1,
            }
        }

        (small_count, large_count)
    }

    pub(crate) fn large_alloc_size(&self) -> AllocationSize {
        self.0
            .iter()
            .map(|comp| {
                unsafe {
                    // SAFETY: since we have a shared reference to `self`, no mutable references exist
                    &*comp.get()
                }
            })
            .map(Component::alloc_size)
            .sum()
    }
}

/// The same requirements as casting `&SyncUnsafeCell<T>` to `&T` apply.
#[inline]
unsafe fn cell_slice_to_ref<'a, T>(slice: &'a [SyncUnsafeCell<T>]) -> &'a [T] {
    let (ptr, len) = (slice.as_ptr(), slice.len());
    let ptr = ptr.cast::<T>();
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

/// The same requirements as casting `&SyncUnsafeCell<T>` to `&mut T` apply.
#[inline]
unsafe fn cell_slice_to_mut<'a, T>(slice: &'a [SyncUnsafeCell<T>]) -> &'a mut [T] {
    let (ptr, len) = (slice.as_ptr(), slice.len());
    let ptr = ptr.cast::<T>().cast_mut();
    unsafe { std::slice::from_raw_parts_mut(ptr, len) }
}

/*
           STORAGE FORMAT

    ID 0    ID 1            ID 3
     |       |-------|       |
     v       v       v       v
 ---------------------------------
 | Width | Width |   -   | Width |
 ---------------------------------
 | Atom  | Atom  | Atom  | Atom  |
 ---------------------------------

*/

def_id_type!(pub(crate) WireStateId);

pub(crate) struct WireStateList {
    widths: Vec<Option<NonZeroU8>>,
    drives: Vec<Atom>,
    states: Vec<SyncUnsafeCell<Atom>>,
}

impl WireStateList {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            widths: Vec::new(),
            drives: Vec::new(),
            states: Vec::new(),
        }
    }

    #[inline]
    pub(crate) fn width_alloc_size(&self) -> AllocationSize {
        AllocationSize(self.widths.capacity() * std::mem::size_of::<u8>())
    }

    #[inline]
    pub(crate) fn drive_alloc_size(&self) -> AllocationSize {
        AllocationSize(self.drives.capacity() * std::mem::size_of::<Atom>())
    }

    #[inline]
    pub(crate) fn state_alloc_size(&self) -> AllocationSize {
        AllocationSize(self.states.capacity() * std::mem::size_of::<Atom>())
    }

    #[inline]
    pub(crate) fn push(&mut self, width: NonZeroU8) -> Option<WireStateId> {
        debug_assert_eq!(self.widths.len(), self.drives.len());
        debug_assert_eq!(self.widths.len(), self.states.len());

        let atom_count = width.safe_div_ceil(Atom::BITS);
        let current_len = u32::try_from(self.widths.len())
            .ok()
            .filter(|&len| len < INVALID_ID)?;
        let new_len = current_len
            .checked_add(atom_count.get() as u32)
            .filter(|&len| len <= INVALID_ID)? as usize;

        self.widths.push(Some(width));
        self.widths.resize(new_len, None);
        self.drives.resize(new_len, Atom::HIGH_Z);
        self.states
            .resize_with(new_len, || SyncUnsafeCell::new(Atom::HIGH_Z));

        Some(WireStateId(current_len))
    }

    #[inline]
    pub(crate) fn shrink_to_fit(&mut self) {
        self.widths.shrink_to_fit();
        self.drives.shrink_to_fit();
        self.states.shrink_to_fit();
    }

    #[inline]
    pub(crate) fn clear_states(&mut self) {
        for state in self.states.iter_mut() {
            *state.get_mut() = Atom::HIGH_Z;
        }
    }

    #[inline]
    pub(crate) fn get_width(&self, id: WireStateId) -> NonZeroU8 {
        get_flatten_expect!(self.widths, id.0 as usize, "invalid wire state ID")
    }

    #[inline]
    pub(crate) fn get_drive(&self, id: WireStateId) -> &[Atom] {
        let width = self.get_width(id);
        let count = width.safe_div_ceil(Atom::BITS);
        let start = id.0 as usize;
        let end = start + (count.get() as usize);

        get_expect!(self.drives, start..end, "invalid wire state data layout")
    }

    #[inline]
    pub(crate) fn get_drive_mut(&mut self, id: WireStateId) -> &mut [Atom] {
        let width = self.get_width(id);
        let count = width.safe_div_ceil(Atom::BITS);
        let start = id.0 as usize;
        let end = start + (count.get() as usize);

        get_mut_expect!(self.drives, start..end, "invalid wire state data layout")
    }

    #[inline]
    pub(crate) fn get_state(&self, id: WireStateId) -> &[Atom] {
        let width = self.get_width(id);
        let count = width.safe_div_ceil(Atom::BITS);
        let start = id.0 as usize;
        let end = start + (count.get() as usize);

        let state = get_expect!(self.states, start..end, "invalid wire state data layout");

        unsafe {
            // SAFETY:
            // We have a shared reference to self, so no mutable reference to this item exists
            // as long as the safety requirement on `get_data_unsafe` has been upheld.
            cell_slice_to_ref(state)
        }
    }

    /// SAFETY: caller must guarantee there are no other references to the item with this ID and offset
    #[inline]
    pub(crate) unsafe fn get_data_unsafe(
        &self,
        id: WireStateId,
    ) -> (NonZeroU8, &[Atom], &mut [Atom]) {
        let width = self.get_width(id);
        let count = width.safe_div_ceil(Atom::BITS);
        let start = id.0 as usize;
        let end = start + (count.get() as usize);

        let drive = get_expect!(self.drives, start..end, "invalid wire state data layout");
        let state = get_expect!(self.states, start..end, "invalid wire state data layout");
        let state = unsafe { cell_slice_to_mut(state) };

        (width, drive, state)
    }
}

def_id_type!(pub(crate) OutputStateId);

pub(crate) struct OutputStateList {
    widths: Vec<Option<NonZeroU8>>,
    states: Vec<SyncUnsafeCell<Atom>>,
}

impl OutputStateList {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            widths: Vec::new(),
            states: Vec::new(),
        }
    }

    #[inline]
    pub(crate) fn width_alloc_size(&self) -> AllocationSize {
        AllocationSize(self.widths.capacity() * std::mem::size_of::<u8>())
    }

    #[inline]
    pub(crate) fn state_alloc_size(&self) -> AllocationSize {
        AllocationSize(self.states.capacity() * std::mem::size_of::<Atom>())
    }

    #[inline]
    pub(crate) fn push(&mut self, width: NonZeroU8) -> Option<OutputStateId> {
        debug_assert_eq!(self.widths.len(), self.states.len());

        let atom_count = width.safe_div_ceil(Atom::BITS);
        let current_len = u32::try_from(self.widths.len())
            .ok()
            .filter(|&len| len < INVALID_ID)?;
        let new_len = current_len
            .checked_add(atom_count.get() as u32)
            .filter(|&len| len <= INVALID_ID)? as usize;

        self.widths.push(Some(width));
        self.widths.resize(new_len, None);
        self.states
            .resize_with(new_len, || SyncUnsafeCell::new(Atom::HIGH_Z));

        Some(OutputStateId(current_len))
    }

    #[inline]
    pub(crate) fn shrink_to_fit(&mut self) {
        self.widths.shrink_to_fit();
        self.states.shrink_to_fit();
    }

    #[inline]
    pub(crate) fn clear_states(&mut self) {
        for state in self.states.iter_mut() {
            *state.get_mut() = Atom::HIGH_Z;
        }
    }

    #[inline]
    pub(crate) fn get_width(&self, id: OutputStateId) -> NonZeroU8 {
        get_flatten_expect!(self.widths, id.0 as usize, "invalid output state ID")
    }

    #[inline]
    pub(crate) fn get_state(&self, id: OutputStateId) -> &[Atom] {
        let width = self.get_width(id);
        let count = width.safe_div_ceil(Atom::BITS);
        let start = id.0 as usize;
        let end = start + (count.get() as usize);

        let state = get_expect!(self.states, start..end, "invalid output state data layout");

        unsafe {
            // SAFETY:
            // We have a shared reference to self, so no mutable reference to this item exists
            // as long as the safety requirement on `get_data_unsafe` has been upheld.
            cell_slice_to_ref(state)
        }
    }

    /// SAFETY: caller must guarantee there are no other references to all items within this slice
    #[inline]
    pub(crate) unsafe fn get_slice_unsafe<'this>(
        &'this self,
        base: OutputStateId,
        atom_count: u16,
    ) -> OutputStateSlice<'this> {
        if atom_count == 0 {
            return OutputStateSlice {
                offset: 0,
                widths: &[],
                states: &mut [],
            };
        }

        debug_assert!(
            self.widths
                .get(base.0 as usize)
                .copied()
                .flatten()
                .is_some(),
            "invalid output state ID",
        );

        let start = base.0 as usize;
        let end = start + (atom_count as usize);

        debug_assert!(
            (end == self.widths.len()) || self.widths.get(end).copied().flatten().is_some(),
            "invalid output state slice length",
        );

        let widths = get_expect!(self.widths, start..end, "invalid output state data layout");
        let states = get_expect!(self.states, start..end, "invalid output state data layout");
        let states = unsafe { cell_slice_to_mut(states) };

        OutputStateSlice {
            offset: base.0 as usize,
            widths,
            states,
        }
    }
}

pub(crate) struct OutputStateSlice<'a> {
    offset: usize,
    widths: &'a [Option<NonZeroU8>],
    states: &'a mut [Atom],
}

impl OutputStateSlice<'_> {
    #[inline]
    pub(crate) fn get_width(&self, id: OutputStateId) -> NonZeroU8 {
        get_flatten_expect!(
            self.widths,
            (id.0 as usize) - self.offset,
            "invalid output state ID",
        )
    }

    #[inline]
    pub(crate) fn get_data(&mut self, id: OutputStateId) -> (NonZeroU8, &mut [Atom]) {
        let width = self.get_width(id);
        let count = width.safe_div_ceil(Atom::BITS);
        let start = (id.0 as usize) - self.offset;
        let end = start + (count.get() as usize);

        let state = get_mut_expect!(self.states, start..end, "invalid output state data layout");

        (width, state)
    }
}
