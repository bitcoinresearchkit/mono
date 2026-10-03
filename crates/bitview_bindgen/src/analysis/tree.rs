//! Tree traversal helpers for pattern analysis.
//!
//! This module provides utilities for working with the TreeNode structure,
//! including leaf name extraction and index pattern detection.

use std::collections::{BTreeMap, BTreeSet};

use bitview_catalog::{TreeNode, extract_json_type};
use brk_types::Index;
use indexmap::IndexMap;

use crate::{IndexSetPattern, PatternField, child_type_name};

/// Get the shortest leaf name from a tree node.
///
/// This is useful for pattern base analysis where we want the "base" case
/// (e.g., the leaf without suffix like `_btc` or `_usd`).
pub fn get_shortest_leaf_name(node: &TreeNode) -> Option<String> {
    shortest_leaf_name(node).map(str::to_owned)
}

fn shortest_leaf_name(node: &TreeNode) -> Option<&str> {
    match node {
        TreeNode::Leaf(leaf) => Some(leaf.name()),
        TreeNode::Branch(children) => children
            .values()
            .filter_map(shortest_leaf_name)
            .min_by_key(|name| name.len()),
    }
}

/// Get the field signature for a branch node's children.
/// Fields are sorted alphabetically for consistent pattern matching.
pub fn get_node_fields(
    children: &IndexMap<String, TreeNode>,
    pattern_lookup: &BTreeMap<Vec<PatternField>, String>,
) -> Vec<PatternField> {
    let mut fields: Vec<PatternField> = children
        .iter()
        .map(|(name, node)| {
            let (rust_type, json_type, indexes) = match node {
                TreeNode::Leaf(leaf) => (
                    leaf.kind().to_string(),
                    extract_json_type(&leaf.schema),
                    leaf.indexes().clone(),
                ),
                TreeNode::Branch(grandchildren) => {
                    let child_fields = get_node_fields(grandchildren, pattern_lookup);
                    let pattern_name = pattern_lookup
                        .get(&child_fields)
                        .cloned()
                        .unwrap_or_else(|| "Unknown".to_string());
                    (pattern_name.clone(), pattern_name, BTreeSet::new())
                }
            };
            PatternField {
                name: name.clone(),
                rust_type,
                json_type,
                indexes,
                type_param: None,
            }
        })
        .collect();
    // Sort for consistent pattern matching (display order preserved in IndexMap)
    fields.sort_by(|a, b| a.name.cmp(&b.name));
    fields
}

/// Detect index patterns (sets of indexes that appear together on series).
pub fn detect_index_patterns(tree: &TreeNode) -> Vec<IndexSetPattern> {
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

/// Result of analyzing a pattern instance's base.
#[derive(Debug, Clone)]
pub struct PatternBaseResult {
    /// The computed base name for the pattern.
    pub(crate) base: String,
    /// Whether an outlier child was excluded to find the pattern.
    /// If true, pattern factory should not be used.
    pub(crate) has_outlier: bool,
    /// Whether this instance uses suffix mode (common prefix) or prefix mode (common suffix).
    /// Used to check compatibility with the pattern's mode.
    pub(crate) is_suffix_mode: bool,
    /// The field parts (suffix in suffix mode, prefix in prefix mode) for each field.
    /// Used to check if instance field parts match the pattern's field parts.
    pub(crate) field_parts: BTreeMap<String, String>,
}

impl PatternBaseResult {
    /// Create a default result that forces inlining (has_outlier = true).
    /// Use when no pattern base could be computed during lookup.
    pub(crate) fn force_inline() -> Self {
        Self {
            base: String::new(),
            has_outlier: true,
            is_suffix_mode: true,
            field_parts: BTreeMap::new(),
        }
    }
}

/// Get fields with child field information for generic pattern lookup.
pub fn get_fields_with_child_info(
    children: &IndexMap<String, TreeNode>,
    parent_name: &str,
    pattern_lookup: &BTreeMap<Vec<PatternField>, String>,
) -> Vec<(PatternField, Option<Vec<PatternField>>)> {
    children
        .iter()
        .map(|(name, node)| {
            let (rust_type, json_type, indexes, child_fields) = match node {
                TreeNode::Leaf(leaf) => (
                    leaf.kind().to_string(),
                    extract_json_type(&leaf.schema),
                    leaf.indexes().clone(),
                    None,
                ),
                TreeNode::Branch(grandchildren) => {
                    let child_fields = get_node_fields(grandchildren, pattern_lookup);
                    let pattern_name = pattern_lookup
                        .get(&child_fields)
                        .cloned()
                        .unwrap_or_else(|| child_type_name(parent_name, name));
                    (
                        pattern_name.clone(),
                        pattern_name,
                        BTreeSet::new(),
                        Some(child_fields),
                    )
                }
            };
            (
                PatternField {
                    name: name.clone(),
                    rust_type,
                    json_type,
                    indexes,
                    type_param: None,
                },
                child_fields,
            )
        })
        .collect()
}
