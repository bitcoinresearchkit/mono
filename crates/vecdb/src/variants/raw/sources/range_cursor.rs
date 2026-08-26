use std::ptr::NonNull;

use rawdb::Region;

use crate::{HEADER_OFFSET, VecIndex, VecValue};

use super::super::RawStrategy;
use super::{RawIoSource, RawMmapSource};

enum Owner<'a, I, T, S> {
    Mmap { _source: RawMmapSource<I, T, S> },
    Io { source: RawIoSource<'a, I, T, S> },
}

/// Forward cursor over a bounded persisted range of a raw vector.
///
/// The complete range lets the cursor choose once between direct mmap access
/// for resident data and buffered file I/O for a cold sequential scan.
pub struct RawRangeCursor<'a, I, T, S> {
    owner: Owner<'a, I, T, S>,
    window: *const u8,
    window_position: usize,
    window_len: usize,
    unbuffered_bytes: usize,
    range_end: usize,
}

impl<'a, I, T, S> RawRangeCursor<'a, I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    const SIZE_OF_T: usize = size_of::<T>();

    pub(crate) fn new(region: &'a Region, stored_len: usize, from: usize, to: usize) -> Self {
        let from = from.min(stored_len);
        let to = to.min(stored_len).max(from);
        let bytes = (to - from) * Self::SIZE_OF_T;
        let offset = HEADER_OFFSET + from * Self::SIZE_OF_T;

        if region.prefers_mmap(offset, bytes) {
            let source = RawMmapSource::new_from_parts(region, stored_len, from, to);
            let (window, window_len) = source.byte_window();
            Self {
                owner: Owner::Mmap { _source: source },
                window,
                window_position: 0,
                window_len,
                unbuffered_bytes: 0,
                range_end: to,
            }
        } else {
            Self {
                owner: Owner::Io {
                    source: RawIoSource::new_from_parts(region, stored_len, from, to),
                },
                window: NonNull::dangling().as_ptr(),
                window_position: 0,
                window_len: 0,
                unbuffered_bytes: bytes,
                range_end: to,
            }
        }
    }

    /// Returns the current absolute vector position.
    #[inline(always)]
    pub fn position(&self) -> usize {
        self.range_end - self.remaining()
    }

    /// Returns the number of values remaining in the declared range.
    #[inline(always)]
    pub fn remaining(&self) -> usize {
        (self.unbuffered_bytes + self.window_len - self.window_position) / Self::SIZE_OF_T
    }

    #[cold]
    #[inline(never)]
    fn refill(&mut self) -> bool {
        if self.unbuffered_bytes == 0 {
            return false;
        }
        let Owner::Io { source } = &mut self.owner else {
            unreachable!();
        };
        let Some((window, bytes)) = source.read_window() else {
            return false;
        };
        self.window = window;
        self.window_position = 0;
        self.window_len = bytes;
        self.unbuffered_bytes -= bytes;
        true
    }

    /// Advances within the declared range without decoding values.
    pub fn advance(&mut self, n: usize) {
        let values = n.min(self.remaining());
        let bytes = values * Self::SIZE_OF_T;
        let available = self.window_len - self.window_position;
        if bytes <= available {
            self.window_position += bytes;
            return;
        }

        let skipped_file_bytes = bytes - available;
        let Owner::Io { source } = &mut self.owner else {
            unreachable!();
        };
        source.skip_file_bytes(skipped_file_bytes);
        self.window_position = 0;
        self.window_len = 0;
        self.unbuffered_bytes -= skipped_file_bytes;
    }

    /// Returns the next value and advances the position.
    #[inline(always)]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<T> {
        if self.window_position == self.window_len && !self.refill() {
            return None;
        }
        let value = unsafe { S::read_from_ptr(self.window, self.window_position) };
        self.window_position += Self::SIZE_OF_T;
        Some(value)
    }

    /// Folds over up to the next `n` values and advances the position.
    #[inline]
    pub fn fold<B>(&mut self, n: usize, mut value: B, mut fold: impl FnMut(B, T) -> B) -> B {
        let mut remaining = n.min(self.remaining());
        while remaining > 0 {
            if self.window_position == self.window_len && !self.refill() {
                break;
            }

            let available = (self.window_len - self.window_position) / Self::SIZE_OF_T;
            let values = remaining.min(available);
            let end = self.window_position + values * Self::SIZE_OF_T;
            while self.window_position < end {
                value = fold(value, unsafe {
                    S::read_from_ptr(self.window, self.window_position)
                });
                self.window_position += Self::SIZE_OF_T;
            }
            remaining -= values;
        }
        value
    }

    /// Calls `f` for up to the next `n` values and advances the position.
    #[inline]
    pub fn for_each(&mut self, n: usize, mut f: impl FnMut(T)) {
        for _ in 0..n.min(self.remaining()) {
            f(self.next().unwrap());
        }
    }
}
