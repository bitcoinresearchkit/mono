use std::{convert::Infallible, marker::PhantomData};

use crate::{ReadableVec, VecIndex, VecValue};

/// Buffered reader that reuses an internal buffer across chunked `read_into_at` calls.
///
/// One allocation for the lifetime of the cursor. Ideal for sequential access patterns
/// (iterating tx-indexed vecs, computing rolling windows) where repeated `collect_one`
/// calls would decompress the same page thousands of times.
///
/// Reads are aligned to chunk boundaries so that each underlying page is decompressed
/// at most once, even when access positions don't start at page-aligned offsets.
///
/// # Example
/// ```ignore
/// let mut c = vec.cursor();
/// while let Some(val) = c.next() {
///     // process val
/// }
/// ```
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
    len: usize,
    _phantom: PhantomData<I>,
}

impl<'a, I: VecIndex, T: VecValue, V: ReadableVec<I, T> + ?Sized> Cursor<'a, I, T, V> {
    /// Creates a new cursor with the source's preferred chunk size.
    #[inline]
    pub fn new(source: &'a V) -> Self {
        let len = source.len();
        let chunk_size = source.cursor_chunk_size().max(1);
        Self {
            source,
            buf: Vec::with_capacity(chunk_size.min(len)),
            buf_start: 0,
            pos: 0,
            chunk_size,
            len,
            _phantom: PhantomData,
        }
    }

    /// Returns the current absolute position.
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Returns the number of elements remaining.
    #[inline]
    pub fn remaining(&self) -> usize {
        self.len.saturating_sub(self.pos)
    }

    /// Advances the position by `n` without reading. Cheap — no decompression.
    #[inline]
    pub fn advance(&mut self, n: usize) {
        self.pos = self.pos.saturating_add(n).min(self.len);
    }

    /// Returns the value at absolute `index`, using the buffer when possible.
    ///
    /// If `index` falls within the currently buffered chunk, returns instantly.
    /// Otherwise decompresses the aligned chunk containing `index`.
    /// Does **not** modify the sequential position used by [`next`](Self::next).
    #[inline]
    pub fn get(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }
        let local = self.ensure_buffered_at(index)?;
        Some(self.buf[local].clone())
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

    /// Folds over the next `n` elements with a monomorphized closure.
    /// Advances position by the number of elements consumed.
    ///
    /// Consumes any already-buffered data first, then reads fresh chunks.
    /// The last chunk may read past `n` — leftover data stays in the buffer
    /// for subsequent `next()` calls.
    #[inline]
    pub fn fold<B>(&mut self, n: usize, init: B, mut f: impl FnMut(B, T) -> B) -> B {
        match self.try_fold(n, init, |acc, value| Ok::<_, Infallible>(f(acc, value))) {
            Ok(value) => value,
            Err(error) => match error {},
        }
    }

    /// Fallibly folds over the next `n` elements with a monomorphized closure.
    /// Advances past every value passed to `f`.
    ///
    /// # Errors
    /// Returns the first error from `f`. The failing value is consumed, so the
    /// cursor resumes at the following value.
    #[inline]
    pub fn try_fold<B, E>(
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
        if at >= self.len {
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
        self.buf_start = aligned;
        self.source.read_into_at(aligned, end, &mut self.buf);

        if self.buf.is_empty() {
            None
        } else {
            Some(at - aligned)
        }
    }
}
