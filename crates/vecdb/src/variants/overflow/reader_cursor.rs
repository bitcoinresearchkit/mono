use crate::{OverflowVecReader, OverflowVecValue, VecIndex};

/// Forward cursor over an [`OverflowVecReader`].
pub struct OverflowVecReaderCursor<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    reader: OverflowVecReader<I, T>,
    position: usize,
}

impl<I, T> OverflowVecReaderCursor<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    /// Returns the current absolute position.
    #[inline(always)]
    pub fn position(&self) -> usize {
        self.position
    }

    /// Returns the number of values remaining.
    #[inline(always)]
    pub fn remaining(&self) -> usize {
        self.reader.len().saturating_sub(self.position)
    }

    /// Advances the cursor by `count` without reading.
    #[inline(always)]
    pub fn advance(&mut self, count: usize) {
        self.position = self.position.saturating_add(count).min(self.reader.len());
    }

    /// Returns the persisted value at `index` without moving the cursor.
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<T> {
        self.reader.try_get_at(index)
    }

    /// Returns the next persisted value and advances the cursor.
    #[inline(always)]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<T> {
        let value = self.reader.try_get_at(self.position)?;
        self.position += 1;
        Some(value)
    }

    /// Folds over the next `count` values and advances the cursor.
    #[inline]
    pub fn fold<B>(&mut self, count: usize, mut value: B, mut fold: impl FnMut(B, T) -> B) -> B {
        let end = self.position.saturating_add(count).min(self.reader.len());
        while self.position < end {
            value = fold(value, self.reader.get_at(self.position));
            self.position += 1;
        }
        value
    }

    /// Calls `f` for each of the next `count` values and advances the cursor.
    #[inline]
    pub fn for_each(&mut self, count: usize, mut f: impl FnMut(T)) {
        self.fold(count, (), |(), value| f(value));
    }
}

impl<I, T> OverflowVecReader<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    /// Creates an allocation-free cursor over persisted values.
    #[inline]
    pub fn cursor(self) -> OverflowVecReaderCursor<I, T> {
        OverflowVecReaderCursor {
            reader: self,
            position: 0,
        }
    }
}
