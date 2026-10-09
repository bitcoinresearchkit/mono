use std::{collections::BTreeMap, sync::Arc};

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

impl TreeNode {
    pub fn branch(children: IndexMap<String, TreeNode>) -> Self {
        Self::Branch(children.into())
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

    /// Removes every leaf of a series, and the branches left empty.
    pub fn remove_series(&mut self, name: &str) {
        if let Self::Branch(branch) = self {
            branch.children.retain(|_, child| match child {
                Self::Leaf(leaf) => leaf.name() != name,
                Self::Branch(_) => {
                    child.remove_series(name);
                    child.leaf_count() > 0
                }
            });
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

    /// Collect each value type's unit: the description of its schema.
    pub fn units(&self) -> BTreeMap<&str, Arc<str>> {
        let mut units = BTreeMap::new();
        self.collect_units(&mut units);
        units
    }

    fn collect_units<'a>(&'a self, units: &mut BTreeMap<&'a str, Arc<str>>) {
        match self {
            Self::Branch(children) => {
                for child in children.values() {
                    child.collect_units(units);
                }
            }
            Self::Leaf(leaf) => {
                // `Option<T>` keeps `T`'s schema under `$defs`.
                let schema = leaf
                    .schema
                    .get("$defs")
                    .and_then(|defs| defs.get(leaf.kind()))
                    .unwrap_or(&leaf.schema);
                let Some(description) = schema.get("description").and_then(|d| d.as_str()) else {
                    return;
                };
                // The first paragraph, unwrapped: later paragraphs are implementation notes.
                let first = description.split("\n\n").next().unwrap_or_default();
                let description = first.split_whitespace().collect::<Vec<_>>().join(" ");
                if let Some(existing) = units.get(leaf.kind()) {
                    debug_assert_eq!(
                        existing.as_ref(),
                        description.as_str(),
                        "two value types share the name {}",
                        leaf.kind()
                    );
                    return;
                }
                units.insert(leaf.kind(), Arc::from(description));
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
        Self::try_collapse_same_name_leaves(merged)
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

    fn leaf_and_group(key: &str, series: &str) -> ! {
        panic!(
            "Key '{key}' is both the series '{series}' and a group: give one of them its own key"
        )
    }

    /// Merges a node into the target map at the given key (consuming version).
    /// Panics on a conflict: two different series, one series with two descriptions, or a key
    /// that is both a series and a group.
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
            (Self::Leaf(leaf), Self::Branch(_)) => Self::leaf_and_group(&key, leaf.name()),
            (Self::Branch(_), Self::Leaf(leaf)) => Self::leaf_and_group(&key, leaf.name()),
            // Both branches: merge recursively
            (Self::Branch(existing_branch), Self::Branch(new_inner)) => {
                existing_branch.merge_fields(new_inner);
            }
        }
    }
}
