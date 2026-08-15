mod last;
mod repeat;
mod strategy;

#[cfg(test)]
mod tests;

use std::{marker::PhantomData, sync::Arc};

use brk_traversable::{Index, SeriesLeaf, SeriesLeafWithSchema, Traversable, TreeNode};
use brk_types::{Day1, Version};
use vecdb::{
    AnyExportableVec, AnyVec, ReadableBoxedVec, ReadableVec, TypedVec, VecIndex, VecValue,
    short_type_name,
};

use super::DailyValue;

pub use last::LastDay;
pub use repeat::RepeatDay;
pub use strategy::DayStrategy;

pub struct DailyView<I, T, S>
where
    I: VecIndex,
    T: VecValue,
{
    name: Arc<str>,
    version: Version,
    source: ReadableBoxedVec<Day1, T>,
    mapping: ReadableBoxedVec<I, Day1>,
    _phantom: PhantomData<fn() -> S>,
}

impl<I, T, S> Clone for DailyView<I, T, S>
where
    I: VecIndex,
    T: VecValue,
{
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            version: self.version,
            source: self.source.clone(),
            mapping: self.mapping.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<I, T, S> DailyView<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: DayStrategy,
{
    pub fn new(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Day1, T>,
        mapping: ReadableBoxedVec<I, Day1>,
    ) -> Self {
        Self {
            name: Arc::from(name),
            version,
            source,
            mapping,
            _phantom: PhantomData,
        }
    }

    fn try_fold_values<B, E, F>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> std::result::Result<B, E>
    where
        F: FnMut(B, Option<T>) -> std::result::Result<B, E>,
    {
        let mapping_len = self.mapping.len();
        let to = to.min(mapping_len);
        if from >= to {
            return Ok(init);
        }

        let mapping = self
            .mapping
            .collect_range_dyn(from, S::mapping_end(to, mapping_len));
        let source_len = self.source.len();
        try_fold_mapped(
            &*self.source,
            0,
            to - from,
            |index| S::source_index(&mapping, index, source_len),
            init,
            f,
        )
    }

    fn fold_values<B, F>(&self, from: usize, to: usize, init: B, mut f: F) -> B
    where
        F: FnMut(B, Option<T>) -> B,
    {
        match self.try_fold_values(from, to, init, |acc, value| {
            Ok::<_, std::convert::Infallible>(f(acc, value))
        }) {
            Ok(result) => result,
            Err(error) => match error {},
        }
    }
}

impl<I, T, S> AnyVec for DailyView<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: DayStrategy,
{
    fn version(&self) -> Version {
        self.version + self.source.version() + self.mapping.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.mapping.len()
    }

    fn index_type_to_string(&self) -> &'static str {
        I::to_string()
    }

    fn region_names(&self) -> Vec<String> {
        vec![]
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<Option<T>>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<Option<T>>()
    }
}

impl<I, T, S> TypedVec for DailyView<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: DayStrategy,
{
    type I = I;
    type T = Option<T>;
}

impl<I, T, S> ReadableVec<I, Option<T>> for DailyView<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: DayStrategy,
{
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<Option<T>>) {
        self.fold_values(from, to, (), |(), value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(Option<T>)) {
        self.fold_values(from, to, (), |(), value| f(value));
    }

    fn fold_range_at<B, F: FnMut(B, Option<T>) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> B {
        self.fold_values(from, to, init, f)
    }

    fn try_fold_range_at<B, E, F: FnMut(B, Option<T>) -> std::result::Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> std::result::Result<B, E> {
        self.try_fold_values(from, to, init, f)
    }

    fn collect_one_at(&self, index: usize) -> Option<Option<T>> {
        let mapping_len = self.mapping.len();
        if index >= mapping_len {
            return None;
        }

        let mapping = self
            .mapping
            .collect_range_dyn(index, S::mapping_end(index.saturating_add(1), mapping_len));
        Some(
            S::source_index(&mapping, 0, self.source.len())
                .and_then(|day| self.source.collect_one_at(day)),
        )
    }
}

impl<I, T, S> Traversable for DailyView<I, T, S>
where
    I: VecIndex,
    T: DailyValue,
    S: DayStrategy,
{
    fn to_tree_node(&self) -> TreeNode {
        let indexes = Index::try_from(I::to_string()).ok().into_iter().collect();
        let leaf = SeriesLeaf::new(
            self.name().to_string(),
            self.value_type_to_string().to_string(),
            indexes,
        );
        let schema = schemars::SchemaGenerator::default().into_root_schema_for::<Option<T>>();
        let schema_json = serde_json::to_value(schema).unwrap_or_default();

        TreeNode::Leaf(SeriesLeafWithSchema::new(leaf, schema_json))
    }

    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }
}

fn try_fold_mapped<T, S, B, E, F, G>(
    source: &S,
    from: usize,
    to: usize,
    mut source_index: G,
    init: B,
    mut f: F,
) -> std::result::Result<B, E>
where
    T: VecValue,
    S: ReadableVec<Day1, T> + ?Sized,
    F: FnMut(B, Option<T>) -> std::result::Result<B, E>,
    G: FnMut(usize) -> Option<usize>,
{
    let mut indices = Vec::with_capacity(to - from);
    let mut slots: Vec<Option<u32>> = Vec::with_capacity(to - from);

    for output_index in from..to {
        let Some(source_index) = source_index(output_index) else {
            slots.push(None);
            continue;
        };

        let slot = match indices.last() {
            Some(&last) if last == source_index => indices.len() - 1,
            Some(&last) => {
                debug_assert!(last < source_index);
                indices.push(source_index);
                indices.len() - 1
            }
            None => {
                indices.push(source_index);
                0
            }
        };
        debug_assert!(u32::try_from(slot).is_ok());
        slots.push(Some(slot as u32));
    }

    let values = source.read_sorted_at(&indices);
    slots.into_iter().try_fold(init, |acc, slot| match slot {
        Some(slot) => f(acc, Some(values[slot as usize].clone())),
        None => f(acc, None),
    })
}
