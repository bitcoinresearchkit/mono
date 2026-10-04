use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    fmt::Display,
    iter,
};

use schemars::{JsonSchema, SchemaGenerator};
use serde::Serialize;
use serde_json::to_value;
use vecdb::{
    BytesVec, BytesVecValue, CachePolicy, CompressionStrategy, DeltaOp, EagerVec, Formattable,
    IndexVec, LazyDeltaVec, LazyVec, MutableVec, OverflowVec, OverflowVecValue, PrintableIndex,
    RawStrategy, ReadOnlyCompressedVec, ReadOnlyMutableVec, ReadOnlyOverflowVec, ReadOnlyRawVec,
    ReadableVec, StoredVec, TypedVec, VecIndex, VecValue,
};

#[cfg(feature = "lz4")]
use vecdb::LZ4Vec;
#[cfg(feature = "lz4")]
use vecdb::LZ4VecValue;
#[cfg(feature = "pco")]
use vecdb::PcoVec;
#[cfg(feature = "pco")]
use vecdb::PcoVecValue;
#[cfg(feature = "zerocopy")]
use vecdb::ZeroCopyVec;
#[cfg(feature = "zerocopy")]
use vecdb::ZeroCopyVecValue;
#[cfg(feature = "zstd")]
use vecdb::ZstdVec;
#[cfg(feature = "zstd")]
use vecdb::ZstdVecValue;

pub use bitview_catalog::{SeriesLeaf, SeriesLeafWithSchema, TreeBranch, TreeNode};
pub use bitview_primitives::Index;
pub use indexmap::IndexMap;
pub use vecdb::{AnyExportableVec, ReadOnlyClone, Ro, Rw, StorageMode};

#[cfg(feature = "derive")]
pub use bitview_traversable_derive::Traversable;

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
                Entry::Vacant(entry) => {
                    entry.insert(description_fragments.clone());
                }
                Entry::Occupied(entry) => {
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

/// Creates a series leaf, including its value schema and nullability, from a vector.
pub fn make_leaf<V>(vec: &V) -> TreeNode
where
    V: TypedVec,
    V::T: JsonSchema + Formattable,
{
    let index_str = <V::I as PrintableIndex>::to_string();
    let index = Index::try_from(index_str).ok();
    let indexes: BTreeSet<Index> = index.into_iter().collect();

    let mut leaf = SeriesLeaf::new(
        vec.name().to_string(),
        vec.value_type_to_string().to_string(),
        indexes.clone(),
    );
    if vecdb::nullable::<V::T>() {
        leaf.nullable = indexes.clone();
    }

    let schema = SchemaGenerator::default().into_root_schema_for::<V::T>();
    let schema_json = to_value(schema).unwrap_or_default();

    let mut leaf = SeriesLeafWithSchema::new(leaf, schema_json);
    if V::T::MISSING {
        leaf.missing = indexes;
    }
    leaf.undefined = V::T::UNDEFINED;
    TreeNode::Leaf(leaf)
}

// BytesVec implementation
impl<I, T, C: CachePolicy> Traversable for BytesVec<I, T, C>
where
    I: VecIndex,
    T: BytesVecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}

// ZeroCopyVec implementation (only if zerocopy feature enabled)
#[cfg(feature = "zerocopy")]
impl<I, T, C: CachePolicy> Traversable for ZeroCopyVec<I, T, C>
where
    I: VecIndex,
    T: ZeroCopyVecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}

// PcoVec implementation (only if pco feature enabled)
#[cfg(feature = "pco")]
impl<I, T, C: CachePolicy> Traversable for PcoVec<I, T, C>
where
    I: VecIndex,
    T: PcoVecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}

// LZ4Vec implementation (only if lz4 feature enabled)
#[cfg(feature = "lz4")]
impl<I, T, C: CachePolicy> Traversable for LZ4Vec<I, T, C>
where
    I: VecIndex,
    T: LZ4VecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}

// ZstdVec implementation (only if zstd feature enabled)
#[cfg(feature = "zstd")]
impl<I, T, C: CachePolicy> Traversable for ZstdVec<I, T, C>
where
    I: VecIndex,
    T: ZstdVecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}

// EagerVec implementation (wraps any stored vector)
impl<V> Traversable for EagerVec<V>
where
    V: StoredVec,
    V::T: Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}

impl<V> Traversable for MutableVec<V>
where
    V: TypedVec,
    MutableVec<V>: ReadableVec<V::I, V::T>,
    V::T: Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}

impl<V> Traversable for ReadOnlyMutableVec<V>
where
    V: TypedVec + ReadableVec<V::I, V::T> + Clone,
    V::T: Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}

impl<I, T> Traversable for OverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}

impl<I, T> Traversable for ReadOnlyOverflowVec<I, T>
where
    I: VecIndex,
    T: OverflowVecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}

// Read-only compressed vec (PcoVec::ReadOnly, LZ4Vec::ReadOnly, ZstdVec::ReadOnly)
impl<I, T, S, C: CachePolicy> Traversable for ReadOnlyCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue + Formattable + Serialize + JsonSchema,
    S: CompressionStrategy<T>,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}

// Read-only raw vec (BytesVec::ReadOnly, ZeroCopyVec::ReadOnly)
impl<I, T, S, C: CachePolicy> Traversable for ReadOnlyRawVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue + Formattable + Serialize + JsonSchema,
    S: RawStrategy<T>,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}

impl<I, T, S> Traversable for IndexVec<I, T, S>
where
    I: VecIndex,
    T: VecValue + Formattable + Serialize + JsonSchema,
    S: TypedVec<I = I>,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
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
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
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
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
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
            None => TreeNode::branch(IndexMap::new()),
        }
    }

    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        self.iter().flat_map(|inner| inner.iter_any_exportable())
    }

    fn iter_any_visible(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        self.iter().flat_map(|inner| inner.iter_any_visible())
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
        let mut branch = TreeBranch::default();
        for (key, value) in self {
            branch.merge_field(key.to_string(), value.to_tree_node());
        }
        TreeNode::Branch(branch)
    }

    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        self.values().flat_map(|value| value.iter_any_exportable())
    }

    fn iter_any_visible(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        self.values().flat_map(|value| value.iter_any_visible())
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
        TreeNode::branch(IndexMap::new())
    }

    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::empty()
    }
}
