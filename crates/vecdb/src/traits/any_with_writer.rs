use crate::{
    AnyReadableVec, Error, Formattable, ReadableVec, Result, TypedVec, ValueWriter, VecValue,
};

use super::chunk_folds::chunk_end;

/// Streams a range through one reusable chunk buffer.
struct ChunkWriter<'a, V, T> {
    vec: &'a V,
    at: usize,
    to: usize,
    chunk_size: usize,
    values: Vec<T>,
    next: usize,
}

impl<V> ValueWriter for ChunkWriter<'_, V, V::T>
where
    V: TypedVec + ReadableVec<V::I, V::T>,
    V::T: Formattable,
{
    fn write_next(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        while self.next == self.values.len() {
            if self.at >= self.to {
                return Err(Error::IteratorEnded);
            }
            let end = chunk_end(self.at, self.chunk_size, self.to);
            self.values.clear();
            self.next = 0;
            self.vec.read_into_at(self.at, end, &mut self.values);
            self.at = end;
        }
        let value = &self.values[self.next];
        self.next += 1;
        value.fmt_csv(buf);
        Ok(())
    }
}

/// Type-erased trait for vecs that can produce a boxed value-by-value [`ValueWriter`].
pub trait AnyVecWithWriter: AnyReadableVec {
    /// Create a value writer that can be advanced value by value
    fn create_writer(&self, from: Option<i64>, to: Option<i64>) -> Box<dyn ValueWriter + '_>;
}

impl<V> AnyVecWithWriter for V
where
    V: TypedVec,
    V: ReadableVec<V::I, V::T>,
    V::T: VecValue + Formattable,
{
    fn create_writer(&self, from: Option<i64>, to: Option<i64>) -> Box<dyn ValueWriter + '_> {
        let at = from.map(|i| self.i64_to_usize(i)).unwrap_or(0);
        let to = to
            .map(|i| self.i64_to_usize(i))
            .unwrap_or_else(|| self.len());
        let chunk_size = self.cursor_chunk_size().max(1);
        Box::new(ChunkWriter {
            vec: self,
            at,
            to,
            chunk_size,
            values: Vec::with_capacity(chunk_size.min(to.saturating_sub(at))),
            next: 0,
        })
    }
}
