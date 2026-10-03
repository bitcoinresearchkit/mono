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
    /// Panics on a conflict (same key with incompatible values): a programming error.
    pub fn merge_branches(self) -> Self {
        let Self::Branch(tree) = self else {
            return self;
        };

        let mut merged = TreeBranch::default();

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
                    merged.merge_field(key, Self::Leaf(leaf));
                }
                Self::Branch(inner) => {
                    // Lift children from branches with their keys
                    merged.merge_fields(inner);
                }
            }
        }

        // If all children are leaves with the same series name, collapse into single leaf
        let node = Self::try_collapse_same_name_leaves(merged);
        if field_suffixes {
            node.with_field_suffixes()
        } else {
            node
        }
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
    /// Panics on a conflict: two different series, or one series with two descriptions.
    pub(crate) fn merge_node(target: &mut IndexMap<String, TreeNode>, key: String, node: TreeNode) {
        let Some(existing) = target.get_mut(&key) else {
            target.insert(key, node);
            return;
        };
        match (existing, node) {
            (Self::Leaf(a), Self::Leaf(b)) if a.is_same_series(&b) => {
                if a.merge(&b).is_none() {
                    panic!(
                        "Conflicting descriptions for series '{}' at key '{key}'",
                        a.name()
                    );
                }
            }
            (Self::Leaf(a), Self::Leaf(b)) => {
                panic!("Conflicting leaves for key '{key}':\n  existing: {a:?}\n  new: {b:?}")
            }
            (existing @ Self::Leaf(_), Self::Branch(branch)) => {
                let Self::Leaf(leaf) = mem::replace(existing, Self::branch(IndexMap::new())) else {
                    unreachable!()
                };
                let Self::Branch(new_branch) = existing else {
                    unreachable!()
                };
                new_branch.insert(BASE.to_string(), Self::Leaf(leaf));
                new_branch.merge_fields(branch);
            }
            (Self::Branch(existing_branch), Self::Leaf(leaf)) => {
                existing_branch.field_suffixes = false;
                Self::merge_node(existing_branch, BASE.to_string(), Self::Leaf(leaf));
            }
            // Both branches: merge recursively
            (Self::Branch(existing_branch), Self::Branch(new_inner)) => {
                existing_branch.field_suffixes &= new_inner.field_suffixes;
                existing_branch.merge_fields(new_inner);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use brk_types::Index;
    use serde_json::Value;

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

    #[test]
    fn lifted_metrics_preserve_raw_and_cumulative_indexes() {
        let mut children = vec![
            ("height", branch(vec![("raw", leaf("s", Index::Height))])),
            (
                "height_cumulative",
                branch(vec![("cumulative", leaf("s_cumulative", Index::Height))]),
            ),
        ];
        children.extend(
            [("day1", Index::Day1), ("week1", Index::Week1)].map(|(name, index)| {
                (
                    name,
                    branch(
                        ["average", "min", "max", "sum", "cumulative"]
                            .map(|key| (key, leaf(&format!("s_{key}"), index)))
                            .to_vec(),
                    ),
                )
            }),
        );
        let TreeNode::Branch(map) = branch(children).merge_branches() else {
            panic!("expected branch");
        };
        assert_eq!(map.len(), 6);
        for key in ["raw", "cumulative", "average", "min", "max", "sum"] {
            let TreeNode::Leaf(leaf) = &map[key] else {
                panic!("expected metric leaf");
            };
            let expected = match key {
                "raw" => BTreeSet::from([Index::Height]),
                "cumulative" => BTreeSet::from([Index::Height, Index::Day1, Index::Week1]),
                _ => BTreeSet::from([Index::Day1, Index::Week1]),
            };
            assert_eq!(leaf.indexes(), &expected, "{key}");
        }
    }
}
