use std::sync::Arc;

use bitview_traversable::{Traversable, TreeNode, make_leaf};
use brk_types::{Height, Version};
use vecdb::{
    AnyExportableVec, AnyVec, CachedBoxedVec, CachedReadableVec, CachedVec, PrintableIndex,
    ReadOnlyClone, ReadableBoxedVec, ReadableVec, TypedVec, VecIndex, short_type_name,
};

/// Pinned first-height lookup derived from one monotonic height-to-period
/// mapping.
#[derive(Clone)]
pub struct CachedFirstHeightVec<I: VecIndex>(CachedVec<FirstHeightSource<I>>);

impl<I: VecIndex> CachedFirstHeightVec<I> {
    pub fn new(mapping: ReadableBoxedVec<Height, I>) -> Self {
        Self(CachedVec::wrap(FirstHeightSource::new(mapping)))
    }

    pub fn invalidate(&self) {
        self.0.invalidate();
    }

    pub fn version(&self) -> Version {
        self.0.version()
    }

    pub fn snapshot(&self) -> Arc<Vec<Height>> {
        self.0.snapshot()
    }

    pub fn read_only_boxed_clone(&self) -> ReadableBoxedVec<I, Height> {
        ReadableBoxedVec::new(self.clone())
    }

    pub fn read_only_cached_boxed_clone(&self) -> CachedBoxedVec<I, Height> {
        self.0.cached_boxed_clone()
    }
}

impl<I: VecIndex> AnyVec for CachedFirstHeightVec<I> {
    fn version(&self) -> Version {
        self.0.version()
    }

    fn name(&self) -> &str {
        self.0.name()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn index_type_to_string(&self) -> &'static str {
        self.0.index_type_to_string()
    }

    fn region_names(&self) -> Vec<String> {
        self.0.region_names()
    }

    fn value_type_to_size_of(&self) -> usize {
        self.0.value_type_to_size_of()
    }

    fn value_type_to_string(&self) -> &'static str {
        self.0.value_type_to_string()
    }
}

impl<I: VecIndex> TypedVec for CachedFirstHeightVec<I> {
    type I = I;
    type T = Height;
}

impl<I: VecIndex> ReadableVec<I, Height> for CachedFirstHeightVec<I> {
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<Height>) {
        self.0.read_into_at(from, to, buf);
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(Height)) {
        self.0.for_each_range_dyn_at(from, to, each);
    }

    fn fold_range_at<B, F: FnMut(B, Height) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> B {
        self.0.fold_range_at(from, to, init, fold)
    }

    fn try_fold_range_at<B, E, F: FnMut(B, Height) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> Result<B, E> {
        self.0.try_fold_range_at(from, to, init, fold)
    }

    fn collect_one_at(&self, index: usize) -> Option<Height> {
        self.0.collect_one_at(index)
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<Height>) {
        self.0.read_sorted_into_at(indices, out);
    }
}

impl<I: VecIndex> ReadOnlyClone for CachedFirstHeightVec<I> {
    type ReadOnly = Self;

    fn read_only_clone(&self) -> Self {
        self.clone()
    }
}

impl<I: VecIndex> Traversable for CachedFirstHeightVec<I> {
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, Height, _>(self)
    }
}

#[derive(Clone)]
struct FirstHeightSource<I: VecIndex> {
    mapping: ReadableBoxedVec<Height, I>,
}

impl<I: VecIndex> FirstHeightSource<I> {
    pub fn new(mapping: ReadableBoxedVec<Height, I>) -> Self {
        Self { mapping }
    }

    fn try_for_each_value<E>(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(Height) -> Result<(), E>,
    ) -> Result<(), E> {
        let to = to.min(self.len());
        if from >= to {
            return Ok(());
        }

        let chunk_size = self.mapping.cursor_chunk_size();
        let source_len = self.mapping.len();
        let mut source_from = 0;
        let mut target = from;
        let mut buf = Vec::with_capacity(chunk_size);
        let mut previous_period = None;

        while source_from < source_len && target < to {
            let source_to = (source_from + chunk_size).min(source_len);
            buf.clear();
            self.mapping.read_into_at(source_from, source_to, &mut buf);

            for (offset, period) in buf.iter().copied().enumerate() {
                let period = period.to_usize();
                debug_assert!(previous_period.is_none_or(|previous| previous <= period));
                previous_period = Some(period);
                while target <= period && target < to {
                    each(Height::from(source_from + offset))?;
                    target += 1;
                }
            }

            source_from = source_to;
        }

        Ok(())
    }

    fn for_each_value(&self, from: usize, to: usize, mut each: impl FnMut(Height)) {
        let result = self.try_for_each_value(from, to, |value| {
            each(value);
            Ok::<_, std::convert::Infallible>(())
        });
        match result {
            Ok(()) => {}
            Err(error) => match error {},
        }
    }
}

impl<I: VecIndex> AnyVec for FirstHeightSource<I> {
    fn version(&self) -> Version {
        self.mapping.version()
    }

    fn name(&self) -> &str {
        "first_height"
    }

    fn len(&self) -> usize {
        self.mapping
            .collect_last()
            .map_or(0, |period| period.to_usize() + 1)
    }

    fn index_type_to_string(&self) -> &'static str {
        <I as PrintableIndex>::to_string()
    }

    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<Height>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<Height>()
    }
}

impl<I: VecIndex> TypedVec for FirstHeightSource<I> {
    type I = I;
    type T = Height;
}

impl<I: VecIndex> ReadableVec<I, Height> for FirstHeightSource<I> {
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<Height>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        self.for_each_value(from, to, |value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(Height)) {
        self.for_each_value(from, to, each);
    }

    fn fold_range_at<B, F: FnMut(B, Height) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> B {
        let mut acc = Some(init);
        self.for_each_value(from, to, |value| {
            acc = Some(fold(acc.take().unwrap(), value));
        });
        acc.unwrap()
    }

    fn try_fold_range_at<B, E, F: FnMut(B, Height) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> Result<B, E> {
        let mut acc = Some(init);
        self.try_for_each_value(from, to, |value| {
            acc = Some(fold(acc.take().unwrap(), value)?);
            Ok(())
        })?;
        Ok(acc.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use brk_types::Day1;
    use parking_lot::RwLock;

    use super::*;

    #[derive(Clone)]
    struct MappingVec(Arc<RwLock<Vec<Day1>>>);

    impl MappingVec {
        fn new(values: impl IntoIterator<Item = usize>) -> Self {
            Self(Arc::new(RwLock::new(
                values.into_iter().map(Day1::from).collect(),
            )))
        }

        fn replace(&self, index: usize, value: usize) {
            self.0.write()[index] = Day1::from(value);
        }

        fn push(&self, value: usize) {
            self.0.write().push(Day1::from(value));
        }
    }

    impl AnyVec for MappingVec {
        fn version(&self) -> Version {
            Version::ONE
        }

        fn name(&self) -> &str {
            "mapping"
        }

        fn len(&self) -> usize {
            self.0.read().len()
        }

        fn index_type_to_string(&self) -> &'static str {
            <Height as PrintableIndex>::to_string()
        }

        fn region_names(&self) -> Vec<String> {
            Vec::new()
        }

        fn value_type_to_size_of(&self) -> usize {
            size_of::<Day1>()
        }

        fn value_type_to_string(&self) -> &'static str {
            short_type_name::<Day1>()
        }
    }

    impl TypedVec for MappingVec {
        type I = Height;
        type T = Day1;
    }

    impl ReadableVec<Height, Day1> for MappingVec {
        fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<Day1>) {
            let values = self.0.read();
            let to = to.min(values.len());
            if from < to {
                buf.extend_from_slice(&values[from..to]);
            }
        }

        fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(Day1)) {
            let values = self.0.read();
            for &value in &values[from.min(values.len())..to.min(values.len())] {
                each(value);
            }
        }

        fn fold_range_at<B, F: FnMut(B, Day1) -> B>(
            &self,
            from: usize,
            to: usize,
            init: B,
            fold: F,
        ) -> B {
            let values = self.0.read();
            values[from.min(values.len())..to.min(values.len())]
                .iter()
                .copied()
                .fold(init, fold)
        }

        fn try_fold_range_at<B, E, F: FnMut(B, Day1) -> Result<B, E>>(
            &self,
            from: usize,
            to: usize,
            init: B,
            fold: F,
        ) -> Result<B, E> {
            let values = self.0.read();
            values[from.min(values.len())..to.min(values.len())]
                .iter()
                .copied()
                .try_fold(init, fold)
        }
    }

    #[test]
    fn gaps_match_the_previous_first_height_semantics() {
        let first_height =
            CachedFirstHeightVec::new(ReadableBoxedVec::new(MappingVec::new([0, 0, 2, 2, 5])));

        assert_eq!(
            first_height.collect(),
            [0_u32, 2, 2, 4, 4, 4].map(Height::from)
        );
        assert_eq!(
            first_height.collect_range(Day1::from(1), Day1::from(5)),
            [2_u32, 2, 4, 4].map(Height::from)
        );
        assert_eq!(
            first_height.read_sorted_at(&[0, 2, 5]),
            [0_u32, 2, 4].map(Height::from)
        );
    }

    #[test]
    fn periods_before_the_first_mapping_use_genesis_height() {
        let first_height =
            CachedFirstHeightVec::new(ReadableBoxedVec::new(MappingVec::new([2, 2, 3])));

        assert_eq!(first_height.collect(), [0_u32, 0, 0, 2].map(Height::from));
    }

    #[test]
    fn growing_to_a_new_period_refreshes_automatically() {
        let mapping = MappingVec::new([0, 0, 1, 1]);
        let first_height = CachedFirstHeightVec::new(ReadableBoxedVec::new(mapping.clone()));

        assert_eq!(first_height.collect(), [0_u32, 2].map(Height::from));

        mapping.push(2);

        assert_eq!(first_height.collect(), [0_u32, 2, 4].map(Height::from));
    }

    #[test]
    fn explicit_invalidation_refreshes_same_length_rewrites() {
        let mapping = MappingVec::new([0, 0, 1, 1]);
        let first_height = CachedFirstHeightVec::new(ReadableBoxedVec::new(mapping.clone()));

        assert_eq!(first_height.collect(), [0_u32, 2].map(Height::from));

        mapping.replace(1, 1);
        assert_eq!(first_height.collect(), [0_u32, 2].map(Height::from));

        first_height.invalidate();
        assert_eq!(first_height.collect(), [0_u32, 1].map(Height::from));
    }
}
