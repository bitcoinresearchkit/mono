use std::vec::IntoIter;

use crate::{
    AnyReadableVec, Error, Formattable, ReadableVec, Result, TypedVec, ValueWriter, VecValue,
};

struct IteratorWriter<T> {
    iter: IntoIter<T>,
}

impl<T> ValueWriter for IteratorWriter<T>
where
    T: VecValue + Formattable,
{
    fn write_next(&mut self, buf: &mut String) -> Result<()> {
        if let Some(value) = self.iter.next() {
            value.fmt_csv(buf)?;
            Ok(())
        } else {
            Err(Error::IteratorEnded)
        }
    }
}

/// Type-erased trait for vecs that can produce a boxed row-by-row [`ValueWriter`].
pub trait AnyVecWithWriter: AnyReadableVec {
    /// Create a value writer that can be advanced row by row
    fn create_writer(&self, from: Option<i64>, to: Option<i64>) -> Box<dyn ValueWriter + '_>;
}

impl<V> AnyVecWithWriter for V
where
    V: TypedVec,
    V: ReadableVec<V::I, V::T>,
    V::T: Formattable,
{
    fn create_writer(&self, from: Option<i64>, to: Option<i64>) -> Box<dyn ValueWriter + '_> {
        let from_usize = from.map(|i| self.i64_to_usize(i)).unwrap_or(0);
        let to_usize = to
            .map(|i| self.i64_to_usize(i))
            .unwrap_or_else(|| self.len());

        let values = self.collect_range_at(from_usize, to_usize);
        Box::new(IteratorWriter {
            iter: values.into_iter(),
        })
    }
}
