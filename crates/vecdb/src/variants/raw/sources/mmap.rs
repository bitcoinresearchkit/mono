use std::{convert::Infallible, result::Result, slice};

use rawdb::Region;

use super::{super::super::RawStrategy, VecReader};
use crate::{HEADER_OFFSET, READ_CHUNK_SIZE, VecValue};

/// Mmap-backed reads of a raw vector's persisted values in a bounded range: aligned native
/// layouts are copied in bulk, other layouts decoded per value.
pub struct RawMmapSource<T, S> {
    reader: VecReader<usize, T, S>,
    pos: usize,
    end: usize,
}

impl<T, S> RawMmapSource<T, S>
where
    T: VecValue,
    S: RawStrategy<T>,
{
    const SIZE_OF_T: usize = size_of::<T>();

    /// The range is bounded by `stored_len` and by the bytes the reader holds, so a length read
    /// before a truncation (or a wrong one) never reaches past them.
    pub fn new_from_parts(region: &Region, stored_len: usize, from: usize, to: usize) -> Self {
        let reader = VecReader::from_region(region, stored_len);
        let to = to.min(reader.stored_len());
        Self {
            reader,
            pos: from.min(to),
            end: to,
        }
    }

    pub fn byte_window(&self) -> (*const u8, usize) {
        let byte_position = self.pos * Self::SIZE_OF_T;
        let byte_len = (self.end - self.pos) * Self::SIZE_OF_T;
        (unsafe { self.reader.data.add(byte_position) }, byte_len)
    }

    /// Appends the stored range, copying native-layout values in bulk.
    pub fn read_into(self, output: &mut Vec<T>) {
        if let Some(values) = self.as_slice() {
            output.extend_from_slice(values);
        } else {
            output.reserve(self.end - self.pos);
            let Ok(()) = self.try_fold((), |(), value| {
                output.push(value);
                Ok::<_, Infallible>(())
            });
        }
    }

    /// Visit resident native-layout values without a staging buffer.
    /// Returns false without visiting anything when decoding or file I/O is needed.
    pub(crate) fn try_for_each_chunk(
        region: &Region,
        stored_len: usize,
        from: usize,
        to: usize,
        each: &mut dyn FnMut(usize, &[T]),
    ) -> bool {
        let to = to.min(stored_len);
        if from >= to {
            return true;
        }
        let offset = HEADER_OFFSET + from * Self::SIZE_OF_T;
        let bytes = (to - from) * Self::SIZE_OF_T;
        if !S::IS_NATIVE_LAYOUT || !region.prefers_mmap(offset, bytes) {
            return false;
        }
        let source = Self::new_from_parts(region, stored_len, from, to);
        let Some(values) = source.as_slice() else {
            return false;
        };
        for (index, values) in values.chunks(READ_CHUNK_SIZE).enumerate() {
            each(from + index * READ_CHUNK_SIZE, values);
        }
        true
    }

    /// The stored values in place, for native layouts stored at `T`'s alignment.
    fn as_slice(&self) -> Option<&[T]> {
        if !S::IS_NATIVE_LAYOUT {
            return None;
        }
        // SAFETY: within the reader's bounded range (`pos <= end <= stored_len`).
        let values = unsafe { self.reader.data.add(self.pos * Self::SIZE_OF_T) }.cast::<T>();
        // Values start `HEADER_OFFSET` bytes into a page-aligned region: only larger alignments miss.
        if !values.is_aligned() {
            return None;
        }
        // SAFETY: aligned native-layout values have T's stored representation. The bounded
        // range remains valid while this source owns the mmap reader.
        Some(unsafe { slice::from_raw_parts(values, self.end - self.pos) })
    }

    #[inline(always)]
    pub fn try_fold<B, E, F: FnMut(B, T) -> Result<B, E>>(self, init: B, mut f: F) -> Result<B, E> {
        let ptr = self.reader.data;
        let mut byte_off = self.pos * Self::SIZE_OF_T;
        let end_byte = self.end * Self::SIZE_OF_T;
        let mut acc = init;
        while byte_off < end_byte {
            acc = f(acc, unsafe { S::read_from_ptr(ptr, byte_off) })?;
            byte_off += Self::SIZE_OF_T;
        }
        Ok(acc)
    }
}
