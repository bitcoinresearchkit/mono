use std::convert::Infallible;

use crate::{AnyVec, ReadableVec, SparseRead, VecIndex, VecValue};

use super::{DeltaOp, LazyDeltaVec};

impl<I, S, T, Op> ReadableVec<I, T> for LazyDeltaVec<I, S, T, Op>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
    Op: DeltaOp<S, T>,
{
    fn cursor_chunk_size(&self) -> usize {
        self.source.cursor_chunk_size()
    }

    #[inline]
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        let Ok(()) =
            self.try_for_each_batch(from, to, |at, current, starts, previous_from, previous| {
                buf.extend(Self::transformed_values(
                    at,
                    current,
                    starts,
                    previous_from,
                    previous,
                ));
                Ok::<_, Infallible>(())
            });
    }

    fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
        self.for_each_transformed_chunk(from, to, f);
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
        self.try_for_each_batch(from, to, |at, current, starts, previous_from, previous| {
            acc = Some(
                Self::transformed_values(at, current, starts, previous_from, previous)
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
        let ago = match Op::ago_index(start) {
            Some(idx) => self.source.collect_one_at(idx)?,
            None => Op::ago_default(),
        };
        Some(Op::combine(current, ago, Op::count(index, start)))
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
            Op::ago_index(starts_iter.next().unwrap().to_usize())
        });
        out.reserve(indices.len());
        for (slot, &index) in indices.iter().enumerate() {
            let start = starts[slot].to_usize();
            out.push(Op::combine(
                values.current(slot),
                values.previous(slot).unwrap_or_else(Op::ago_default),
                Op::count(index, start),
            ));
        }
    }
}
