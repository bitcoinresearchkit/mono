use std::{convert::Infallible, marker::PhantomData};

use crate::{ReadableVec, VecIndex, VecValue};

/// Buffered reader that reuses one buffer across chunked `read_into_at` calls.
///
/// Reads are aligned to chunk boundaries, so sequential access decodes each underlying page at
/// most once, even from unaligned positions (repeated `collect_one` calls decode it every time).
///
/// The source must be dense: buffer offsets are positions, so a source with holes
/// (a `MutableVec` with removed values) reads through its own sorted or point reads.
///
/// A bounded cursor ([`ReadableVec::bounded_cursor_at`]) reads only inside its range: refills
/// never copy values before its start or past its end, so a short update reads only what it uses.
pub struct Cursor<
    'a,
    I: VecIndex,
    T: VecValue,
    V: ReadableVec<I, T> + ?Sized = dyn ReadableVec<I, T>,
> {
    source: &'a V,
    buf: Vec<T>,
    /// Absolute position of buf[0] in the source vec.
    buf_start: usize,
    /// Current absolute position in the source vec (used by sequential methods).
    pos: usize,
    chunk_size: usize,
    /// Readable range: `start..len`.
    start: usize,
    len: usize,
    _phantom: PhantomData<I>,
}

impl<'a, I: VecIndex, T: VecValue, V: ReadableVec<I, T> + ?Sized> Cursor<'a, I, T, V> {
    /// Creates a new cursor with the source's preferred chunk size.
    #[inline]
    pub fn new(source: &'a V) -> Self {
        Self::bounded(source, 0, usize::MAX)
    }

    /// Creates a cursor that reads only `[from, to)`, starting at `from`.
    #[inline]
    pub(crate) fn bounded(source: &'a V, from: usize, to: usize) -> Self {
        let len = to.min(source.len());
        let start = from.min(len);
        let chunk_size = source.cursor_chunk_size().max(1);
        Self {
            source,
            buf: Vec::with_capacity(chunk_size.min(len - start)),
            buf_start: start,
            pos: start,
            chunk_size,
            start,
            len,
            _phantom: PhantomData,
        }
    }

    /// Returns the current absolute position.
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Advances the position by `n` without reading.
    #[inline]
    pub fn advance(&mut self, n: usize) {
        self.pos = self.pos.saturating_add(n).min(self.len);
    }

    /// Returns the value at absolute `index`, refilling the aligned chunk holding it when it is
    /// not buffered. Does not move the sequential position used by [`next`](Self::next).
    #[inline]
    pub fn get(&mut self, index: usize) -> Option<T> {
        let local = self.ensure_buffered_at(index)?;
        Some(self.buf[local].clone())
    }

    /// Collects a range into a reusable output buffer, retaining the decoded
    /// chunk for the next call. Supports gaps and backward reads.
    pub fn collect_range_into_at(&mut self, from: usize, to: usize, out: &mut Vec<T>) {
        out.clear();
        let count = self.seek_range(from, to);
        out.reserve(count);
        self.for_each(count, |value| out.push(value));
    }

    /// Visits an absolute range through the retained decoded chunks.
    /// Backward reads are supported; an error leaves the cursor after that value.
    #[inline]
    pub fn try_for_each_range_at<E>(
        &mut self,
        from: usize,
        to: usize,
        mut each: impl FnMut(T) -> Result<(), E>,
    ) -> Result<(), E> {
        let count = self.seek_range(from, to);
        self.try_fold(count, (), |(), value| each(value))
    }

    /// Positions at the readable part of `[from, to)`, returning its length.
    #[inline]
    fn seek_range(&mut self, from: usize, to: usize) -> usize {
        self.pos = from.max(self.start).min(self.len);
        to.min(self.len).saturating_sub(self.pos)
    }

    /// Returns the next value and advances position, or `None` if exhausted.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<T> {
        let local = self.ensure_buffered_at(self.pos)?;
        let val = self.buf[local].clone();
        self.pos += 1;
        Some(val)
    }

    /// Folds over the next `n` values, advancing past them. The last chunk may read past `n`: the
    /// rest stays buffered for later `next()` calls.
    #[inline]
    fn fold<B>(&mut self, n: usize, init: B, mut f: impl FnMut(B, T) -> B) -> B {
        let Ok(value) = self.try_fold(n, init, |acc, value| Ok::<_, Infallible>(f(acc, value)));
        value
    }

    /// Folds over the next `n` values, returning the first error from `f`. Every value passed to
    /// `f` advances the cursor, the failing one included.
    #[inline]
    fn try_fold<B, E>(
        &mut self,
        n: usize,
        init: B,
        mut f: impl FnMut(B, T) -> Result<B, E>,
    ) -> Result<B, E> {
        let target = self.pos.saturating_add(n).min(self.len);
        let mut acc = init;

        while self.pos < target {
            if self.ensure_buffered_at(self.pos).is_none() {
                break;
            }
            let local = self.pos - self.buf_start;
            let local_end = (target - self.buf_start).min(self.buf.len());
            for (offset, value) in self.buf[local..local_end].iter().cloned().enumerate() {
                match f(acc, value) {
                    Ok(value) => acc = value,
                    Err(error) => {
                        self.pos = self.buf_start + local + offset + 1;
                        return Err(error);
                    }
                }
            }
            self.pos = self.buf_start + local_end;
        }

        Ok(acc)
    }

    /// Calls `f` for each of the next `n` elements.
    /// Advances position by the number of elements consumed.
    #[inline]
    pub fn for_each(&mut self, n: usize, mut f: impl FnMut(T)) {
        self.fold(n, (), |(), v| f(v));
    }

    /// Ensures the buffer contains data at `at`.
    /// Reads are aligned to `chunk_size` boundaries so that each underlying
    /// compressed page is decompressed at most once across sequential accesses.
    /// Returns the local index within `buf`, or `None` if out of bounds.
    #[inline]
    fn ensure_buffered_at(&mut self, at: usize) -> Option<usize> {
        if at >= self.len || at < self.start {
            return None;
        }

        let buf_end = self.buf_start + self.buf.len();
        if at >= self.buf_start && at < buf_end {
            return Some(at - self.buf_start);
        }

        // Refill aligned to chunk boundary to avoid cross-page decompression.
        self.buf.clear();
        let aligned = (at / self.chunk_size) * self.chunk_size;
        let end = (aligned + self.chunk_size).min(self.len);
        let aligned = aligned.max(self.start);
        self.buf_start = aligned;
        self.source.read_into_at(aligned, end, &mut self.buf);

        if self.buf.is_empty() {
            None
        } else {
            Some(at - aligned)
        }
    }
}
