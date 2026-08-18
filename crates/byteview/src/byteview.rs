// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    mem::ManuallyDrop,
    ops::Deref,
    sync::atomic::{AtomicU64, Ordering, fence},
};

pub use crate::builder::Builder;

#[cfg(target_pointer_width = "64")]
const INLINE_SIZE: usize = 20;

#[cfg(target_pointer_width = "32")]
const INLINE_SIZE: usize = 16;

const PREFIX_SIZE: usize = 4;

#[repr(C)]
struct HeapAllocationHeader {
    ref_count: AtomicU64,
}

fn allocation_layout(data_len: usize) -> std::alloc::Layout {
    let Some(total_size) = std::mem::size_of::<HeapAllocationHeader>().checked_add(data_len) else {
        panic!("byte slice too long");
    };
    let alignment = std::mem::align_of::<HeapAllocationHeader>();
    let Ok(layout) = std::alloc::Layout::from_size_align(total_size, alignment) else {
        unreachable!("heap header alignment is always valid");
    };
    layout
}

#[repr(C)]
struct ShortRepr {
    len: u32,
    data: [u8; INLINE_SIZE],
}

#[repr(C)]
struct LongRepr {
    len: u32,
    prefix: [u8; PREFIX_SIZE],
    heap: *const u8,
    original_len: u32,
    offset: u32,
}

#[repr(C)]
union Trailer {
    short: ManuallyDrop<ShortRepr>,
    long: ManuallyDrop<LongRepr>,
}

impl Default for Trailer {
    fn default() -> Self {
        Self {
            short: ManuallyDrop::new(ShortRepr {
                len: 0,
                data: [0; INLINE_SIZE],
            }),
        }
    }
}

/// An immutable byte slice
///
/// Will be inlined (no pointer dereference or heap allocation)
/// if it is 20 characters or shorter (on a 64-bit system).
///
/// A single heap allocation will be shared between multiple slices.
/// Even subslices of that heap allocation can be cloned without additional heap allocation.
///
/// [`ByteView`] does not guarantee any sort of alignment for zero-copy (de)serialization.
///
/// The design is very similar to:
///
/// - [Polars' strings](<https://pola.rs/posts/polars-string-type>)
/// - [CedarDB's German strings](<https://cedardb.com/blog/german_strings>)
/// - [Umbra's string](<https://db.in.tum.de/~freitag/papers/p29-neumann-cidr20.pdf>)
/// - [Velox' String View](https://facebookincubator.github.io/velox/develop/vectors.html)
/// - [Apache Arrow's String View](https://arrow.apache.org/docs/cpp/api/datatype.html#_CPPv4N5arrow14BinaryViewType6c_typeE)
#[repr(C)]
#[derive(Default)]
pub struct ByteView {
    trailer: Trailer,
}

#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for ByteView {}
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for ByteView {}

impl Clone for ByteView {
    fn clone(&self) -> Self {
        if !self.is_inline() {
            self.get_heap_region()
                .ref_count
                .fetch_add(1, Ordering::Relaxed);
        }

        // SAFETY: Inline views own no external resource. Heap views share their
        // allocation, whose reference count was incremented above.
        unsafe { std::ptr::read(self) }
    }
}

impl Drop for ByteView {
    fn drop(&mut self) {
        if self.is_inline() {
            return;
        }

        let heap_region = self.get_heap_region();

        if heap_region.ref_count.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }
        fence(Ordering::Acquire);

        unsafe {
            let layout = allocation_layout(self.trailer.long.original_len as usize);
            let ptr = self.trailer.long.heap.cast_mut();
            std::alloc::dealloc(ptr, layout);
        }
    }
}

impl Eq for ByteView {}

impl std::cmp::PartialEq for ByteView {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let a = std::ptr::from_ref(self).cast::<u64>().read_unaligned();
            let b = std::ptr::from_ref(other).cast::<u64>().read_unaligned();

            if a != b {
                return false;
            }
        }

        // The first word contains the length and cached four-byte prefix.
        // Compare only the bytes that were not already checked.
        self.get(PREFIX_SIZE..).unwrap_or_default() == other.get(PREFIX_SIZE..).unwrap_or_default()
    }
}

impl std::cmp::Ord for ByteView {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.prefix().cmp(other.prefix()).then_with(|| {
            self.get(PREFIX_SIZE..)
                .unwrap_or_default()
                .cmp(other.get(PREFIX_SIZE..).unwrap_or_default())
        })
    }
}

impl std::cmp::PartialOrd for ByteView {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Debug for ByteView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &**self)
    }
}

impl Deref for ByteView {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        if self.is_inline() {
            self.get_short_slice()
        } else {
            self.get_long_slice()
        }
    }
}

impl std::hash::Hash for ByteView {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

impl ByteView {
    #[doc(hidden)]
    #[must_use]
    pub unsafe fn builder_unzeroed(len: usize) -> Builder {
        // SAFETY: The caller is responsible for initializing every byte before
        // the returned builder is frozen.
        unsafe { Builder::new(Self::with_size_unzeroed(len)) }
    }

    fn prefix(&self) -> &[u8] {
        let len = PREFIX_SIZE.min(self.len());

        // SAFETY: Both trailer layouts have the prefix stored at the same position
        unsafe { self.trailer.short.data.get_unchecked(..len) }
    }

    fn is_inline(&self) -> bool {
        self.len() <= INLINE_SIZE
    }

    pub(crate) fn update_prefix(&mut self) {
        if !self.is_inline() {
            unsafe {
                let slice_ptr: &[u8] = &*self;
                let slice_ptr = slice_ptr.as_ptr();

                let prefix = (*self.trailer.long).prefix.as_mut_ptr();
                std::ptr::copy_nonoverlapping(slice_ptr, prefix, PREFIX_SIZE);
            }
        }
    }

    /// Creates a byteview and populates it with `len` bytes
    /// from the given reader.
    ///
    /// # Errors
    ///
    /// Returns an error if an I/O error occurred.
    pub fn from_reader<R: std::io::Read>(reader: &mut R, len: usize) -> std::io::Result<Self> {
        // NOTE: We can use _unzeroed to skip zeroing of the heap allocated slice
        // because we receive the `len` parameter
        // If the reader does not give us exactly `len` bytes, `read_exact` fails anyway
        let mut builder = unsafe { Self::builder_unzeroed(len) };
        reader.read_exact(&mut builder)?;
        Ok(builder.freeze())
    }

    /// Fuses two byte slices into a single byteview.
    #[must_use]
    pub fn fused(left: &[u8], right: &[u8]) -> Self {
        let len = left.len() + right.len();
        let mut builder = unsafe { Self::builder_unzeroed(len) };
        let (left_target, right_target) = builder.split_at_mut(left.len());
        left_target.copy_from_slice(left);
        right_target.copy_from_slice(right);
        builder.freeze()
    }

    /// Creates a new fixed-length byteview, **with uninitialized contents**.
    ///
    /// # Panics
    ///
    /// Panics if the length does not fit in a u32 (4 GiB).
    #[doc(hidden)]
    #[must_use]
    pub unsafe fn with_size_unzeroed(slice_len: usize) -> Self {
        let view = if slice_len <= INLINE_SIZE {
            Self {
                trailer: Trailer {
                    short: ManuallyDrop::new(ShortRepr {
                        // SAFETY: We know slice_len is INLINE_SIZE or less, so it must be
                        // a valid u32
                        #[allow(clippy::cast_possible_truncation)]
                        len: slice_len as u32,
                        data: [0; INLINE_SIZE],
                    }),
                },
            }
        } else {
            let Ok(len) = u32::try_from(slice_len) else {
                panic!("byte slice too long");
            };

            unsafe {
                let layout = allocation_layout(slice_len);

                let heap_ptr = std::alloc::alloc(layout);
                if heap_ptr.is_null() {
                    std::alloc::handle_alloc_error(layout);
                }

                // Set ref count
                #[expect(
                    clippy::cast_ptr_alignment,
                    reason = "the allocation uses HeapAllocationHeader alignment"
                )]
                let heap_region = heap_ptr.cast::<HeapAllocationHeader>();
                let heap_region = &*heap_region;
                heap_region.ref_count.store(1, Ordering::Release);

                Self {
                    trailer: Trailer {
                        long: ManuallyDrop::new(LongRepr {
                            len,
                            prefix: [0; PREFIX_SIZE],
                            heap: heap_ptr,
                            original_len: len,
                            offset: 0,
                        }),
                    },
                }
            }
        };

        debug_assert_eq!(1, view.ref_count());

        view
    }

    /// Creates a new byteview from an existing byte slice.
    ///
    /// Will heap-allocate the slice if it has at least length 21.
    ///
    /// # Panics
    ///
    /// Panics if the length does not fit in a u32 (4 GiB).
    #[must_use]
    pub fn new(slice: &[u8]) -> Self {
        let slice_len = slice.len();

        let mut view = unsafe { Self::with_size_unzeroed(slice_len) };

        if view.is_inline() {
            // SAFETY: We check for inlinability
            // so we know the the input slice fits our buffer
            unsafe {
                let data_ptr = std::ptr::addr_of_mut!((*view.trailer.short).data).cast();
                std::ptr::copy_nonoverlapping(slice.as_ptr(), data_ptr, slice_len);
            }
        } else {
            let long_repr = unsafe { &mut *view.trailer.long };

            // Copy prefix
            // SAFETY: We know that there are at least 4 bytes in the input slice
            #[allow(clippy::indexing_slicing)]
            long_repr.prefix.copy_from_slice(&slice[0..PREFIX_SIZE]);

            // Copy byte slice into heap allocation
            view.get_mut_slice().copy_from_slice(slice);
        }

        debug_assert_eq!(1, view.ref_count());

        view
    }

    unsafe fn data_ptr(&self) -> *const u8 {
        const HEADER_SIZE: usize = std::mem::size_of::<HeapAllocationHeader>();

        debug_assert!(!self.is_inline());

        // SAFETY: The non-inline representation is active, and its allocation
        // contains the header followed by `original_len` data bytes.
        unsafe {
            self.trailer
                .long
                .heap
                .add(HEADER_SIZE)
                .add(self.trailer.long.offset as usize)
        }
    }

    unsafe fn data_ptr_mut(&mut self) -> *mut u8 {
        const HEADER_SIZE: usize = std::mem::size_of::<HeapAllocationHeader>();

        debug_assert!(!self.is_inline());

        // SAFETY: The non-inline representation is active, and its allocation
        // contains the header followed by `original_len` data bytes.
        unsafe {
            self.trailer
                .long
                .heap
                .add(HEADER_SIZE)
                .add(self.trailer.long.offset as usize)
                .cast_mut()
        }
    }

    fn get_heap_region(&self) -> &HeapAllocationHeader {
        debug_assert!(
            !self.is_inline(),
            "inline slice does not have a heap allocation"
        );

        unsafe {
            let ptr = self.trailer.long.heap;
            #[expect(
                clippy::cast_ptr_alignment,
                reason = "heap pointers come from a HeapAllocationHeader-aligned allocation"
            )]
            let heap_region: *const HeapAllocationHeader = ptr.cast::<HeapAllocationHeader>();
            &*heap_region
        }
    }

    fn ref_count(&self) -> u64 {
        if self.is_inline() {
            1
        } else {
            self.get_heap_region().ref_count.load(Ordering::Acquire)
        }
    }

    /// Clones the given range of the existing byteview without heap allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use byteview::ByteView;
    /// let slice = ByteView::from("helloworld_thisisalongstring");
    /// let copy = slice.slice(11..);
    /// assert_eq!(b"thisisalongstring", &*copy);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the slice is out of bounds.
    #[must_use]
    pub fn slice(&self, range: impl std::ops::RangeBounds<usize>) -> Self {
        use core::ops::Bound;

        // Credits: This is essentially taken from
        // https://github.com/tokio-rs/bytes/blob/291df5acc94b82a48765e67eeb1c1a2074539e68/src/bytes.rs#L264

        let self_len = self.len();

        let begin = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n
                .checked_add(1)
                .unwrap_or_else(|| panic!("range start out of bounds")),
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(&n) => n
                .checked_add(1)
                .unwrap_or_else(|| panic!("range end out of bounds")),
            Bound::Excluded(&n) => n,
            Bound::Unbounded => self_len,
        };

        assert!(
            begin <= end,
            "range start must not be greater than end: {begin:?} <= {end:?}",
        );
        assert!(
            end <= self_len,
            "range end out of bounds: {end:?} <= {self_len:?}",
        );

        let new_len = end - begin;
        let Ok(len) = u32::try_from(new_len) else {
            unreachable!("a ByteView range always fits in u32");
        };
        let Ok(begin_u32) = u32::try_from(begin) else {
            unreachable!("a ByteView offset always fits in u32");
        };

        // Target and destination slices are inlined
        // so we just need to memcpy the struct, and replace
        // the inline slice with the requested range
        if new_len <= INLINE_SIZE {
            let mut child = Self {
                trailer: Trailer {
                    short: ManuallyDrop::new(ShortRepr {
                        len,
                        data: [0; INLINE_SIZE],
                    }),
                },
            };

            let Some(slice) = self.get(begin..end) else {
                unreachable!("range was validated above");
            };
            debug_assert_eq!(slice.len(), new_len);

            let data_ptr = unsafe { &mut (*child.trailer.short).data };

            unsafe {
                std::ptr::copy_nonoverlapping(slice.as_ptr(), data_ptr.as_mut_ptr(), new_len);
            }

            child
        } else {
            // IMPORTANT: Increase ref count
            let heap_region = self.get_heap_region();
            heap_region.ref_count.fetch_add(1, Ordering::Relaxed);

            let mut child = Self {
                // SAFETY: self.data must be defined
                // we cannot get a range larger than our own slice
                // so we cannot be inlined while the requested slice is not inlinable
                trailer: Trailer {
                    long: ManuallyDrop::new(LongRepr {
                        len,
                        prefix: [0; PREFIX_SIZE],
                        heap: unsafe { self.trailer.long.heap },
                        offset: unsafe { self.trailer.long.offset } + begin_u32,
                        original_len: unsafe { self.trailer.long.original_len },
                    }),
                },
            };

            let Some(prefix) = self.get(begin..(begin + PREFIX_SIZE)) else {
                unreachable!("non-inline ranges contain a full prefix");
            };
            debug_assert_eq!(prefix.len(), 4);

            unsafe {
                (*child.trailer.long).prefix.copy_from_slice(prefix);
            }

            child
        }
    }

    /// Returns `true` if the slice is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the amount of bytes in the slice.
    #[must_use]
    pub fn len(&self) -> usize {
        unsafe { self.trailer.short.len as usize }
    }

    pub(crate) fn get_mut_slice(&mut self) -> &mut [u8] {
        let len = self.len();

        if self.is_inline() {
            unsafe { std::slice::from_raw_parts_mut((*self.trailer.short).data.as_mut_ptr(), len) }
        } else {
            unsafe { std::slice::from_raw_parts_mut(self.data_ptr_mut(), len) }
        }
    }

    fn get_short_slice(&self) -> &[u8] {
        let len = self.len();

        debug_assert!(
            len <= INLINE_SIZE,
            "cannot get short slice - slice is not inlined",
        );

        // SAFETY: Shall only be called if slice is inlined
        unsafe { std::slice::from_raw_parts((*self.trailer.short).data.as_ptr(), len) }
    }

    fn get_long_slice(&self) -> &[u8] {
        let len = self.len();

        debug_assert!(
            len > INLINE_SIZE,
            "cannot get long slice - slice is inlined"
        );

        // SAFETY: Shall only be called if slice is heap allocated
        unsafe { std::slice::from_raw_parts(self.data_ptr(), len) }
    }
}

impl std::borrow::Borrow<[u8]> for ByteView {
    fn borrow(&self) -> &[u8] {
        self
    }
}

impl AsRef<[u8]> for ByteView {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl From<&[u8]> for ByteView {
    fn from(value: &[u8]) -> Self {
        Self::new(value)
    }
}

impl From<Vec<u8>> for ByteView {
    fn from(value: Vec<u8>) -> Self {
        Self::new(&value)
    }
}

impl From<&str> for ByteView {
    fn from(value: &str) -> Self {
        Self::from(value.as_bytes())
    }
}

impl<const N: usize> From<[u8; N]> for ByteView {
    fn from(value: [u8; N]) -> Self {
        Self::from(value.as_slice())
    }
}

impl<const N: usize> From<&[u8; N]> for ByteView {
    fn from(value: &[u8; N]) -> Self {
        Self::from(value.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::{ByteView, HeapAllocationHeader};
    use std::io::Cursor;

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn memsize() {
        use crate::byteview::{LongRepr, ShortRepr, Trailer};

        assert_eq!(
            std::mem::size_of::<ShortRepr>(),
            std::mem::size_of::<LongRepr>()
        );
        assert_eq!(
            std::mem::size_of::<Trailer>(),
            std::mem::size_of::<LongRepr>()
        );

        assert_eq!(24, std::mem::size_of::<ByteView>());
        assert_eq!(
            32,
            std::mem::size_of::<ByteView>() + std::mem::size_of::<HeapAllocationHeader>()
        );
    }

    #[test]
    fn sliced_clone() {
        let s = ByteView::from([
            1, 255, 255, 255, 251, 255, 255, 255, 255, 255, 1, 21, 255, 255, 255, 255, 5, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 4, 3, 255,
            255, 0, 0, 255, 0, 0, 0, 254, 2, 0, 0, 0, 5, 2, 42, 0, 0, 0, 1, 0, 0, 0, 44, 0, 0, 0,
            2, 0, 0, 0,
        ]);
        let slice = s.slice(12..(12 + 21));

        #[allow(clippy::redundant_clone)]
        let cloned = slice.clone();

        assert_eq!(slice.prefix(), cloned.prefix());
        assert_eq!(slice, cloned);
    }

    #[test]
    fn sized_slice_ref() {
        let b = b"hello";
        let _bytes = ByteView::from(b);
    }

    #[test]
    fn fuse_empty() {
        let bytes = ByteView::fused(&[], &[]);
        assert_eq!(&*bytes, &[] as &[u8]);
    }

    #[test]
    fn fuse_one() {
        let bytes = ByteView::fused(b"abc", &[]);
        assert_eq!(&*bytes, b"abc");
    }

    #[test]
    fn fuse_two() {
        let bytes = ByteView::fused(b"abc", b"def");
        assert_eq!(&*bytes, b"abcdef");
    }

    #[test]
    fn dealloc_order() {
        let bytes = ByteView::new(&(0..32).collect::<Vec<_>>());
        let bytes_slice = bytes.slice(..31);
        drop(bytes);
        drop(bytes_slice);
    }

    #[test]
    fn dealloc_order_2() {
        let bytes = ByteView::new(&(0..32).collect::<Vec<_>>());
        let bytes_slice = bytes.slice(..31);
        let bytes_slice_2 = bytes.slice(..5);
        let bytes_slice_3 = bytes.slice(..6);

        drop(bytes);
        drop(bytes_slice);
        drop(bytes_slice_2);
        drop(bytes_slice_3);
    }

    #[test]
    fn from_reader_1() -> std::io::Result<()> {
        let str = b"abcdef";
        let mut cursor = Cursor::new(str);

        let a = ByteView::from_reader(&mut cursor, 6)?;
        assert_eq!(&*a, b"abcdef");

        Ok(())
    }

    #[test]
    fn cmp_misc_1() {
        let a = ByteView::from("abcdef");
        let b = ByteView::from("abcdefhelloworldhelloworld");
        assert!(a < b);
    }

    #[test]
    fn nostr() {
        let slice = ByteView::from("");
        assert_eq!(0, slice.len());
        assert_eq!(&*slice, b"");
        assert_eq!(1, slice.ref_count());
        assert!(slice.is_inline());
    }

    #[test]
    fn default_str() {
        let slice = ByteView::default();
        assert_eq!(0, slice.len());
        assert_eq!(&*slice, b"");
        assert_eq!(1, slice.ref_count());
        assert!(slice.is_inline());
    }

    #[test]
    fn short_str() {
        let slice = ByteView::from("abcdef");
        assert_eq!(6, slice.len());
        assert_eq!(&*slice, b"abcdef");
        assert_eq!(1, slice.ref_count());
        assert_eq!(&slice.prefix(), b"abcd");
        assert!(slice.is_inline());
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn medium_str() {
        let slice = ByteView::from("abcdefabcdef");
        assert_eq!(12, slice.len());
        assert_eq!(&*slice, b"abcdefabcdef");
        assert_eq!(1, slice.ref_count());
        assert_eq!(&slice.prefix(), b"abcd");
        assert!(slice.is_inline());
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn medium_long_str() {
        let slice = ByteView::from("abcdefabcdefabcdabcd");
        assert_eq!(20, slice.len());
        assert_eq!(&*slice, b"abcdefabcdefabcdabcd");
        assert_eq!(1, slice.ref_count());
        assert_eq!(&slice.prefix(), b"abcd");
        assert!(slice.is_inline());
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn medium_str_clone() {
        let slice = ByteView::from("abcdefabcdefabcdefab");
        let copy = slice.clone();
        assert_eq!(slice, copy);
        assert_eq!(copy.prefix(), slice.prefix());

        assert_eq!(1, slice.ref_count());

        drop(copy);
        assert_eq!(1, slice.ref_count());
    }

    #[test]
    fn long_str() {
        let slice = ByteView::from("abcdefabcdefabcdefababcd");
        assert_eq!(24, slice.len());
        assert_eq!(&*slice, b"abcdefabcdefabcdefababcd");
        assert_eq!(1, slice.ref_count());
        assert_eq!(&slice.prefix(), b"abcd");
        assert!(!slice.is_inline());
    }

    #[test]
    fn long_str_clone() {
        let slice = ByteView::from("abcdefabcdefabcdefababcd");
        let copy = slice.clone();
        assert_eq!(slice, copy);
        assert_eq!(copy.prefix(), slice.prefix());

        assert_eq!(2, slice.ref_count());

        drop(copy);
        assert_eq!(1, slice.ref_count());
    }

    #[test]
    fn long_str_slice_full() {
        let slice = ByteView::from("helloworld_thisisalongstring");

        let copy = slice.slice(..);
        assert_eq!(copy, slice);

        assert_eq!(2, slice.ref_count());

        drop(copy);
        assert_eq!(1, slice.ref_count());
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn long_str_slice() {
        let slice = ByteView::from("helloworld_thisisalongstring");

        let copy = slice.slice(11..);
        assert_eq!(b"thisisalongstring", &*copy);
        assert_eq!(&copy.prefix(), b"this");

        assert_eq!(1, slice.ref_count());

        drop(copy);
        assert_eq!(1, slice.ref_count());
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn long_str_slice_twice() {
        let slice = ByteView::from("helloworld_thisisalongstring");

        let copy = slice.slice(11..);
        assert_eq!(b"thisisalongstring", &*copy);

        let copycopy = copy.slice(..);
        assert_eq!(copy, copycopy);

        assert_eq!(1, slice.ref_count());

        drop(copy);
        assert_eq!(1, slice.ref_count());

        drop(slice);
        assert_eq!(1, copycopy.ref_count());
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn long_str_slice_downgrade() {
        let slice = ByteView::from("helloworld_thisisalongstring");

        let copy = slice.slice(11..);
        assert_eq!(b"thisisalongstring", &*copy);

        let copycopy = copy.slice(0..4);
        assert_eq!(b"this", &*copycopy);

        {
            let copycopy = copy.slice(0..=4);
            assert_eq!(b"thisi", &*copycopy);
            assert_eq!(Some(b't'), copycopy.first().copied());
        }

        assert_eq!(1, slice.ref_count());

        drop(copy);
        assert_eq!(1, slice.ref_count());

        drop(copycopy);
        assert_eq!(1, slice.ref_count());
    }

    #[test]
    fn short_str_clone() {
        let slice = ByteView::from("abcdef");
        let copy = slice.clone();
        assert_eq!(slice, copy);

        assert_eq!(1, slice.ref_count());

        drop(slice);
        assert_eq!(&*copy, b"abcdef");

        assert_eq!(1, copy.ref_count());
    }

    #[test]
    fn short_str_slice_full() {
        let slice = ByteView::from("abcdef");
        let copy = slice.slice(..);
        assert_eq!(slice, copy);

        assert_eq!(1, slice.ref_count());

        drop(slice);
        assert_eq!(&*copy, b"abcdef");

        assert_eq!(1, copy.ref_count());
    }

    #[test]
    fn short_str_slice_part() {
        let slice = ByteView::from("abcdef");
        let copy = slice.slice(3..);

        assert_eq!(1, slice.ref_count());

        drop(slice);
        assert_eq!(&*copy, b"def");

        assert_eq!(1, copy.ref_count());
    }

    #[test]
    fn short_str_slice_empty() {
        let slice = ByteView::from("abcdef");
        let copy = slice.slice(0..0);

        assert_eq!(1, slice.ref_count());

        drop(slice);
        assert_eq!(&*copy, b"");

        assert_eq!(1, copy.ref_count());
    }

    #[test]
    fn tiny_str_starts_with() {
        let a = ByteView::from("abc");
        assert!(a.starts_with(b"ab"));
        assert!(!a.starts_with(b"b"));
    }

    #[test]
    fn long_str_starts_with() {
        let a = ByteView::from("abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdef");
        assert!(a.starts_with(b"abcdef"));
        assert!(!a.starts_with(b"def"));
    }

    #[test]
    fn tiny_str_cmp() {
        let a = ByteView::from("abc");
        let b = ByteView::from("def");
        assert!(a < b);
    }

    #[test]
    fn tiny_str_eq() {
        let a = ByteView::from("abc");
        let b = ByteView::from("def");
        assert_ne!(a, b);
    }

    #[test]
    fn long_str_eq() {
        let a = ByteView::from("abcdefabcdefabcdefabcdef");
        let b = ByteView::from("xycdefabcdefabcdefabcdef");
        assert_ne!(a, b);
    }

    #[test]
    fn long_str_cmp() {
        let a = ByteView::from("abcdefabcdefabcdefabcdef");
        let b = ByteView::from("xycdefabcdefabcdefabcdef");
        assert!(a < b);
    }

    #[test]
    fn long_str_eq_2() {
        let a = ByteView::from("abcdefabcdefabcdefabcdef");
        let b = ByteView::from("abcdefabcdefabcdefabcdef");
        assert_eq!(a, b);
    }

    #[test]
    fn long_str_cmp_2() {
        let a = ByteView::from("abcdefabcdefabcdefabcdef");
        let b = ByteView::from("abcdefabcdefabcdefabcdeg");
        assert!(a < b);
    }

    #[test]
    fn long_str_cmp_3() {
        let a = ByteView::from("abcdefabcdefabcdefabcde");
        let b = ByteView::from("abcdefabcdefabcdefabcdef");
        assert!(a < b);
    }

    #[test]
    fn cmp_fuzz_1() {
        let a = ByteView::from([0]);
        let b = ByteView::from([]);

        assert!(a > b);
        assert_ne!(a, b);
    }

    #[test]
    fn cmp_fuzz_2() {
        let a = ByteView::from([0, 0]);
        let b = ByteView::from([0]);

        assert!(a > b);
        assert_ne!(a, b);
    }

    #[test]
    fn cmp_fuzz_3() {
        let a = ByteView::from([255, 255, 12, 255, 0]);
        let b = ByteView::from([255, 255, 12, 255]);

        assert!(a > b);
        assert_ne!(a, b);
    }

    #[test]
    fn cmp_fuzz_4() {
        let a = ByteView::from([
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        ]);
        let b = ByteView::from([
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0,
        ]);

        assert!(a > b);
        assert_ne!(a, b);
    }
}
