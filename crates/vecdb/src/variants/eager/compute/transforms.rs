use std::ops::Range;

use crate::{
    AnyStoredVec, AnyVec, BinaryTransform, Cursor, Error, ReadableVec, StoredVec, VecIndex,
    VecValue, Version, WritableVec,
};
use brk_exit::Exit;

use super::super::EagerVec;

impl<V> EagerVec<V>
where
    V: StoredVec,
{
    /// Computes a fixed output range from source data prepared in bounded batches.
    /// The callback must append exactly one value for every index in its range.
    pub fn compute_batched_to<F>(
        &mut self,
        max_from: V::I,
        to: usize,
        version: Version,
        batch_size: usize,
        mut compute: F,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        F: FnMut(&mut Self, Range<usize>) -> crate::Result<()>,
    {
        if batch_size == 0 {
            return Err(Error::InvalidArgument(
                "EagerVec batch size must be greater than zero",
            ));
        }

        self.validate_computed_version_or_reset(version)?;
        self.truncate_if_needed(max_from)?;

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

        if self.is_dirty() {
            let _lock = exit.lock();
            self.write()?;
        }

        Ok(())
    }

    pub fn compute_to<F>(
        &mut self,
        max_from: V::I,
        to: usize,
        version: Version,
        mut t: F,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        F: FnMut(V::I) -> (V::I, V::T),
    {
        self.compute_init(version, max_from, exit, |this| {
            let from = this.len();
            let end = this.batch_end(to);
            if from >= end {
                return Ok(());
            }

            for i in from..end {
                let (idx, val) = t(V::I::from(i));
                this.debug_checked_push(idx, val);
            }

            Ok(())
        })
    }

    pub fn compute_range<A, F>(
        &mut self,
        max_from: V::I,
        other: &impl ReadableVec<V::I, A>,
        t: F,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        A: VecValue,
        F: FnMut(V::I) -> (V::I, V::T),
    {
        self.compute_to(max_from, other.len(), other.version(), t, exit)
    }

    pub fn compute_from_index<A>(
        &mut self,
        max_from: V::I,
        other: &impl ReadableVec<V::I, A>,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        V::T: From<V::I>,
        A: VecValue,
    {
        self.compute_to(
            max_from,
            other.len(),
            other.version(),
            |i| (i, V::T::from(i)),
            exit,
        )
    }

    pub fn compute_transform<A, F>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, A>,
        mut t: F,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        A: VecValue,
        F: FnMut((V::I, A, &Self)) -> (V::I, V::T),
    {
        self.compute_init(source.version(), max_from, exit, |this| {
            let skip = this.len();
            let end = this.batch_end(source.len());
            if skip >= end {
                return Ok(());
            }

            let mut i = skip;
            source.try_fold_range_at(skip, end, (), |(), b: A| {
                let (idx, v) = t((V::I::from(i), b, &*this));
                i += 1;
                this.debug_checked_push(idx, v);
                Ok(())
            })
        })
    }

    pub fn compute_transform2<A, B, F>(
        &mut self,
        max_from: V::I,
        other1: &impl ReadableVec<V::I, A>,
        other2: &impl ReadableVec<V::I, B>,
        t: F,
        exit: &Exit,
    ) -> crate::Result<()>
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
    ) -> crate::Result<()>
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

                other1.try_fold_range_at(range.start, range.end, (), |(), b: A| {
                    let (idx, v) = t((V::I::from(i), b, iter2.next().unwrap(), &*this));
                    i += 1;
                    this.debug_checked_push(idx, v);
                    Ok(())
                })
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
    ) -> crate::Result<()>
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
    ) -> crate::Result<()>
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

                other1.try_fold_range_at(skip, end, (), |(), b: A| {
                    let (idx, v) = t((
                        V::I::from(i),
                        b,
                        iter2.next().unwrap(),
                        iter3.next().unwrap(),
                        &*this,
                    ));
                    i += 1;
                    this.debug_checked_push(idx, v);
                    Ok(())
                })
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
    ) -> crate::Result<()>
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

                other1.try_fold_range_at(skip, end, (), |(), b: A| {
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
                    Ok(())
                })
            },
        )
    }

    /// Compute values through an indirection: for each index i, produces
    /// `source2[source1[i]]`. Keys from source1 must be monotonically increasing
    /// so that source2 access is sequential (cursor-friendly).
    pub fn compute_indirect_sequential<A>(
        &mut self,
        max_from: V::I,
        source1: &impl ReadableVec<V::I, A>,
        source2: &impl ReadableVec<A, V::T>,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        A: VecValue + VecIndex,
    {
        // Cursor persists across batches to avoid re-decompressing pages.
        let mut cursor = Cursor::new(source2);
        let mut cursor_pos: usize = 0;
        let mut last_v: Option<V::T> = None;

        self.compute_init(
            source1.version() + source2.version(),
            max_from,
            exit,
            |this| {
                let skip = this.len();
                let end = this.batch_end(source1.len());
                if skip >= end {
                    return Ok(());
                }

                let keys: Vec<A> = source1.collect_range_at(skip, end);

                for key in keys {
                    let key_pos = key.to_usize();
                    let v = if key_pos >= cursor_pos {
                        if key_pos > cursor_pos {
                            cursor.advance(key_pos - cursor_pos);
                        }
                        let v = cursor.next().unwrap();
                        cursor_pos = key_pos + 1;
                        v
                    } else {
                        // Duplicate key from gap-filled periods — reuse previous value
                        last_v.clone().unwrap()
                    };
                    last_v = Some(v.clone());
                    this.push(v);
                }

                Ok(())
            },
        )
    }

    pub fn compute_first_per_index(
        &mut self,
        max_from: V::T,
        other: &impl ReadableVec<V::T, V::I>,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        V::I: VecValue + VecIndex,
        V::T: VecIndex,
    {
        self.validate_computed_version_or_reset(other.version())?;

        self.repeat_until_complete(exit, |this| {
            let skip = if this.len() > 0 {
                this.collect_last()
                    .unwrap()
                    .to_usize()
                    .min(max_from.to_usize())
            } else {
                0
            };

            let end = this.batch_end(other.len());
            if skip >= end {
                return Ok(());
            }

            let mut prev_i = None;
            let batch = other.collect_range_at(skip, end);

            if let Some(&first_target) = batch.first() {
                this.truncate_if_needed(first_target)?;
            }

            for (j, i) in batch.into_iter().enumerate() {
                let v = V::T::from(skip + j);
                debug_assert!(prev_i.is_none_or(|prev| prev <= i));
                if prev_i.is_some_and(|prev_i| prev_i == i) {
                    continue;
                }
                if this.collect_one(i).is_none_or(|old_v| old_v > v) {
                    // Pad gaps with the current value so empty periods get zero-length ranges
                    let i_usize = i.to_usize();
                    while this.len() < i_usize {
                        this.push(v);
                    }
                    this.push(v);
                }
                prev_i.replace(i);
            }

            Ok(())
        })
    }
}
