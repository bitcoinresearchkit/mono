use std::{marker::PhantomData, sync::Arc};

use brk_types::{Height, StoredU64};
use vecdb::{
    AnyVec, BinaryTransform, Cursor, PrintableIndex, ReadableBoxedVec, ReadableVec, TypedVec,
    VecValue, Version, short_type_name,
};

use crate::internal::CachedBlockCountReader;

pub struct LazyRatioWithCachedBlockCount<T, F>
where
    T: VecValue,
{
    name: Arc<str>,
    base_version: Version,
    numerator: ReadableBoxedVec<Height, StoredU64>,
    denominator: CachedBlockCountReader,
    _output: PhantomData<(T, F)>,
}

impl<T, F> LazyRatioWithCachedBlockCount<T, F>
where
    T: VecValue,
{
    pub fn new(
        name: &str,
        version: Version,
        numerator: ReadableBoxedVec<Height, StoredU64>,
        denominator: CachedBlockCountReader,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            numerator,
            denominator,
            _output: PhantomData,
        }
    }
}

impl<T, F> LazyRatioWithCachedBlockCount<T, F>
where
    T: VecValue,
    F: BinaryTransform<StoredU64, StoredU64, T> + Send + Sync,
{
    fn for_each_value(&self, from: usize, to: usize, mut each: impl FnMut(T)) {
        let to = to.min(self.len());
        if from >= to {
            return;
        }

        let mut numerators = self.numerator.collect_range_dyn(from, to).into_iter();
        self.denominator
            .for_each_cumulative(from, to, |denominator| {
                each(F::apply(numerators.next().unwrap(), denominator));
            });
    }
}

impl<T, F> Clone for LazyRatioWithCachedBlockCount<T, F>
where
    T: VecValue,
{
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            base_version: self.base_version,
            numerator: self.numerator.clone(),
            denominator: self.denominator.clone(),
            _output: PhantomData,
        }
    }
}

impl<T, F> AnyVec for LazyRatioWithCachedBlockCount<T, F>
where
    T: VecValue,
    F: BinaryTransform<StoredU64, StoredU64, T> + Send + Sync,
{
    fn version(&self) -> Version {
        self.base_version + self.numerator.version() + self.denominator.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.numerator.len().min(self.denominator.len())
    }

    fn index_type_to_string(&self) -> &'static str {
        <Height as PrintableIndex>::to_string()
    }

    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<T>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<T>()
    }
}

impl<T, F> TypedVec for LazyRatioWithCachedBlockCount<T, F>
where
    T: VecValue,
    F: BinaryTransform<StoredU64, StoredU64, T> + Send + Sync,
{
    type I = Height;
    type T = T;
}

impl<T, F> ReadableVec<Height, T> for LazyRatioWithCachedBlockCount<T, F>
where
    T: VecValue,
    F: BinaryTransform<StoredU64, StoredU64, T> + Send + Sync,
{
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        buf.reserve(to.saturating_sub(from));
        self.for_each_value(from, to, |value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(T)) {
        self.for_each_value(from, to, each);
    }

    fn fold_range_at<B, G: FnMut(B, T) -> B>(&self, from: usize, to: usize, init: B, fold: G) -> B {
        let mut values = Vec::with_capacity(to.saturating_sub(from));
        self.read_into_at(from, to, &mut values);
        values.into_iter().fold(init, fold)
    }

    fn try_fold_range_at<B, E, G: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: G,
    ) -> Result<B, E> {
        let mut values = Vec::with_capacity(to.saturating_sub(from));
        self.read_into_at(from, to, &mut values);
        values.into_iter().try_fold(init, fold)
    }

    fn collect_one_at(&self, index: usize) -> Option<T> {
        Some(F::apply(
            self.numerator.collect_one_at(index)?,
            self.denominator.cumulative_at(index)?,
        ))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        match indices {
            [] => return,
            &[index] => {
                if let Some(value) = self.collect_one_at(index) {
                    out.push(value);
                }
                return;
            }
            _ => {}
        }

        let indices = &indices[..indices.partition_point(|&index| index < self.len())];
        let denominators = self.denominator.read_sorted_at(indices);
        let mut numerators = Cursor::new(&*self.numerator);

        out.reserve(indices.len());
        for (&index, denominator) in indices.iter().zip(denominators) {
            let Some(numerator) = numerators.get(index) else {
                continue;
            };
            out.push(F::apply(numerator, denominator));
        }
    }
}
