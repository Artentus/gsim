use crate::id_lists::{Id, IdInternal};
use std::mem::ManuallyDrop;
use std::ptr::NonNull;

const INLINE_CAPACITY: u32 = 3;

#[repr(C)]
struct InlineIdVec<T: Id> {
    len: u32,
    data: [T; INLINE_CAPACITY as usize],
}

#[repr(C)]
struct HeapIdVec<T: Id> {
    len: u32,
    capacity: u32,
    data: NonNull<T>,
}

fn alloc_new<T: Id>(existing: &[T], new: T) -> HeapIdVec<T> {
    let mut vec = Vec::<T>::with_capacity(existing.len().saturating_mul(2));
    vec.extend_from_slice(existing);
    vec.push(new);

    let len = vec.len();
    assert!(len <= (u32::MAX as usize));
    let capacity = vec.capacity();
    assert!(capacity <= (u32::MAX as usize));

    let data = NonNull::new(vec.as_mut_ptr()).expect("`Vec::as_mut_ptr` returned null-pointer");
    std::mem::forget(vec);

    HeapIdVec {
        len: len as u32,
        capacity: capacity as u32,
        data,
    }
}

#[repr(C)]
pub(crate) union IdVec<T: IdInternal> {
    /// SAFETY: __never__ write to this field to not invalidate the data
    len: u32,
    inline: ManuallyDrop<InlineIdVec<T>>,
    heap: ManuallyDrop<HeapIdVec<T>>,
}

impl<T: IdInternal> IdVec<T> {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            inline: ManuallyDrop::new(InlineIdVec {
                len: 0,
                data: [T::INVALID; INLINE_CAPACITY as usize],
            }),
        }
    }

    #[inline]
    pub(crate) fn len(&self) -> u32 {
        unsafe {
            // SAFETY: len is always initialized because the filed exists at the same location in all variants
            self.len
        }
    }

    /// SAFETY: the variant must be `heap` and not have been deallocated already
    unsafe fn drop_alloc(&mut self) {
        unsafe {
            let vec = Vec::from_raw_parts(
                self.heap.data.as_ptr(),
                self.heap.len as usize,
                self.heap.capacity as usize,
            );
            std::mem::drop(vec);
        }
    }

    pub(crate) fn push(&mut self, id: T) {
        let len = self.len();
        const_assert!(usize::BITS >= u32::BITS);
        assert!((len as usize) < (isize::MAX as usize));
        assert!((len as usize) < (u32::MAX as usize));

        if len < INLINE_CAPACITY {
            unsafe {
                // SAFETY: when len <= INLINE_CAPACITY, the active variant is `inline`
                self.inline.data[len as usize] = id;
                self.inline.len += 1;
            }
        } else if len == INLINE_CAPACITY {
            unsafe {
                // SAFETY: when len == INLINE_CAPACITY, the active variant is `inline`, but after adding it must be `heap`
                let new_heap = alloc_new(&self.inline.data, id);
                self.heap = ManuallyDrop::new(new_heap);
            }
        } else {
            unsafe {
                // SAFETY: when len > INLINE_CAPACITY, the active variant is `heap`
                let capacity = self.heap.capacity;
                if len < capacity {
                    self.heap.data.as_ptr().add(len as usize).write(id);
                    self.heap.len += 1;
                } else {
                    let existing = std::slice::from_raw_parts(
                        self.heap.data.as_ptr().cast_const(),
                        len as usize,
                    );
                    let new_heap = alloc_new(existing, id);
                    self.drop_alloc();
                    self.heap = ManuallyDrop::new(new_heap);
                }
            }
        }
    }

    pub(crate) fn as_slice(&self) -> &[T] {
        let len = self.len();
        if len <= INLINE_CAPACITY {
            unsafe {
                // SAFETY: when len <= INLINE_CAPACITY, the active variant is `inline`
                &self.inline.data[..(len as usize)]
            }
        } else {
            unsafe {
                // SAFETY: when len > INLINE_CAPACITY, the active variant is `heap`
                std::slice::from_raw_parts(self.heap.data.as_ptr().cast_const(), len as usize)
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

impl<T: IdInternal> Drop for IdVec<T> {
    fn drop(&mut self) {
        if self.len() > INLINE_CAPACITY {
            unsafe {
                // SAFETY: when len > INLINE_CAPACITY, the active variant is `heap`
                self.drop_alloc();
            }
        }
    }
}

unsafe impl<T: IdInternal> Send for IdVec<T> {}
unsafe impl<T: IdInternal> Sync for IdVec<T> {}
