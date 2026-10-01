// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    alloc::{Layout, alloc, alloc_zeroed, dealloc, handle_alloc_error},
    borrow::Borrow,
    cmp::{Ord, Ordering as CmpOrdering, PartialEq, PartialOrd},
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    io::{Read, Result as IoResult},
    mem::{self, ManuallyDrop, MaybeUninit},
    ops::{Bound, Deref, RangeBounds},
    process, ptr, slice,
    sync::atomic::{AtomicU64, Ordering, fence},
};

#[path = "builder.rs"]
mod builder;

pub use builder::Builder;

#[cfg(target_pointer_width = "64")]
const INLINE_SIZE: usize = 12;

#[cfg(target_pointer_width = "32")]
const INLINE_SIZE: usize = 8;

#[repr(C)]
struct HeapAllocationHeader {
    ref_count: AtomicU64,
    len: u32,
}

fn allocation_layout(data_len: usize) -> Layout {
    let Some(total_size) = mem::size_of::<HeapAllocationHeader>().checked_add(data_len) else {
        panic!("byte slice too long");
    };
    let alignment = mem::align_of::<HeapAllocationHeader>();
    let Ok(layout) = Layout::from_size_align(total_size, alignment) else {
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
    offset: u32,
    heap: *const u8,
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
/// if it is 12 bytes or shorter (on a 64-bit system).
///
/// A single heap allocation will be shared between multiple slices.
/// Even subslices of that heap allocation can be cloned without additional heap allocation.
///
/// [`ByteView`] does not guarantee any sort of alignment for zero-copy (de)serialization.
#[repr(C)]
#[derive(Default)]
pub struct ByteView {
    trailer: Trailer,
}

// SAFETY: Shared allocations contain immutable bytes and an atomic reference count.
// Mutable access is confined to a uniquely owned Builder before it is frozen.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for ByteView {}
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for ByteView {}

impl Clone for ByteView {
    fn clone(&self) -> Self {
        if !self.is_inline() {
            self.increment_ref_count();
        }

        // SAFETY: Inline views own no external resource. Heap views share their
        // allocation, whose reference count was incremented above.
        unsafe { ptr::read(self) }
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
            let layout = allocation_layout(heap_region.len as usize);
            let ptr = self.trailer.long.heap.cast_mut();
            dealloc(ptr, layout);
        }
    }
}

impl Eq for ByteView {}

impl PartialEq for ByteView {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Ord for ByteView {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl PartialOrd for ByteView {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Debug for ByteView {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
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

impl Hash for ByteView {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

impl ByteView {
    /// Creates a uniquely owned, zero-initialized buffer for writing bytes.
    ///
    /// # Panics
    ///
    /// Panics if the length does not fit in a u32 (4 GiB).
    #[must_use]
    pub fn builder(len: usize) -> Builder {
        // SAFETY: allocation initializes the header and all exposed bytes.
        Builder::new(unsafe { Self::with_size(len, true) })
    }

    /// Initializes a fresh allocation without first clearing its bytes.
    ///
    /// The initializer must return the entire buffer as an initialized slice.
    /// On error, the partially initialized allocation is discarded.
    ///
    /// # Errors
    ///
    /// Returns the initializer's error.
    ///
    /// # Panics
    ///
    /// Panics if the length exceeds `u32::MAX`, or the initializer returns a
    /// different buffer or length.
    pub fn try_init<E>(
        len: usize,
        init: impl FnOnce(&mut [MaybeUninit<u8>]) -> Result<&[u8], E>,
    ) -> Result<Self, E> {
        // SAFETY: Only MaybeUninit bytes are exposed until initialization succeeds.
        let mut view = unsafe { Self::with_size(len, false) };
        let data = if view.is_inline() {
            unsafe { (*view.trailer.short).data.as_mut_ptr() }
        } else {
            unsafe { view.data_ptr_mut() }
        };
        let uninit = unsafe { slice::from_raw_parts_mut(data.cast(), len) };
        let initialized = init(uninit)?;
        assert_eq!(
            initialized.len(),
            len,
            "initializer returned a partial buffer"
        );
        if len != 0 {
            assert_eq!(
                initialized.as_ptr(),
                data,
                "initializer returned a different buffer"
            );
        }
        // The returned &[u8] proves that every byte in this allocation is initialized.
        Ok(view)
    }

    fn increment_ref_count(&self) {
        let previous = self
            .get_heap_region()
            .ref_count
            .fetch_add(1, Ordering::Relaxed);
        // Leaked clones must never wrap the counter and free a live allocation.
        if previous >= u64::MAX / 2 {
            process::abort();
        }
    }

    fn is_inline(&self) -> bool {
        self.len() <= INLINE_SIZE
    }

    /// Creates a byteview and populates it with `len` bytes
    /// from the given reader.
    ///
    /// # Errors
    ///
    /// Returns an error if an I/O error occurred.
    pub fn from_reader<R: Read>(reader: &mut R, len: usize) -> IoResult<Self> {
        let mut builder = Self::builder(len);
        reader.read_exact(&mut builder)?;
        Ok(builder.freeze())
    }

    /// Fuses two byte slices into a single byteview.
    ///
    /// # Panics
    ///
    /// Panics if the combined length does not fit in a [`usize`].
    #[must_use]
    pub fn fused(left: &[u8], right: &[u8]) -> Self {
        let Some(len) = left.len().checked_add(right.len()) else {
            panic!("byte slice too long");
        };
        if len <= INLINE_SIZE {
            let mut data = [0; INLINE_SIZE];
            let (left_target, remaining) = data.split_at_mut(left.len());
            let (right_target, _) = remaining.split_at_mut(right.len());
            left_target.copy_from_slice(left);
            right_target.copy_from_slice(right);
            return Self {
                trailer: Trailer {
                    short: ManuallyDrop::new(ShortRepr {
                        #[allow(clippy::cast_possible_truncation)]
                        len: len as u32,
                        data,
                    }),
                },
            };
        }

        let mut view = unsafe { Self::with_size(len, false) };
        // SAFETY: the fresh allocation has space for both slices. No borrowed
        // byte slice is exposed until both copies have initialized the data.
        unsafe {
            let data = view.data_ptr_mut();
            ptr::copy_nonoverlapping(left.as_ptr(), data, left.len());
            ptr::copy_nonoverlapping(right.as_ptr(), data.add(left.len()), right.len());
        }
        view
    }

    // If zeroed is false, the caller must initialize all bytes before exposing a slice.
    unsafe fn with_size(slice_len: usize, zeroed: bool) -> Self {
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

                let heap_ptr = if zeroed {
                    alloc_zeroed(layout)
                } else {
                    alloc(layout)
                };
                if heap_ptr.is_null() {
                    handle_alloc_error(layout);
                }

                // Set ref count
                #[expect(
                    clippy::cast_ptr_alignment,
                    reason = "the allocation uses HeapAllocationHeader alignment"
                )]
                let heap_region = heap_ptr.cast::<HeapAllocationHeader>();
                heap_region.write(HeapAllocationHeader {
                    ref_count: AtomicU64::new(1),
                    len,
                });

                Self {
                    trailer: Trailer {
                        long: ManuallyDrop::new(LongRepr {
                            len,
                            heap: heap_ptr,
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
    /// Heap-allocates slices longer than 12 bytes on 64-bit targets.
    ///
    /// # Panics
    ///
    /// Panics if the length does not fit in a u32 (4 GiB).
    #[must_use]
    pub fn new(slice: &[u8]) -> Self {
        let slice_len = slice.len();

        let mut view = unsafe { Self::with_size(slice_len, false) };

        if view.is_inline() {
            // SAFETY: We check for inlinability
            // so we know the the input slice fits our buffer
            unsafe {
                let data_ptr = ptr::addr_of_mut!((*view.trailer.short).data).cast();
                ptr::copy_nonoverlapping(slice.as_ptr(), data_ptr, slice_len);
            }
        } else {
            // SAFETY: the unique allocation has exactly slice_len data bytes.
            unsafe { ptr::copy_nonoverlapping(slice.as_ptr(), view.data_ptr_mut(), slice_len) };
        }

        debug_assert_eq!(1, view.ref_count());

        view
    }

    unsafe fn data_ptr(&self) -> *const u8 {
        const HEADER_SIZE: usize = mem::size_of::<HeapAllocationHeader>();

        debug_assert!(!self.is_inline());

        // SAFETY: The non-inline representation is active, and its allocation
        // contains the header followed by the allocation's data bytes.
        unsafe {
            self.trailer
                .long
                .heap
                .add(HEADER_SIZE)
                .add(self.trailer.long.offset as usize)
        }
    }

    unsafe fn data_ptr_mut(&mut self) -> *mut u8 {
        const HEADER_SIZE: usize = mem::size_of::<HeapAllocationHeader>();

        debug_assert!(!self.is_inline());

        // SAFETY: The non-inline representation is active, and its allocation
        // contains the header followed by the allocation's data bytes.
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
    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
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
                ptr::copy_nonoverlapping(slice.as_ptr(), data_ptr.as_mut_ptr(), new_len);
            }

            child
        } else {
            // IMPORTANT: Increase ref count
            self.increment_ref_count();

            Self {
                // SAFETY: a non-inline child comes from a non-inline parent.
                // Its offset and length stay within the original allocation.
                trailer: Trailer {
                    long: ManuallyDrop::new(LongRepr {
                        len,
                        heap: unsafe { self.trailer.long.heap },
                        offset: unsafe { self.trailer.long.offset } + begin_u32,
                    }),
                },
            }
        }
    }

    /// Returns the amount of bytes in the slice.
    #[must_use]
    fn len(&self) -> usize {
        unsafe { self.trailer.short.len as usize }
    }

    fn get_mut_slice(&mut self) -> &mut [u8] {
        let len = self.len();

        if self.is_inline() {
            unsafe { slice::from_raw_parts_mut((*self.trailer.short).data.as_mut_ptr(), len) }
        } else {
            unsafe { slice::from_raw_parts_mut(self.data_ptr_mut(), len) }
        }
    }

    fn get_short_slice(&self) -> &[u8] {
        let len = self.len();

        debug_assert!(
            len <= INLINE_SIZE,
            "cannot get short slice - slice is not inlined",
        );

        // SAFETY: Shall only be called if slice is inlined
        unsafe { slice::from_raw_parts((*self.trailer.short).data.as_ptr(), len) }
    }

    fn get_long_slice(&self) -> &[u8] {
        let len = self.len();

        debug_assert!(
            len > INLINE_SIZE,
            "cannot get long slice - slice is inlined"
        );

        // SAFETY: Shall only be called if slice is heap allocated
        unsafe { slice::from_raw_parts(self.data_ptr(), len) }
    }
}

impl Borrow<[u8]> for ByteView {
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
    #[cfg(target_pointer_width = "64")]
    use std::mem;

    use std::{
        io::{Cursor, ErrorKind, Read, Result},
        thread,
    };

    use super::{ByteView, HeapAllocationHeader, INLINE_SIZE, LongRepr, ShortRepr, Trailer};

    #[test]
    #[expect(
        clippy::indexing_slicing,
        reason = "bounds follow the generated input length"
    )]
    fn inline_boundary_and_shared_subslices() {
        for len in [0, 1, 8, 10, INLINE_SIZE, INLINE_SIZE + 1, 20, 21, 64] {
            let bytes: Vec<_> = (0..=u8::MAX).take(len).collect();
            let view = ByteView::new(&bytes);
            assert_eq!(view.is_inline(), len <= INLINE_SIZE);
            assert_eq!(&*view, bytes);
            for split in 0..=len {
                let fused = ByteView::fused(&bytes[..split], &bytes[split..]);
                assert_eq!(view, fused);
            }
        }

        let bytes: Vec<_> = (0..128).collect();
        let parent = ByteView::new(&bytes);
        let child = parent.slice(13..100).slice(7..70);
        drop(parent);
        thread::scope(|scope| {
            for _ in 0..4 {
                let copy = child.clone();
                scope.spawn(move || {
                    for _ in 0..100 {
                        assert_eq!(&*copy.clone().slice(3..40), &(23..60).collect::<Vec<_>>());
                    }
                });
            }
        });
        assert_eq!(&*child, &bytes[20..83]);
        assert_eq!(child.ref_count(), 1);
    }

    struct InspectingReader;

    #[test]
    #[should_panic(expected = "initializer returned a different buffer")]
    fn initializer_rejects_another_buffer() {
        let _ = ByteView::try_init(64, |_| Ok::<_, ()>(&[0; 64]));
    }

    #[test]
    #[should_panic(expected = "initializer returned a partial buffer")]
    fn initializer_rejects_partial_success() {
        let _ = ByteView::try_init(64, |_| Ok::<_, ()>(&[]));
    }

    #[test]
    fn initializer_can_fail_after_partial_write() {
        let result = ByteView::try_init(64, |buffer| {
            if let Some(byte) = buffer.first_mut() {
                byte.write(1);
            }
            Err(())
        });
        assert!(result.is_err());
    }

    impl Read for InspectingReader {
        fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
            assert!(buffer.iter().all(|byte| *byte == 0));
            buffer.fill(42);
            Ok(buffer.len())
        }
    }

    #[test]
    fn builders_and_readers_only_expose_initialized_bytes() -> Result<()> {
        for len in [0, INLINE_SIZE, INLINE_SIZE + 1, 4096] {
            let mut builder = ByteView::builder(len);
            assert!(builder.iter().all(|byte| *byte == 0));
            builder.fill(42);
            let view = builder.freeze();
            assert_eq!(view, ByteView::from_reader(&mut InspectingReader, len)?);
            assert_eq!(&*view.clone(), vec![42; len]);
        }
        assert!(matches!(
            ByteView::from_reader(&mut Cursor::new([1, 2, 3]), 4096),
            Err(error) if error.kind() == ErrorKind::UnexpectedEof
        ));
        Ok(())
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn memsize() {
        assert_eq!(mem::size_of::<ShortRepr>(), mem::size_of::<LongRepr>());
        assert_eq!(mem::size_of::<Trailer>(), mem::size_of::<LongRepr>());

        assert_eq!(16, mem::size_of::<ByteView>());
        assert_eq!(
            32,
            mem::size_of::<ByteView>() + mem::size_of::<HeapAllocationHeader>()
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
        assert!(bytes.is_inline());
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
    fn from_reader_1() -> Result<()> {
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
        assert!(slice.is_inline());
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn medium_str() {
        let slice = ByteView::from("abcdefabcdef");
        assert_eq!(12, slice.len());
        assert_eq!(&*slice, b"abcdefabcdef");
        assert_eq!(1, slice.ref_count());
        assert!(slice.is_inline());
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn medium_long_str() {
        let slice = ByteView::from("abcdefabcdefabcdabcd");
        assert_eq!(20, slice.len());
        assert_eq!(&*slice, b"abcdefabcdefabcdabcd");
        assert_eq!(1, slice.ref_count());
        assert!(!slice.is_inline());
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn medium_str_clone() {
        let slice = ByteView::from("abcdefabcdef");
        let copy = slice.clone();
        assert_eq!(slice, copy);

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
        assert!(!slice.is_inline());
    }

    #[test]
    fn long_str_clone() {
        let slice = ByteView::from("abcdefabcdefabcdefababcd");
        let copy = slice.clone();
        assert_eq!(slice, copy);

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

        assert_eq!(2, slice.ref_count());

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

        assert_eq!(3, slice.ref_count());

        drop(copy);
        assert_eq!(2, slice.ref_count());

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

        assert_eq!(2, slice.ref_count());

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
