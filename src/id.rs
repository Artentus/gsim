use crate::{AllocationSize, Component, Wire};
use std::cmp::Ordering;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ptr::NonNull;
use sync_unsafe_cell::SyncUnsafeCell;

const_assert!(usize::BITS >= u32::BITS);

/// An ID type
pub trait Id: Copy + Eq + std::hash::Hash {
    fn to_bits(self) -> u32;
    fn from_bits(val: u32) -> Self;
}

macro_rules! def_id_type {
    ($(#[$attr:meta])* $id_vis:vis $id_name:ident) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        $id_vis struct $id_name(u32);
        assert_eq_size!($id_name, u32);
        assert_eq_align!($id_name, u32);

        impl crate::id::Id for $id_name {
            #[inline]
            fn to_bits(self) -> u32 {
                self.0
            }

            #[inline]
            fn from_bits(val: u32) -> Self {
                Self(val)
            }
        }
    };
}
pub(crate) use def_id_type;

const INLINE_CAP: usize = 1 + size_of::<usize>() / size_of::<u32>();

#[repr(C)]
struct InlineIdVec<T: Id> {
    len: u32,
    data: [MaybeUninit<T>; INLINE_CAP],
}

impl<T: Id> InlineIdVec<T> {
    fn as_slice(&self) -> &[T] {
        let ptr: *const T = self.data.as_slice().as_ptr().cast();
        unsafe {
            // SAFETY: by invariant the first `len` elements are initialized
            std::slice::from_raw_parts(ptr, self.len as usize)
        }
    }
}

#[repr(C)]
struct HeapIdVec<T: Id> {
    len: u32,
    cap: u32,
    data: NonNull<T>,
}

impl<T: Id> HeapIdVec<T> {
    #[inline]
    fn as_slice(&self) -> &[T] {
        unsafe {
            // SAFETY: by invariant the first `len` elements are initialized
            std::slice::from_raw_parts(self.data.as_ptr().cast_const(), self.len as usize)
        }
    }
}

fn alloc_new<T: Id>(existing: &[T], new: T) -> HeapIdVec<T> {
    let mut vec = Vec::<T>::with_capacity(existing.len().saturating_mul(2));
    vec.extend_from_slice(existing);
    vec.push(new);

    let len = vec.len();
    assert!(len <= (u32::MAX as usize), "capacity overflow");
    let cap = vec.capacity();
    assert!(cap <= (u32::MAX as usize), "capacity overflow");

    let data = NonNull::new(vec.as_mut_ptr()).expect("`Vec::as_mut_ptr` returned null-pointer");
    std::mem::forget(vec);

    HeapIdVec {
        len: len as u32,
        cap: cap as u32,
        data,
    }
}

#[repr(C)]
pub(crate) union IdVec<T: Id> {
    /// SAFETY: __never__ write to this field to not invalidate the data
    len: u32,
    inline: ManuallyDrop<InlineIdVec<T>>,
    heap: ManuallyDrop<HeapIdVec<T>>,
}

impl<T: Id> IdVec<T> {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            inline: ManuallyDrop::new(InlineIdVec {
                len: 0,
                data: [const { MaybeUninit::uninit() }; INLINE_CAP],
            }),
        }
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        unsafe {
            // SAFETY: len is always initialized because the filed exists at the same location in all variants
            self.len as usize
        }
    }

    /// SAFETY: the variant must be `heap` and not have been deallocated already
    unsafe fn drop_alloc(&mut self) {
        unsafe {
            let vec = Vec::from_raw_parts(
                self.heap.data.as_ptr(),
                self.heap.len as usize,
                self.heap.cap as usize,
            );
            std::mem::drop(vec);
        }
    }

    pub(crate) fn push(&mut self, id: T) {
        let len = self.len();
        assert!(len < (isize::MAX as usize));
        assert!(len < (u32::MAX as usize));

        match len.cmp(&INLINE_CAP) {
            Ordering::Less => {
                unsafe {
                    // SAFETY: when len <= INLINE_CAP, the active variant is `inline`
                    self.inline.data[len].write(id);
                    self.inline.len += 1;
                }
            }
            Ordering::Equal => {
                unsafe {
                    // SAFETY: when len == INLINE_CAP, the active variant is `inline`, but after adding it must be `heap`
                    let new_heap = alloc_new(self.inline.as_slice(), id);
                    self.heap = ManuallyDrop::new(new_heap);
                }
            }
            Ordering::Greater => {
                unsafe {
                    // SAFETY: when len > INLINE_CAP, the active variant is `heap`
                    let capacity = self.heap.cap as usize;
                    if len < capacity {
                        self.heap.data.as_ptr().add(len).write(id);
                        self.heap.len += 1;
                    } else {
                        let existing =
                            std::slice::from_raw_parts(self.heap.data.as_ptr().cast_const(), len);
                        let new_heap = alloc_new(existing, id);
                        self.drop_alloc();
                        self.heap = ManuallyDrop::new(new_heap);
                    }
                }
            }
        }
    }

    pub(crate) fn as_slice(&self) -> &[T] {
        if self.len() <= INLINE_CAP {
            unsafe {
                // SAFETY: when len <= INLINE_CAP, the active variant is `inline`
                self.inline.as_slice()
            }
        } else {
            unsafe {
                // SAFETY: when len > INLINE_CAP, the active variant is `heap`
                self.heap.as_slice()
            }
        }
    }

    #[inline]
    pub(crate) fn contains(&self, id: T) -> bool {
        self.as_slice().contains(&id)
    }

    #[inline]
    pub(crate) fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.as_slice().iter().copied()
    }
}

impl<T: Id> Drop for IdVec<T> {
    fn drop(&mut self) {
        if self.len() > INLINE_CAP {
            unsafe {
                // SAFETY: when len > INLINE_CAP, the active variant is `heap`
                self.drop_alloc();
            }
        }
    }
}

unsafe impl<T: Id> Send for IdVec<T> {}
unsafe impl<T: Id> Sync for IdVec<T> {}

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
                let current_len = u32::try_from(self.0.len()).ok()?;
                self.0.push(SyncUnsafeCell::new(item));
                Some($id_name(current_len))
            }

            #[inline]
            pub(crate) fn shrink_to_fit(&mut self) {
                self.0.shrink_to_fit()
            }

            #[inline]
            pub(crate) fn get(&self, id: $id_name) -> Option<&$t> {
                self.0.get(id.0 as usize).map(|t| unsafe {
                    // SAFETY: since we have a shared reference to `self`, no mutable references exist
                    &*t.get()
                })
            }

            #[inline]
            pub(crate) fn get_mut(&mut self, id: $id_name) -> Option<&mut $t> {
                self.0.get_mut(id.0 as usize).map(SyncUnsafeCell::get_mut)
            }

            /// SAFETY: caller must ensure there are no other references to the item with this ID
            #[allow(clippy::mut_from_ref)]
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
    };
}

def_id_list!(WireList<WireId, Wire>);

impl WireList {
    #[inline]
    pub(crate) fn wire_count(&self) -> usize {
        self.0.len()
    }
}

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
