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

mod compact;

pub use bitview_catalog::{SeriesLeaf, SeriesLeafWithSchema, TreeBranch, TreeNode};
pub use bitview_primitives::Index;
pub use compact::Compact;
pub use indexmap::IndexMap;
pub use vecdb::{AnyExportableVec, AnyStoredVec, ReadOnlyClone, Ro, Rw, StorageMode};

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

    /// Every writable stored vec of derived and leaf impls, hidden ones included; lazy views yield
    /// nothing. Skipped and write-only fields are left out; manual impls must forward it.
    fn iter_any_stored_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        iter::empty()
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
                    debug_assert_eq!(
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

    // Serializing (not `Schema::to_value`) applies schemars' canonical keyword order.
    let schema = SchemaGenerator::default().into_root_schema_for::<V::T>();
    let schema_json = to_value(schema).unwrap_or_default();
    let mut leaf = SeriesLeafWithSchema::new(leaf, schema_json);
    if V::T::MISSING {
        leaf.missing = indexes;
    }
    leaf.undefined = V::T::UNDEFINED;
    TreeNode::Leaf(leaf)
}

/// A vec published as one series leaf. `stored` vecs also yield themselves for writes.
macro_rules! leaf {
    (@ {$($stored:tt)*} [$($g:tt)*] $ty:ty where $($bounds:tt)+) => {
        impl<$($g)*> Traversable for $ty
        where
            $($bounds)+
        {
            fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
                iter::once(self as &dyn AnyExportableVec)
            }

            $($stored)*

            fn to_tree_node(&self) -> TreeNode {
                make_leaf(self)
            }
        }
    };
    (stored $($rest:tt)+) => {
        leaf!(@ {
            fn iter_any_stored_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
                iter::once(self as &mut dyn AnyStoredVec)
            }
        } $($rest)+);
    };
    ($($rest:tt)+) => {
        leaf!(@ {} $($rest)+);
    };
}

leaf!(stored [I, T, C] BytesVec<I, T, C> where
    I: VecIndex,
    T: BytesVecValue + Formattable + Serialize + JsonSchema,
    C: CachePolicy,
);

#[cfg(feature = "zerocopy")]
leaf!(stored [I, T, C] ZeroCopyVec<I, T, C> where
    I: VecIndex,
    T: ZeroCopyVecValue + Formattable + Serialize + JsonSchema,
    C: CachePolicy,
);

#[cfg(feature = "pco")]
leaf!(stored [I, T, C] PcoVec<I, T, C> where
    I: VecIndex,
    T: PcoVecValue + Formattable + Serialize + JsonSchema,
    C: CachePolicy,
);

#[cfg(feature = "lz4")]
leaf!(stored [I, T, C] LZ4Vec<I, T, C> where
    I: VecIndex,
    T: LZ4VecValue + Formattable + Serialize + JsonSchema,
    C: CachePolicy,
);

#[cfg(feature = "zstd")]
leaf!(stored [I, T, C] ZstdVec<I, T, C> where
    I: VecIndex,
    T: ZstdVecValue + Formattable + Serialize + JsonSchema,
    C: CachePolicy,
);

leaf!(stored [V] EagerVec<V> where
    V: StoredVec,
    V::T: Formattable + Serialize + JsonSchema,
);

leaf!(stored [V] MutableVec<V> where
    V: TypedVec,
    MutableVec<V>: ReadableVec<V::I, V::T> + AnyStoredVec,
    V::T: Formattable + Serialize + JsonSchema,
);

leaf!(stored [I, T] OverflowVec<I, T> where
    I: VecIndex,
    T: OverflowVecValue + Formattable + Serialize + JsonSchema,
);

leaf!([V] ReadOnlyMutableVec<V> where
    V: TypedVec + ReadableVec<V::I, V::T> + Clone,
    V::T: Formattable + Serialize + JsonSchema,
);

leaf!([I, T] ReadOnlyOverflowVec<I, T> where
    I: VecIndex,
    T: OverflowVecValue + Formattable + Serialize + JsonSchema,
);

// PcoVec::ReadOnly, LZ4Vec::ReadOnly, ZstdVec::ReadOnly
leaf!([I, T, S, C] ReadOnlyCompressedVec<I, T, S, C> where
    I: VecIndex,
    T: VecValue + Formattable + Serialize + JsonSchema,
    S: CompressionStrategy<T>,
    C: CachePolicy,
);

// BytesVec::ReadOnly, ZeroCopyVec::ReadOnly
leaf!([I, T, S, C] ReadOnlyRawVec<I, T, S, C> where
    I: VecIndex,
    T: VecValue + Formattable + Serialize + JsonSchema,
    S: RawStrategy<T>,
    C: CachePolicy,
);

leaf!([I, T, S] IndexVec<I, T, S> where
    I: VecIndex,
    T: VecValue + Formattable + Serialize + JsonSchema,
    S: TypedVec<I = I>,
);

leaf!([I, T, S1I, S1T] LazyVec<I, T, S1I, S1T> where
    I: VecIndex,
    T: VecValue + Formattable + Serialize + JsonSchema,
    S1I: VecIndex,
    S1T: VecValue,
);

leaf!([I, S, T, Op] LazyDeltaVec<I, S, T, Op> where
    I: VecIndex,
    S: VecValue,
    T: VecValue + Formattable + Serialize + JsonSchema,
    Op: DeltaOp<S, T>,
);

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

    fn iter_any_stored_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        (**self).iter_any_stored_mut()
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

    fn iter_any_stored_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.iter_mut()
            .flat_map(|inner| inner.iter_any_stored_mut())
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

    fn iter_any_stored_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.values_mut()
            .flat_map(|value| value.iter_any_stored_mut())
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
