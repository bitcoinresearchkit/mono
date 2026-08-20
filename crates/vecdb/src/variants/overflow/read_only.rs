use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    AnyVec, BytesStrategy, BytesVecReader, OverflowVecReader, OverflowVecValue, ReadOnlyMutableVec,
    ReadOnlyRawVec, ReadableVec, SharedLen, TypedVec, VecIndex, Version, short_type_name, unlikely,
};

use super::DECODE_CHUNK_SIZE;

/// Lean read-only clone of an [`OverflowVec`](crate::OverflowVec).
pub struct ReadOnlyOverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    compact: ReadOnlyMutableVec<ReadOnlyRawVec<I, T::Compact, BytesStrategy<T::Compact>>>,
    overflow: ReadOnlyMutableVec<ReadOnlyRawVec<usize, T, BytesStrategy<T>>>,
    visible_len: SharedLen,
    gate: Arc<RwLock<()>>,
}

impl<I, T> ReadOnlyOverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    #[doc(hidden)]
    pub fn new(
        compact: ReadOnlyMutableVec<ReadOnlyRawVec<I, T::Compact, BytesStrategy<T::Compact>>>,
        overflow: ReadOnlyMutableVec<ReadOnlyRawVec<usize, T, BytesStrategy<T>>>,
        visible_len: SharedLen,
        gate: Arc<RwLock<()>>,
    ) -> Self {
        Self {
            compact,
            overflow,
            visible_len,
            gate,
        }
    }

    pub fn reader(&self) -> OverflowVecReader<I, T> {
        let _guard = self.gate.read();
        OverflowVecReader::from_read_only(&self.compact, &self.overflow)
    }

    #[inline(always)]
    fn decode(&self, compact: T::Compact) -> T {
        let overflow_index = T::overflow_index(compact);
        if unlikely(overflow_index.is_some()) {
            self.overflow
                .collect_one_at(overflow_index.unwrap())
                .expect("OverflowVec pointer must reference a stored value")
        } else {
            T::from_compact(compact)
        }
    }

    #[inline(always)]
    fn decode_with_reader(compact: T::Compact, overflow: &BytesVecReader<usize, T>) -> T {
        let overflow_index = T::overflow_index(compact);
        if unlikely(overflow_index.is_some()) {
            overflow
                .try_get_at(overflow_index.unwrap())
                .expect("OverflowVec pointer must reference a stored value")
        } else {
            T::from_compact(compact)
        }
    }
}

impl<I, T> Clone for ReadOnlyOverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    fn clone(&self) -> Self {
        Self {
            compact: self.compact.clone(),
            overflow: self.overflow.clone(),
            visible_len: self.visible_len.clone(),
            gate: Arc::clone(&self.gate),
        }
    }
}

impl<I, T> AnyVec for ReadOnlyOverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    fn version(&self) -> Version {
        self.compact.version()
    }

    fn name(&self) -> &str {
        self.compact.name()
    }

    fn len(&self) -> usize {
        self.visible_len.get()
    }

    fn is_mutable(&self) -> bool {
        true
    }

    fn index_type_to_string(&self) -> &'static str {
        I::to_string()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<T>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<T>()
    }

    fn region_names(&self) -> Vec<String> {
        let mut names = self.compact.region_names();
        names.extend(self.overflow.region_names());
        names
    }
}

impl<I, T> TypedVec for ReadOnlyOverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    type I = I;
    type T = T;
}

impl<I, T> ReadableVec<I, T> for ReadOnlyOverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue,
{
    #[inline(always)]
    fn collect_one_at(&self, index: usize) -> Option<T> {
        let _guard = self.gate.read();
        if index >= self.visible_len.get() {
            return None;
        }
        self.compact
            .collect_one_at(index)
            .map(|compact| self.decode(compact))
    }

    #[inline]
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        let _guard = self.gate.read();
        let len = self.visible_len.get();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return;
        }

        let overflow = BytesVecReader::new(self.overflow.reader());
        buf.reserve(to - from);
        if to - from > DECODE_CHUNK_SIZE {
            let mut compact = Vec::with_capacity(DECODE_CHUNK_SIZE);
            for chunk_from in (from..to).step_by(DECODE_CHUNK_SIZE) {
                compact.clear();
                self.compact.read_into_at(
                    chunk_from,
                    (chunk_from + DECODE_CHUNK_SIZE).min(to),
                    &mut compact,
                );
                buf.extend(
                    compact
                        .iter()
                        .copied()
                        .map(|value| Self::decode_with_reader(value, &overflow)),
                );
            }
            return;
        }

        self.compact.for_each_range_at(from, to, |compact| {
            buf.push(Self::decode_with_reader(compact, &overflow));
        });
    }

    #[inline]
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        let _guard = self.gate.read();
        let to = to.min(self.visible_len.get());
        let overflow = BytesVecReader::new(self.overflow.reader());
        self.compact
            .for_each_range_dyn_at(from, to, &mut |compact| {
                f(Self::decode_with_reader(compact, &overflow));
            });
    }

    #[inline]
    fn fold_range_at<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> B {
        let _guard = self.gate.read();
        let to = to.min(self.visible_len.get());
        let overflow = BytesVecReader::new(self.overflow.reader());
        self.compact.fold_range_at(from, to, init, |acc, compact| {
            f(acc, Self::decode_with_reader(compact, &overflow))
        })
    }

    #[inline]
    fn try_fold_range_at<B, E, F: FnMut(B, T) -> std::result::Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> std::result::Result<B, E> {
        let _guard = self.gate.read();
        let to = to.min(self.visible_len.get());
        let overflow = BytesVecReader::new(self.overflow.reader());
        self.compact
            .try_fold_range_at(from, to, init, |acc, compact| {
                f(acc, Self::decode_with_reader(compact, &overflow))
            })
    }
}
