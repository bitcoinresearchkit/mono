use std::{result::Result, sync::Arc};

use parking_lot::RwLock;

use super::decode::{self, decode};
use crate::{
    AnyVec, BytesStrategy, OverflowVecReader, OverflowVecValue, ReadOnlyMutableVec, ReadOnlyRawVec,
    ReadableVec, SharedLen, TypedVec, VecIndex, Version,
};

#[derive(Clone)]
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
    pub(crate) fn new(
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
        let _guard = self.gate.read_recursive();
        OverflowVecReader::from_read_only(&self.compact, &self.overflow)
    }

    #[inline(always)]
    fn decode(&self, compact: T::Compact) -> T {
        decode::<T>(compact, |index| {
            self.overflow
                .collect_one_at(index)
                .expect("OverflowVec pointer must reference a stored value")
        })
    }

    /// Reads overflow values through one reader of the persisted sidecar.
    fn sidecar(&self) -> impl Fn(usize) -> T {
        let overflow = self.overflow.reader();
        move |index| {
            overflow
                .try_get_at(index)
                .expect("OverflowVec pointer must reference a stored value")
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

    crate::any_vec_type_methods!(I, T);

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
        let _guard = self.gate.read_recursive();
        if index >= self.visible_len.get() {
            return None;
        }
        self.compact
            .collect_one_at(index)
            .map(|compact| self.decode(compact))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        let _guard = self.gate.read_recursive();
        let len = self.visible_len.get();
        decode::read_sorted_into(&self.compact, len, indices, out, || self.sidecar());
    }

    #[inline]
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        let _guard = self.gate.read_recursive();
        let len = self.visible_len.get();
        decode::read_into(&self.compact, len, from, to, buf, || self.sidecar());
    }

    #[inline]
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        let _guard = self.gate.read_recursive();
        let to = to.min(self.visible_len.get());
        let overflow = self.sidecar();
        self.compact
            .for_each_range_dyn_at(from, to, &mut |compact| f(decode(compact, &overflow)));
    }

    #[inline]
    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> Result<B, E> {
        let _guard = self.gate.read_recursive();
        let to = to.min(self.visible_len.get());
        let overflow = self.sidecar();
        self.compact
            .try_fold_range_at(from, to, init, |acc, compact| {
                f(acc, decode(compact, &overflow))
            })
    }
}
