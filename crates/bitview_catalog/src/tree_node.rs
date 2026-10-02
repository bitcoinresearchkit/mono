use std::{collections::BTreeMap, mem, sync::Arc};

use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{SeriesLeafWithSchema, TreeBranch};

/// Hierarchical tree node for organizing series into categories
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum TreeNode {
    /// Branch node containing subcategories
    Branch(TreeBranch),
    /// Leaf node containing series metadata with schema
    Leaf(SeriesLeafWithSchema),
}

const BASE: &str = "raw";

impl TreeNode {
    pub fn branch(children: IndexMap<String, TreeNode>) -> Self {
        Self::Branch(children.into())
    }

    /// Preserve an inner declaration when a transparent projection already has one.
    pub fn with_source(mut self, source: &'static str) -> Self {
        if let Self::Branch(branch) = &mut self {
            branch.source.get_or_insert(source);
        }
        self
    }

    /// Declare naming before wrapping or combining this family with other nodes.
    pub fn with_field_suffixes(mut self) -> Self {
        if let Self::Branch(branch) = &mut self {
            branch.field_suffixes = true;
        }
        self
    }

    /// Count all series leaves in this subtree.
    pub fn leaf_count(&self) -> usize {
        match self {
            Self::Branch(children) => children.values().map(Self::leaf_count).sum(),
            Self::Leaf(_) => 1,
        }
    }

    /// Attach interned descriptions to every matching series leaf.
    pub fn set_descriptions(&mut self, descriptions: &BTreeMap<&str, Arc<str>>) {
        match self {
            Self::Branch(children) => {
                for child in children.values_mut() {
                    child.set_descriptions(descriptions);
                }
            }
            Self::Leaf(leaf) => {
                let Some(description) = descriptions.get(leaf.name()) else {
                    return;
                };
                if let Some(existing) = &leaf.leaf.description {
                    assert_eq!(
                        existing,
                        description,
                        "Conflicting descriptions for series {}",
                        leaf.name()
                    );
                } else {
                    leaf.leaf.description = Some(description.clone());
                }
            }
        }
    }

    /// Collect one shared description per documented series.
    pub fn descriptions(&self) -> BTreeMap<&str, Arc<str>> {
        let mut descriptions = BTreeMap::new();
        self.collect_descriptions(&mut descriptions);
        descriptions
    }

    fn collect_descriptions<'a>(&'a self, descriptions: &mut BTreeMap<&'a str, Arc<str>>) {
        match self {
            Self::Branch(children) => {
                for child in children.values() {
                    child.collect_descriptions(descriptions);
                }
            }
            Self::Leaf(leaf) => {
                let Some(description) = &leaf.leaf.description else {
                    return;
                };
                if let Some(existing) = descriptions.insert(leaf.name(), description.clone()) {
                    assert_eq!(
                        existing,
                        *description,
                        "Conflicting descriptions for series {}",
                        leaf.name()
                    );
                }
            }
        }
    }

    /// Wraps a node in a Branch with the given key.
    /// Used by #[traversable(wrap = "...")] to produce Branch { key: inner }.
    pub fn wrap(key: &str, inner: Self) -> Self {
        let mut map = IndexMap::new();
        map.insert(key.to_string(), inner);
        Self::branch(map)
    }

    /// Merges all first-level branches into a single flattened structure (consuming version).
    /// Direct leaves use their key (use #[traversable(rename = "...")] to control).
    /// Branch children are lifted with their keys.
    /// If all resulting children are leaves with the same series name, collapses to a single leaf.
    /// Returns None if conflicts are found (same key with incompatible values).
    pub fn merge_branches(self) -> Option<Self> {
        let Self::Branch(tree) = self else {
            return Some(self);
        };

        let mut merged = TreeBranch::default();
        if let Some(Self::Branch(first)) = tree.values().next()
            && first.source.is_some()
            && tree
                .values()
                .all(|node| matches!(node, Self::Branch(branch) if branch.source == first.source))
        {
            merged.source = first.source;
        }

        // Lifting only field-suffixed branches preserves their naming contract.
        // Undeclared branches or direct leaves do not supply that guarantee.
        let field_suffixes = !tree.is_empty()
            && tree
                .values()
                .all(|node| matches!(node, Self::Branch(branch) if branch.field_suffixes));

        for (key, node) in tree.children {
            match node {
                Self::Leaf(leaf) => {
                    // Direct leaves use their key (which may be renamed via attribute)
                    let declaration = tree.field_types.get(&key).copied();
                    merged.merge_field(key, Self::Leaf(leaf), declaration)?;
                }
                Self::Branch(inner) => {
                    // Lift children from branches with their keys
                    merged.merge_fields(inner)?;
                }
            }
        }

        // If all children are leaves with the same series name, collapse into single leaf
        let node = Self::try_collapse_same_name_leaves(merged);
        Some(if field_suffixes {
            node.with_field_suffixes()
        } else {
            node
        })
    }

    /// If all entries in the map are leaves with the same series name,
    /// collapse them into a single leaf with merged indexes.
    fn try_collapse_same_name_leaves(map: TreeBranch) -> Self {
        if map.is_empty() {
            return Self::Branch(map);
        }

        // Check if all entries are compatible leaves for the same series.
        let mut merged_leaf: Option<SeriesLeafWithSchema> = None;

        for node in map.values() {
            match node {
                Self::Leaf(leaf) => {
                    if let Some(merged) = &mut merged_leaf {
                        if leaf.name() != merged.name() || merged.merge(leaf).is_none() {
                            // Incompatible leaves cannot collapse.
                            return Self::Branch(map);
                        }
                    } else {
                        merged_leaf = Some(leaf.clone());
                    }
                }
                Self::Branch(_) => {
                    // Has non-leaf entries - can't collapse
                    return Self::Branch(map);
                }
            }
        }

        // All entries were leaves with the same name
        Self::Leaf(merged_leaf.unwrap())
    }

    /// Merges a node into the target map at the given key (consuming version).
    /// Returns None if there's a conflict.
    pub fn merge_node(
        target: &mut IndexMap<String, TreeNode>,
        key: String,
        node: TreeNode,
    ) -> Option<()> {
        match target.get_mut(&key) {
            None => {
                target.insert(key, node);
                Some(())
            }
            Some(existing) => {
                match (existing, node) {
                    (Self::Leaf(a), Self::Leaf(b)) if a.is_same_series(&b) => a.merge(&b),
                    (Self::Leaf(a), Self::Leaf(b)) => {
                        eprintln!("Conflict: Different leaf values for key '{key}'");
                        eprintln!("  Existing: {a:?}");
                        eprintln!("  New: {b:?}");
                        None
                    }
                    (existing @ Self::Leaf(_), Self::Branch(branch)) => {
                        let Self::Leaf(leaf) =
                            mem::replace(existing, Self::branch(IndexMap::new()))
                        else {
                            unreachable!()
                        };
                        let Self::Branch(new_branch) = existing else {
                            unreachable!()
                        };
                        new_branch.insert(BASE.to_string(), Self::Leaf(leaf));

                        new_branch.merge_fields(branch)?;
                        Some(())
                    }
                    (Self::Branch(existing_branch), Self::Leaf(leaf)) => {
                        existing_branch.field_suffixes = false;
                        existing_branch.source = None;
                        Self::merge_node(existing_branch, BASE.to_string(), Self::Leaf(leaf))?;
                        Some(())
                    }
                    // Both branches: merge recursively
                    (Self::Branch(existing_branch), Self::Branch(new_inner)) => {
                        existing_branch.field_suffixes &= new_inner.field_suffixes;
                        if existing_branch.source != new_inner.source {
                            existing_branch.source = None;
                        }
                        existing_branch.merge_fields(new_inner)?;
                        Some(())
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use brk_types::Index;
    use serde_json::{Value, to_string as SerdeJsonToString};

    use super::*;
    use crate::SeriesLeaf;

    fn leaf(name: &str, index: Index) -> TreeNode {
        TreeNode::Leaf(SeriesLeafWithSchema {
            leaf: SeriesLeaf {
                name: name.to_string(),
                kind: "TestType".to_string(),
                indexes: BTreeSet::from([index]),
                description: None,
            },
            openapi_type: "object".to_string(),
            schema: Value::Null,
        })
    }

    fn branch(children: Vec<(&str, TreeNode)>) -> TreeNode {
        TreeNode::Branch(
            children
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    fn get_leaf_indexes(node: &TreeNode) -> Option<&BTreeSet<Index>> {
        match node {
            TreeNode::Leaf(l) => Some(&l.leaf.indexes),
            _ => None,
        }
    }

    #[test]
    fn descriptions_are_shared_by_every_matching_leaf() {
        let mut tree = branch(vec![
            ("height", leaf("shared", Index::Height)),
            ("day", leaf("shared", Index::Day1)),
            ("other", leaf("other", Index::Height)),
        ]);
        let descriptions = BTreeMap::from([("shared", Arc::from("Shared metric description."))]);

        tree.set_descriptions(&descriptions);

        let collected = tree.descriptions();
        assert_eq!(collected.len(), 1);
        assert!(Arc::ptr_eq(&collected["shared"], &descriptions["shared"]));
        let weak = Arc::downgrade(&descriptions["shared"]);
        let json = SerdeJsonToString(&tree).unwrap();
        assert_eq!(json.matches("Shared metric description.").count(), 2);
        assert!(!json.contains("\"other\":{\"description\""));
        drop(collected);
        drop(descriptions);
        drop(tree);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn merge_conflict_from_lifted_branches() {
        // Two branches lifting children with same key but different series names → conflict
        let tree = branch(vec![
            ("a", branch(vec![("data", leaf("s_a", Index::Height))])),
            ("b", branch(vec![("data", leaf("s_b", Index::Day1))])),
        ]);
        let result = tree.merge_branches();
        assert!(result.is_none(), "Should detect conflict");
    }

    #[test]
    fn collapse_direct_leaf_with_lifted_branches_same_name() {
        // ComputedVecsDateLast pattern:
        // - day1: direct leaf (field name as key)
        // - rest (flattened): DerivedDateLast → branches with "last" children
        // All leaves have same series name → collapse to single Leaf
        let tree = branch(vec![
            // Direct leaf from day1 field (no wrap attribute)
            ("day1", leaf("1m_block_count", Index::Day1)),
            // Flattened from rest: DerivedDateLast
            (
                "week1",
                branch(vec![("last", leaf("1m_block_count", Index::Week1))]),
            ),
            (
                "month1",
                branch(vec![("last", leaf("1m_block_count", Index::Month1))]),
            ),
        ]);

        let merged = tree.merge_branches().unwrap();

        // All leaves have same name "1m_block_count" → collapses to single Leaf
        match &merged {
            TreeNode::Leaf(leaf) => {
                assert_eq!(leaf.name(), "1m_block_count");
                let indexes = leaf.indexes();
                assert!(indexes.contains(&Index::Day1));
                assert!(indexes.contains(&Index::Week1));
                assert!(indexes.contains(&Index::Month1));
            }
            TreeNode::Branch(map) => {
                panic!(
                    "Expected collapsed leaf, got branch: {:?}",
                    map.keys().collect::<Vec<_>>()
                );
            }
        }
    }

    #[test]
    fn case5_computed_block_full() {
        // ComputedBlockFull has:
        // - height: wrapped as "raw" (raw values, not aggregated)
        // - rest (flatten): DerivedComputedBlockFull {
        //     height_cumulative: CumulativeVec → Branch{"cumulative": Leaf}
        //     day1: Full → Branch{avg, min, max, sum, cumulative}
        //     dates (flatten): more aggregation branches
        //   }
        let tree = branch(vec![
            // height wrapped as "raw" (raw values at height granularity)
            ("height", branch(vec![("raw", leaf("s", Index::Height))])),
            // height_cumulative wrapped as cumulative
            (
                "height_cumulative",
                branch(vec![("cumulative", leaf("s_cumulative", Index::Height))]),
            ),
            // day1 Full
            (
                "day1",
                branch(vec![
                    ("average", leaf("s_average", Index::Day1)),
                    ("min", leaf("s_min", Index::Day1)),
                    ("max", leaf("s_max", Index::Day1)),
                    ("sum", leaf("s_sum", Index::Day1)),
                    ("cumulative", leaf("s_cumulative", Index::Day1)),
                ]),
            ),
            // week1 (from flattened dates)
            (
                "week1",
                branch(vec![
                    ("average", leaf("s_average", Index::Week1)),
                    ("min", leaf("s_min", Index::Week1)),
                    ("max", leaf("s_max", Index::Week1)),
                    ("sum", leaf("s_sum", Index::Week1)),
                    ("cumulative", leaf("s_cumulative", Index::Week1)),
                ]),
            ),
        ]);

        let merged = tree.merge_branches().unwrap();

        // DESIRED: { base, average, min, max, sum, cumulative } each with merged indexes
        match &merged {
            TreeNode::Branch(map) => {
                assert_eq!(
                    map.len(),
                    6,
                    "Expected 6 keys, got: {:?}",
                    map.keys().collect::<Vec<_>>()
                );

                // base should have Height only
                let base_indexes = get_leaf_indexes(map.get("raw").unwrap()).unwrap();
                assert!(base_indexes.contains(&Index::Height));
                assert_eq!(base_indexes.len(), 1);

                // cumulative should include Height (from height_cumulative)
                let cumulative_indexes = get_leaf_indexes(map.get("cumulative").unwrap()).unwrap();
                assert!(
                    cumulative_indexes.contains(&Index::Height),
                    "cumulative should include Height"
                );
                assert!(cumulative_indexes.contains(&Index::Day1));
                assert!(cumulative_indexes.contains(&Index::Week1));

                // average, min, max, sum should have Day1 and Week1 only
                for key in ["average", "min", "max", "sum"] {
                    let indexes = get_leaf_indexes(map.get(key).unwrap()).unwrap();
                    assert!(!indexes.contains(&Index::Height));
                    assert!(indexes.contains(&Index::Day1));
                    assert!(indexes.contains(&Index::Week1));
                }
            }
            _ => panic!("Expected branch"),
        }
    }
}
