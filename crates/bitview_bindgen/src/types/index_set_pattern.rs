use std::collections::BTreeSet;

use bitview_catalog::TreeNode;
use bitview_primitives::Index;

/// A pattern of indexes that appear together on multiple series.
#[derive(Debug, Clone)]
pub struct IndexSetPattern {
    /// Pattern name (e.g., "DateHeightIndexes")
    pub(crate) name: String,
    /// The set of indexes
    pub(crate) indexes: BTreeSet<Index>,
}

/// The position of the accessor for a leaf's index set.
pub(crate) fn accessor_of(accessors: &[IndexSetPattern], indexes: &BTreeSet<Index>) -> usize {
    accessors
        .iter()
        .position(|accessor| accessor.indexes == *indexes)
        .expect("every leaf index set has an accessor")
}

/// Detect index patterns (sets of indexes that appear together on series).
pub(crate) fn detect_index_patterns(tree: &TreeNode) -> Vec<IndexSetPattern> {
    let mut unique_index_sets = BTreeSet::new();
    collect_index_sets_from_tree(tree, &mut unique_index_sets);

    // Sort by count (descending) then by first index name for deterministic ordering
    let mut sorted_sets: Vec<_> = unique_index_sets
        .into_iter()
        .filter(|indexes| !indexes.is_empty())
        .collect();
    sorted_sets.sort_by(|a, b| {
        b.len()
            .cmp(&a.len())
            .then_with(|| a.iter().next().cmp(&b.iter().next()))
    });

    // Assign unique sequential names
    sorted_sets
        .into_iter()
        .enumerate()
        .map(|(i, indexes)| IndexSetPattern {
            name: format!("SeriesPattern{}", i + 1),
            indexes: indexes.clone(),
        })
        .collect()
}

fn collect_index_sets_from_tree<'a>(
    node: &'a TreeNode,
    unique_index_sets: &mut BTreeSet<&'a BTreeSet<Index>>,
) {
    match node {
        TreeNode::Leaf(leaf) => {
            unique_index_sets.insert(leaf.indexes());
        }
        TreeNode::Branch(children) => {
            for child in children.values() {
                collect_index_sets_from_tree(child, unique_index_sets);
            }
        }
    }
}
