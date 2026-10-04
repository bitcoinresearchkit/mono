use std::collections::BTreeSet;

use bitview_catalog::{SeriesLeafWithSchema, TreeNode};
use bitview_primitives::Index;

/// How clients reach a leaf: its indexes, and those whose values can be missing (an empty
/// period), which clients type as nullable. A property of the series, never of its value type:
/// undefined values (NaN, sentinels) are typed with the value type, so shapes stay shared.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct Access {
    indexes: BTreeSet<Index>,
    missing: BTreeSet<Index>,
}

/// The leaf value types, split by whether they have an undefined value of their own (NaN, or a
/// sentinel) written as `null`: clients type those nullable wherever they appear.
pub(crate) fn value_types(tree: &TreeNode) -> ValueTypes {
    let mut types = ValueTypes::default();
    collect_value_types(tree, &mut types);
    types
}

#[derive(Default)]
pub(crate) struct ValueTypes {
    pub(crate) undefined: BTreeSet<String>,
    pub(crate) defined: BTreeSet<String>,
}

fn collect_value_types(node: &TreeNode, types: &mut ValueTypes) {
    match node {
        TreeNode::Leaf(leaf) if leaf.undefined => {
            types.undefined.insert(leaf.kind().to_owned());
        }
        TreeNode::Leaf(leaf) => {
            types.defined.insert(leaf.kind().to_owned());
        }
        TreeNode::Branch(children) => {
            for child in children.values() {
                collect_value_types(child, types);
            }
        }
    }
}

impl Access {
    pub(crate) fn of(leaf: &SeriesLeafWithSchema) -> Self {
        Self {
            indexes: leaf.indexes().clone(),
            missing: leaf.missing.clone(),
        }
    }
}

/// A leaf accessor type shared by every series with the same [`Access`].
#[derive(Debug, Clone)]
pub struct IndexSetPattern {
    /// Accessor name (e.g., "SeriesPattern1")
    pub(crate) name: String,
    /// The set of indexes
    pub(crate) indexes: BTreeSet<Index>,
    /// The indexes whose values can be missing
    pub(crate) missing: BTreeSet<Index>,
}

/// The position of the accessor for a leaf's access.
pub(crate) fn accessor_of(accessors: &[IndexSetPattern], access: &Access) -> usize {
    accessors
        .iter()
        .position(|accessor| {
            accessor.indexes == access.indexes && accessor.missing == access.missing
        })
        .expect("every leaf access has an accessor")
}

/// One accessor per distinct leaf access: most indexes first, then by first index, then the
/// fewest missing indexes.
pub(crate) fn detect_index_patterns(tree: &TreeNode) -> Vec<IndexSetPattern> {
    let mut accesses = BTreeSet::new();
    collect_accesses(tree, &mut accesses);

    let mut sorted: Vec<Access> = accesses
        .into_iter()
        .filter(|access| !access.indexes.is_empty())
        .collect();
    sorted.sort_by(|a, b| {
        b.indexes
            .len()
            .cmp(&a.indexes.len())
            .then_with(|| a.indexes.iter().next().cmp(&b.indexes.iter().next()))
            .then_with(|| a.missing.len().cmp(&b.missing.len()))
            .then_with(|| a.cmp(b))
    });

    sorted
        .into_iter()
        .enumerate()
        .map(|(i, access)| IndexSetPattern {
            name: format!("SeriesPattern{}", i + 1),
            indexes: access.indexes,
            missing: access.missing,
        })
        .collect()
}

fn collect_accesses(node: &TreeNode, accesses: &mut BTreeSet<Access>) {
    match node {
        TreeNode::Leaf(leaf) => {
            accesses.insert(Access::of(leaf));
        }
        TreeNode::Branch(children) => {
            for child in children.values() {
                collect_accesses(child, accesses);
            }
        }
    }
}
