use crate::{RawStrategy, VecReader, VecValue};

/// Forward cursor over a raw vector reader.
///
/// Unlike [`crate::Cursor`], this reads values directly from the existing mmap
/// and does not allocate or copy through a staging buffer. Like [`VecReader`],
/// it only sees persisted values. Use [`crate::RawRangeCursor`] when a bounded
/// range is known and may be nonresident.
pub struct VecReaderCursor<I, T, S> {
    reader: VecReader<I, T, S>,
    pos: usize,
}

impl<I, T, S> VecReader<I, T, S>
where
    T: VecValue,
    S: RawStrategy<T>,
{
    /// Creates an allocation-free cursor over the persisted values.
    #[inline]
    pub fn cursor(self) -> VecReaderCursor<I, T, S> {
        VecReaderCursor {
            reader: self,
            pos: 0,
        }
    }
}

impl<I, T, S> VecReaderCursor<I, T, S>
where
    T: VecValue,
    S: RawStrategy<T>,
{
    /// Returns the current absolute position.
    #[inline(always)]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Advances the position by `n` without reading.
    #[inline(always)]
    pub fn advance(&mut self, n: usize) {
        self.pos = self.pos.saturating_add(n).min(self.reader.stored_len());
    }

    /// Returns the value at absolute `index` without changing the position.
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<T> {
        self.reader.try_get_at(index)
    }

    /// Returns the next value and advances the position.
    #[inline(always)]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<T> {
        let value = self.reader.try_get_at(self.pos)?;
        self.pos += 1;
        Some(value)
    }
}
