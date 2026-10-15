use std::{collections::BTreeMap, sync::Arc};

use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{SeriesLeaf, SeriesLeafWithSchema, TreeBranch};

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

    /// Visit every leaf of this subtree, in tree order.
    pub fn for_each_leaf<'a>(&'a self, f: &mut impl FnMut(&'a SeriesLeafWithSchema)) {
        match self {
            Self::Branch(children) => {
                for child in children.values() {
                    child.for_each_leaf(f);
                }
            }
            Self::Leaf(leaf) => f(leaf),
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
                    debug_assert_eq!(
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
        self.for_each_leaf(&mut |leaf| {
            let Some(description) = &leaf.leaf.description else {
                return;
            };
            let existing = descriptions.insert(leaf.name(), description.clone());
            debug_assert!(
                existing.is_none_or(|existing| existing == *description),
                "Conflicting descriptions for series {}",
                leaf.name()
            );
        });
        descriptions
    }

    /// Collect each value type's unit: the description of its schema.
    pub fn units(&self) -> BTreeMap<&str, Arc<str>> {
        let mut units = BTreeMap::<_, Arc<str>>::new();
        self.for_each_leaf(&mut |leaf| match units.get(leaf.kind()) {
            Some(existing) => debug_assert!(
                Self::unit(leaf).is_none_or(|unit| existing.as_ref() == unit),
                "two value types share the name {}",
                leaf.kind()
            ),
            None => {
                if let Some(unit) = Self::unit(leaf) {
                    units.insert(leaf.kind(), Arc::from(unit));
                }
            }
        });
        units
    }

    /// A leaf's unit: the first paragraph of its value type's schema description, unwrapped
    /// (later paragraphs are implementation notes).
    fn unit(leaf: &SeriesLeafWithSchema) -> Option<String> {
        // `Option<T>` keeps `T`'s schema under `$defs`.
        let schema = leaf
            .schema
            .get("$defs")
            .and_then(|defs| defs.get(leaf.kind()))
            .unwrap_or(&leaf.schema);
        let description = schema.get("description")?.as_str()?;
        let first = description.split("\n\n").next().unwrap_or_default();
        Some(first.split_whitespace().collect::<Vec<_>>().join(" "))
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

    /// If all entries in the map are leaves of the same series with compatible descriptions,
    /// collapse them into a single leaf with merged indexes.
    fn try_collapse_same_name_leaves(map: TreeBranch) -> Self {
        if !Self::collapsible(&map) {
            return Self::Branch(map);
        }
        let mut leaves = map.children.into_values().map(|node| match node {
            Self::Leaf(leaf) => leaf,
            Self::Branch(_) => unreachable!("checked by collapsible"),
        });
        let mut merged = leaves.next().expect("checked by collapsible");
        for leaf in leaves {
            let merged_ok = merged.merge(&leaf).is_some();
            debug_assert!(merged_ok, "checked by collapsible");
        }
        Self::Leaf(merged)
    }

    /// Whether every entry is a leaf of one series, its descriptions agreeing.
    fn collapsible(map: &TreeBranch) -> bool {
        let mut first: Option<&SeriesLeafWithSchema> = None;
        let mut description = None;
        map.values().all(|node| {
            let Self::Leaf(leaf) = node else {
                return false;
            };
            if first.get_or_insert(leaf).name() != leaf.name() {
                return false;
            }
            let incoming = leaf.leaf.description.as_ref();
            let agree = SeriesLeaf::descriptions_agree(description, incoming);
            description = description.or(incoming);
            agree
        }) && first.is_some()
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
