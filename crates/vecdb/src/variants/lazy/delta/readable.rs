use std::convert::Infallible;

use crate::{AnyVec, ReadableVec, SparseRead, VecIndex, VecValue, before_window};

use super::{DeltaOp, LazyDeltaVec};

impl<I, S, T, Op> ReadableVec<I, T> for LazyDeltaVec<I, S, T, Op>
where
    I: VecIndex,
    S: VecValue + Default,
    T: VecValue,
    Op: DeltaOp<S, T>,
{
    fn cursor_chunk_size(&self) -> usize {
        self.source.cursor_chunk_size()
    }

    #[inline]
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        let Ok(()) = self.try_for_each_batch(from, to, |at, starts, batch| {
            buf.extend(Self::transformed_values(at, starts, batch));
            Ok::<_, Infallible>(())
        });
    }

    /// Lends each batch's values through one reused buffer.
    fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
        let mut output = Vec::new();
        let Ok(()) = self.try_for_each_batch(from, to, |at, starts, batch| {
            output.clear();
            output.extend(Self::transformed_values(at, starts, batch));
            f(at, &output);
            Ok::<_, Infallible>(())
        });
    }

    #[inline]
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.fold_range_at(from, to, (), |(), value| f(value));
    }

    #[inline]
    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> Result<B, E>
    where
        Self: Sized,
    {
        let mut acc = Some(init);
        self.try_for_each_batch(from, to, |at, starts, batch| {
            acc = Some(
                Self::transformed_values(at, starts, batch)
                    .try_fold(acc.take().unwrap(), &mut f)?,
            );
            Ok(())
        })?;
        Ok(acc.unwrap())
    }

    #[inline]
    fn collect_one_at(&self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }
        let start = self.window_starts.collect_one_at(index)?.to_usize();
        let current = self.source.collect_one_at(index)?;
        let before = match before_window(start) {
            Some(index) => self.source.collect_one_at(index)?,
            None => S::default(),
        };
        Some(Op::combine(current, before, Op::count(index, start)))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        if indices.is_empty() {
            return;
        }
        let len = self.len();
        let indices = &indices[..indices.partition_point(|&index| index < len)];
        let starts = self.window_starts.read_sorted_at(indices);
        let mut starts_iter = starts.iter();
        let values = SparseRead::new(&*self.source, indices, |_| {
            before_window(starts_iter.next().unwrap().to_usize())
        });
        out.reserve(indices.len());
        for (slot, &index) in indices.iter().enumerate() {
            let start = starts[slot].to_usize();
            out.push(Op::combine(
                values.current(slot),
                values.previous(slot).unwrap_or_default(),
                Op::count(index, start),
            ));
        }
    }
}
