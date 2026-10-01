use std::ops::Range;

use brk_exit::Exit;

use super::super::EagerVec;
use crate::{
    AnyStoredVec, AnyVec, BinaryTransform, Error, ReadableVec, Result, StoredVec, VecIndex,
    VecValue, Version, WritableVec,
};

impl<V> EagerVec<V>
where
    V: StoredVec,
{
    /// Computes a fixed output range from source data prepared in bounded batches.
    /// Clamps the retained prefix to `to` and persists it even without new rows.
    /// The callback must append exactly one value for every index in its range.
    pub fn compute_batched_to<F>(
        &mut self,
        max_from: V::I,
        to: usize,
        version: Version,
        batch_size: usize,
        mut compute: F,
        exit: &Exit,
    ) -> Result<()>
    where
        F: FnMut(&mut Self, Range<usize>) -> Result<()>,
    {
        if batch_size == 0 {
            return Err(Error::InvalidArgument(
                "EagerVec batch size must be greater than zero",
            ));
        }

        {
            let _lock = exit.lock();
            self.validate_computed_version_or_reset(version)?;
            self.truncate_if_needed_at(max_from.to_usize().min(to))?;
            self.write()?;
        }

        while self.len() < to {
            let from = self.len();
            let end = from.saturating_add(batch_size).min(to);
            compute(self, from..end)?;
            if self.len() != end {
                return Err(Error::InvalidArgument(
                    "EagerVec batch callback must append one value per index",
                ));
            }

            let _lock = exit.lock();
            self.write()?;
        }

        Ok(())
    }

    pub fn compute_transform<A, F>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, A>,
        mut t: F,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        F: FnMut((V::I, A, &Self)) -> (V::I, V::T),
    {
        let max_from = V::I::from(max_from.to_usize().min(source.len()));
        self.compute_init(source.version(), max_from, exit, |this| {
            let skip = this.len();
            let end = this.batch_end(source.len());
            if skip >= end {
                return Ok(());
            }

            let mut i = skip;
            source.fold_range_at(skip, end, (), |(), b: A| {
                let (idx, v) = t((V::I::from(i), b, &*this));
                i += 1;
                this.debug_checked_push(idx, v);
            });
            Ok(())
        })
    }

    pub fn compute_transform2<A, B, F>(
        &mut self,
        max_from: V::I,
        other1: &impl ReadableVec<V::I, A>,
        other2: &impl ReadableVec<V::I, B>,
        t: F,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        B: VecValue,
        F: FnMut((V::I, A, B, &Self)) -> (V::I, V::T),
    {
        let batch_size = self.batch_capacity();
        self.compute_transform2_batched(max_from, other1, other2, batch_size, t, exit)
    }

    pub fn compute_transform2_batched<A, B, F>(
        &mut self,
        max_from: V::I,
        other1: &impl ReadableVec<V::I, A>,
        other2: &impl ReadableVec<V::I, B>,
        batch_size: usize,
        mut t: F,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        B: VecValue,
        F: FnMut((V::I, A, B, &Self)) -> (V::I, V::T),
    {
        let source_end = other1.len().min(other2.len());
        self.compute_batched_to(
            max_from,
            source_end,
            other1.version() + other2.version(),
            batch_size,
            |this, range| {
                let batch2 = other2.collect_range_at(range.start, range.end);
                let mut iter2 = batch2.into_iter();
                let mut i = range.start;

                other1.fold_range_at(range.start, range.end, (), |(), b: A| {
                    let (idx, v) = t((V::I::from(i), b, iter2.next().unwrap(), &*this));
                    i += 1;
                    this.debug_checked_push(idx, v);
                });
                Ok(())
            },
            exit,
        )
    }

    pub fn compute_binary<A, B, F>(
        &mut self,
        max_from: V::I,
        source1: &impl ReadableVec<V::I, A>,
        source2: &impl ReadableVec<V::I, B>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        B: VecValue,
        F: BinaryTransform<A, B, V::T>,
    {
        self.compute_transform2(
            max_from,
            source1,
            source2,
            |(h, a, b, ..)| (h, F::apply(a, b)),
            exit,
        )
    }

    pub fn compute_transform3<A, B, C, F>(
        &mut self,
        max_from: V::I,
        other1: &impl ReadableVec<V::I, A>,
        other2: &impl ReadableVec<V::I, B>,
        other3: &impl ReadableVec<V::I, C>,
        mut t: F,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        B: VecValue,
        C: VecValue,
        F: FnMut((V::I, A, B, C, &Self)) -> (V::I, V::T),
    {
        self.compute_init(
            other1.version() + other2.version() + other3.version(),
            max_from,
            exit,
            |this| {
                let skip = this.len();
                let source_end = other1.len().min(other2.len()).min(other3.len());
                let end = this.batch_end(source_end);
                if skip >= end {
                    return Ok(());
                }

                let batch2 = other2.collect_range_at(skip, end);
                let batch3 = other3.collect_range_at(skip, end);
                let mut iter2 = batch2.into_iter();
                let mut iter3 = batch3.into_iter();
                let mut i = skip;

                other1.fold_range_at(skip, end, (), |(), b: A| {
                    let (idx, v) = t((
                        V::I::from(i),
                        b,
                        iter2.next().unwrap(),
                        iter3.next().unwrap(),
                        &*this,
                    ));
                    i += 1;
                    this.debug_checked_push(idx, v);
                });
                Ok(())
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compute_transform4<A, B, C, D, F>(
        &mut self,
        max_from: V::I,
        other1: &impl ReadableVec<V::I, A>,
        other2: &impl ReadableVec<V::I, B>,
        other3: &impl ReadableVec<V::I, C>,
        other4: &impl ReadableVec<V::I, D>,
        mut t: F,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        B: VecValue,
        C: VecValue,
        D: VecValue,
        F: FnMut((V::I, A, B, C, D, &Self)) -> (V::I, V::T),
    {
        self.compute_init(
            other1.version() + other2.version() + other3.version() + other4.version(),
            max_from,
            exit,
            |this| {
                let skip = this.len();
                let source_end = other1
                    .len()
                    .min(other2.len())
                    .min(other3.len())
                    .min(other4.len());
                let end = this.batch_end(source_end);
                if skip >= end {
                    return Ok(());
                }

                let batch2 = other2.collect_range_at(skip, end);
                let batch3 = other3.collect_range_at(skip, end);
                let batch4 = other4.collect_range_at(skip, end);
                let mut iter2 = batch2.into_iter();
                let mut iter3 = batch3.into_iter();
                let mut iter4 = batch4.into_iter();
                let mut i = skip;

                other1.fold_range_at(skip, end, (), |(), b: A| {
                    let (idx, v) = t((
                        V::I::from(i),
                        b,
                        iter2.next().unwrap(),
                        iter3.next().unwrap(),
                        iter4.next().unwrap(),
                        &*this,
                    ));
                    i += 1;
                    this.debug_checked_push(idx, v);
                });
                Ok(())
            },
        )
    }
}
