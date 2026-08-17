use std::{collections::BTreeMap, fmt::Display};

pub use brk_types::{Index, SeriesLeaf, SeriesLeafWithSchema, TreeNode};
pub use indexmap::IndexMap;

#[cfg(feature = "derive")]
pub use brk_traversable_derive::Traversable;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AggFold, AnyExportableVec, AnyVec, BytesVec, BytesVecValue, CachedVec, ColumnId, ColumnarVec,
    CompressionStrategy, DeltaOp, EagerVec, Formattable, LazyAggVec, LazyColumnSumVec,
    LazyColumnVec, LazyColumnarVec, LazyDeltaVec, LazyVec, OverflowVec, OverflowVecValue,
    RawStrategy, ReadOnlyColumnarVec, ReadOnlyCompressedVec, ReadOnlyOverflowVec, ReadOnlyRawVec,
    ReadableColumnarVec, StoredVec, TypedVec, VecIndex, VecValue,
};

pub trait Traversable {
    fn to_tree_node(&self) -> TreeNode;
    /// All vecs including hidden — used for disk writes, flushes, exports.
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec>;
    /// Only non-hidden vecs — used for building the public series list.
    fn iter_any_visible(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        self.iter_any_exportable()
    }

    /// Collects public series description fragments from documented field paths.
    fn collect_series_descriptions<'a>(
        &'a self,
        description_fragments: &mut Vec<&'static str>,
        descriptions: &mut BTreeMap<&'a str, Vec<&'static str>>,
    ) {
        if description_fragments.is_empty() {
            return;
        }

        for vec in self.iter_any_visible() {
            match descriptions.entry(vec.name()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(description_fragments.clone());
                }
                std::collections::btree_map::Entry::Occupied(entry) => {
                    assert_eq!(
                        entry.get(),
                        description_fragments,
                        "Conflicting descriptions for series {}",
                        entry.key()
                    );
                }
            }
        }
    }
}

/// Creates a series leaf, including its value schema, from a vector.
pub fn make_leaf<I: VecIndex, T: JsonSchema, V: AnyVec>(vec: &V) -> TreeNode {
    let index_str = I::to_string();
    let index = Index::try_from(index_str).ok();
    let indexes = index.into_iter().collect();

    let leaf = SeriesLeaf::new(
        vec.name().to_string(),
        vec.value_type_to_string().to_string(),
        indexes,
    );

    let schema = schemars::SchemaGenerator::default().into_root_schema_for::<T>();
    let schema_json = serde_json::to_value(schema).unwrap_or_default();

    TreeNode::Leaf(SeriesLeafWithSchema::new(leaf, schema_json))
}

// BytesVec implementation
impl<I, T> Traversable for BytesVec<I, T>
where
    I: VecIndex,
    T: BytesVecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}

// ZeroCopyVec implementation (only if zerocopy feature enabled)
#[cfg(feature = "zerocopy")]
impl<I, T> Traversable for vecdb::ZeroCopyVec<I, T>
where
    I: VecIndex,
    T: vecdb::ZeroCopyVecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}

// PcoVec implementation (only if pco feature enabled)
#[cfg(feature = "pco")]
impl<I, T> Traversable for vecdb::PcoVec<I, T>
where
    I: VecIndex,
    T: vecdb::PcoVecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}

// LZ4Vec implementation (only if lz4 feature enabled)
#[cfg(feature = "lz4")]
impl<I, T> Traversable for vecdb::LZ4Vec<I, T>
where
    I: VecIndex,
    T: vecdb::LZ4VecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}

// ZstdVec implementation (only if zstd feature enabled)
#[cfg(feature = "zstd")]
impl<I, T> Traversable for vecdb::ZstdVec<I, T>
where
    I: VecIndex,
    T: vecdb::ZstdVecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}

// EagerVec implementation (wraps any stored vector)
impl<V> Traversable for EagerVec<V>
where
    V: StoredVec,
    V::T: Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<V::I, V::T, _>(self)
    }
}

impl<V, C> Traversable for ColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
    C::Row<V::T>: Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<V::I, C::Row<V::T>, _>(self)
    }
}

impl<V, C> Traversable for ReadOnlyColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
    C::Row<V::T>: Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<V::I, C::Row<V::T>, _>(self)
    }
}

impl<I, T> Traversable for OverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}

impl<I, T> Traversable for ReadOnlyOverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}

impl<S, T, C> Traversable for LazyColumnarVec<S, T, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
    T: VecValue,
    C::Row<T>: Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<S::I, C::Row<T>, _>(self)
    }
}

// Read-only compressed vec (PcoVec::ReadOnly, LZ4Vec::ReadOnly, ZstdVec::ReadOnly)
impl<I, T, S> Traversable for ReadOnlyCompressedVec<I, T, S>
where
    I: VecIndex,
    T: VecValue + Formattable + Serialize + JsonSchema,
    S: CompressionStrategy<T>,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}

// Read-only raw vec (BytesVec::ReadOnly, ZeroCopyVec::ReadOnly)
impl<I, T, S> Traversable for ReadOnlyRawVec<I, T, S>
where
    I: VecIndex,
    T: VecValue + Formattable + Serialize + JsonSchema,
    S: RawStrategy<T>,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}

impl<I, T, S1I, S1T> Traversable for LazyVec<I, T, S1I, S1T>
where
    I: VecIndex,
    T: VecValue + Formattable + Serialize + JsonSchema,
    S1I: VecIndex,
    S1T: VecValue,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}

impl<I, O, S1I, S2T, S1T, Strat> Traversable for LazyAggVec<I, O, S1I, S2T, S1T, Strat>
where
    I: VecIndex,
    O: VecValue + Formattable + Serialize + JsonSchema,
    S1I: VecIndex,
    S2T: VecValue,
    S1T: VecValue,
    Strat: AggFold<O, S1I, S2T, S1T>,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, O, _>(self)
    }
}

impl<I, S, T, Op> Traversable for LazyDeltaVec<I, S, T, Op>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue + Formattable + Serialize + JsonSchema,
    Op: DeltaOp<S, T>,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}

impl<S, C> Traversable for LazyColumnVec<S, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
    S::T: Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<S::I, S::T, _>(self)
    }
}

impl<S, C> Traversable for LazyColumnSumVec<S, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
    S::T: Formattable + Serialize + JsonSchema + std::ops::AddAssign,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<S::I, S::T, _>(self)
    }
}

impl<V: TypedVec + Traversable> Traversable for CachedVec<V> {
    fn to_tree_node(&self) -> TreeNode {
        self.inner.to_tree_node()
    }

    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        self.inner.iter_any_exportable()
    }

    fn iter_any_visible(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        self.inner.iter_any_visible()
    }

    fn collect_series_descriptions<'a>(
        &'a self,
        description_fragments: &mut Vec<&'static str>,
        descriptions: &mut BTreeMap<&'a str, Vec<&'static str>>,
    ) {
        self.inner
            .collect_series_descriptions(description_fragments, descriptions);
    }
}

impl<T: Traversable + ?Sized> Traversable for Box<T> {
    fn to_tree_node(&self) -> TreeNode {
        (**self).to_tree_node()
    }

    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        (**self).iter_any_exportable()
    }

    fn iter_any_visible(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        (**self).iter_any_visible()
    }

    fn collect_series_descriptions<'a>(
        &'a self,
        description_fragments: &mut Vec<&'static str>,
        descriptions: &mut BTreeMap<&'a str, Vec<&'static str>>,
    ) {
        (**self).collect_series_descriptions(description_fragments, descriptions);
    }
}

impl<T: Traversable> Traversable for Option<T> {
    fn to_tree_node(&self) -> TreeNode {
        match self {
            Some(inner) => inner.to_tree_node(),
            None => TreeNode::Branch(IndexMap::new()),
        }
    }

    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        match self {
            Some(inner) => Box::new(inner.iter_any_exportable())
                as Box<dyn Iterator<Item = &dyn AnyExportableVec>>,
            None => Box::new(std::iter::empty()),
        }
    }

    fn iter_any_visible(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        match self {
            Some(inner) => Box::new(inner.iter_any_visible())
                as Box<dyn Iterator<Item = &dyn AnyExportableVec>>,
            None => Box::new(std::iter::empty()),
        }
    }

    fn collect_series_descriptions<'a>(
        &'a self,
        description_fragments: &mut Vec<&'static str>,
        descriptions: &mut BTreeMap<&'a str, Vec<&'static str>>,
    ) {
        if let Some(inner) = self {
            inner.collect_series_descriptions(description_fragments, descriptions);
        }
    }
}

impl<K: Display, V: Traversable> Traversable for BTreeMap<K, V> {
    fn to_tree_node(&self) -> TreeNode {
        let children = self
            .iter()
            .map(|(k, v)| (format!("{}", k), v.to_tree_node()))
            .collect();
        TreeNode::Branch(children)
    }

    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        let mut iter: Box<dyn Iterator<Item = &dyn AnyExportableVec>> =
            Box::new(std::iter::empty());
        for v in self.values() {
            iter = Box::new(iter.chain(v.iter_any_exportable()));
        }
        iter
    }

    fn iter_any_visible(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        let mut iter: Box<dyn Iterator<Item = &dyn AnyExportableVec>> =
            Box::new(std::iter::empty());
        for v in self.values() {
            iter = Box::new(iter.chain(v.iter_any_visible()));
        }
        iter
    }

    fn collect_series_descriptions<'a>(
        &'a self,
        description_fragments: &mut Vec<&'static str>,
        descriptions: &mut BTreeMap<&'a str, Vec<&'static str>>,
    ) {
        for value in self.values() {
            value.collect_series_descriptions(description_fragments, descriptions);
        }
    }
}

/// Unit type implementation - used as ZST placeholder for disabled features
/// (e.g., Unpriced variants where dollar fields are not needed)
impl Traversable for () {
    fn to_tree_node(&self) -> TreeNode {
        TreeNode::Branch(IndexMap::new())
    }

    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::empty()
    }
}
