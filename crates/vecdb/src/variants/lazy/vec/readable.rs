use crate::traits::chunk_folds;
use crate::{AnyVec, ReadableVec, VecIndex, VecValue};

use super::LazyVec;

impl<I, T, S1I, S1T> ReadableVec<I, T> for LazyVec<I, T, S1I, S1T>
where
    I: VecIndex,
    T: VecValue,
    S1I: VecIndex,
    S1T: VecValue,
{
    #[inline]
    fn cursor_chunk_size(&self) -> usize {
        self.source.cursor_chunk_size()
    }

    #[inline]
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        let to = to.min(self.len());
        buf.reserve(to.saturating_sub(from));
        if from < to {
            (self.read)(self, from, to, buf);
        }
    }

    fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
        let to = to.min(self.len());
        if from < to {
            (self.visit)(self, from, to, f);
        }
    }

    #[inline]
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.for_each_chunk_at(from, to, &mut |_, values| {
            values.iter().cloned().for_each(&mut *f);
        });
    }

    #[inline]
    fn fold_range_at<B, F: FnMut(B, T) -> B>(&self, from: usize, to: usize, init: B, f: F) -> B
    where
        Self: Sized,
    {
        chunk_folds::fold(self, from, to, init, f)
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
        // Preserve early-exit semantics: do not evaluate this transform for values after the
        // first error, nor read past its chunk. Infallible reads use the bulk path.
        let compute = self.compute;
        chunk_folds::try_fold(
            &*self.source,
            from,
            to,
            (init, from),
            |(acc, index), value| Ok((f(acc, compute(I::from(index), value))?, index + 1)),
        )
        .map(|(acc, _)| acc)
    }

    #[inline]
    fn collect_one_at(&self, index: usize) -> Option<T> {
        let v = self.source.collect_one_at(index)?;
        Some((self.compute)(I::from(index), v))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        let compute = self.compute;
        // The source is dense, so its values pair with the leading indices.
        let source_vals = self.source.read_sorted_at(indices);
        out.reserve(source_vals.len());
        indices
            .iter()
            .zip(source_vals)
            .for_each(|(&i, v)| out.push(compute(I::from(i), v)));
    }
}
