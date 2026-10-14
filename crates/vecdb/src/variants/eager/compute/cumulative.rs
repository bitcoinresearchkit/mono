use std::ops::AddAssign;

use brk_exit::Exit;

use super::super::EagerVec;
use crate::{ReadableVec, Result, StoredVec, VecValue, WritableVec};

impl<V> EagerVec<V>
where
    V: StoredVec,
{
    /// Compute cumulative sum from a source vec.
    ///
    /// Each value in the result is the sum of all values from the source up to
    /// and including that index.
    pub fn compute_cumulative<S>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, S>,
        exit: &Exit,
    ) -> Result<()>
    where
        S: VecValue + Into<V::T>,
        V::T: Default + AddAssign + Copy,
    {
        self.compute_to(
            max_from,
            source.len(),
            source.version(),
            |this, range| {
                let mut cumulative_val = this.collect_last().unwrap_or_default();
                source.try_fold_range_at(range.start, range.end, (), |(), v: S| {
                    cumulative_val += v.into();
                    this.push(cumulative_val);
                    Ok(())
                })
            },
            exit,
        )
    }

    /// Compute cumulative sum from a custom binary transform of two source vecs.
    ///
    /// Each value in the result is the cumulative sum of `transform(source1[i], source2[i])`
    /// for all indices up to and including i.
    pub fn compute_cumulative_transformed_binary<S1, S2, F>(
        &mut self,
        max_from: V::I,
        source1: &impl ReadableVec<V::I, S1>,
        source2: &impl ReadableVec<V::I, S2>,
        mut transform: F,
        exit: &Exit,
    ) -> Result<()>
    where
        S1: VecValue,
        S2: VecValue,
        V::T: Default + AddAssign + Copy,
        F: FnMut(S1, S2) -> V::T,
    {
        self.compute_to(
            max_from,
            source1.len().min(source2.len()),
            source1.version() + source2.version(),
            |this, range| {
                let mut cumulative_val = this.collect_last().unwrap_or_default();
                let batch2 = source2.collect_range_at(range.start, range.end);
                let mut iter2 = batch2.into_iter();

                source1.try_fold_range_at(range.start, range.end, (), |(), v1: S1| {
                    let v2 = iter2.next().unwrap();
                    cumulative_val += transform(v1, v2);
                    this.push(cumulative_val);
                    Ok(())
                })
            },
            exit,
        )
    }
}
