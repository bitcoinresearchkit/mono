//! Shared tree generation helpers.

use std::collections::{BTreeMap, BTreeSet};

use brk_types::TreeNode;

use crate::{
    ClientMetadata, PatternBaseResult, PatternField, child_type_name, get_fields_with_child_info,
};

/// Build a child path by appending a child name to a parent path.
/// Uses "/" as separator. If parent is empty, returns just the child name.
#[inline]
pub fn build_child_path(parent: &str, child: &str) -> String {
    if parent.is_empty() {
        child.to_string()
    } else {
        format!("{}/{}", parent, child)
    }
}

/// Pre-computed context for a single child node.
pub struct ChildContext<'a> {
    /// The child's field name in the tree.
    pub name: &'a str,
    /// The child node.
    pub node: &'a TreeNode,
    /// The field info for this child (with type_param set for generic patterns).
    pub field: PatternField,
    /// Pattern analysis result.
    pub base_result: PatternBaseResult,
    /// Whether this is a leaf node.
    pub is_leaf: bool,
    /// Whether to use an inline type instead of a pattern type (only meaningful for branches).
    pub should_inline: bool,
    /// The type name to use for inline branches.
    pub inline_type_name: String,
}

/// Context for generating a tree node, returned by `prepare_tree_node`.
pub struct TreeNodeContext<'a> {
    /// Pre-computed context for each child.
    pub children: Vec<ChildContext<'a>>,
}

/// Prepare a tree node for generation.
/// Returns None if the node should be skipped (not a branch, already generated,
/// or matches a parameterizable pattern).
///
/// The `path` parameter is the tree path to this node (e.g., "distribution/utxoCohorts").
/// It's used to look up pre-computed PatternBaseResult from the analysis phase.
pub fn prepare_tree_node<'a>(
    node: &'a TreeNode,
    name: &str,
    path: &str,
    pattern_lookup: &BTreeMap<Vec<PatternField>, String>,
    metadata: &ClientMetadata,
    generated: &mut BTreeSet<String>,
) -> Option<TreeNodeContext<'a>> {
    let TreeNode::Branch(branch_children) = node else {
        return None;
    };

    let fields_with_child_info = get_fields_with_child_info(branch_children, name, pattern_lookup);
    let fields: Vec<PatternField> = fields_with_child_info
        .iter()
        .map(|(f, _)| f.clone())
        .collect();

    // Look up the pre-computed base result, or use a default that forces inlining
    let base_result = metadata
        .get_node_base(path)
        .cloned()
        .unwrap_or_else(PatternBaseResult::force_inline);

    // Skip if this matches a parameterizable pattern AND has no outlier AND field parts match
    let pattern_compatible = pattern_lookup
        .get(&fields)
        .and_then(|name| metadata.find_pattern(name))
        .is_none_or(|p| {
            p.is_suffix_mode() == base_result.is_suffix_mode
                && p.field_parts_match(&base_result.field_parts)
        });
    if let Some(pattern_name) = pattern_lookup.get(&fields)
        && pattern_name != name
        && metadata.is_parameterizable(pattern_name)
        && !base_result.has_outlier
        && pattern_compatible
    {
        return None;
    }

    // Skip if already generated
    if generated.contains(name) {
        return None;
    }
    generated.insert(name.to_string());

    // Build child contexts with pre-computed decisions
    let children: Vec<ChildContext<'a>> = branch_children
        .iter()
        .zip(fields_with_child_info)
        .map(|((child_name, child_node), (mut field, child_fields))| {
            let is_leaf = matches!(child_node, TreeNode::Leaf(_));

            // Set type_param for generic patterns so field_type_annotation works directly
            if let Some(cf) = &child_fields {
                field.type_param = metadata.get_type_param(cf).cloned();
            }

            // Build child path and look up its pre-computed base result
            let child_path = build_child_path(path, child_name);
            let base_result = metadata
                .get_node_base(&child_path)
                .cloned()
                .unwrap_or_else(PatternBaseResult::force_inline);

            // Single lookup for the child's matching pattern (avoids repeated scans)
            let matching_pattern = child_fields
                .as_ref()
                .and_then(|cf| metadata.find_pattern_by_fields(cf));

            let matches_any_pattern = matching_pattern.is_some();
            let pattern_compatible = matching_pattern.is_none_or(|p| {
                p.is_suffix_mode() == base_result.is_suffix_mode
                    && p.field_parts_match(&base_result.field_parts)
            });
            let is_parameterizable =
                matching_pattern.is_none_or(|p| metadata.is_parameterizable(&p.name));

            // should_inline determines if we generate an inline struct type
            let should_inline = !is_leaf
                && (!matches_any_pattern
                    || !pattern_compatible
                    || !is_parameterizable
                    || base_result.has_outlier);

            let inline_type_name = if should_inline {
                child_type_name(name, child_name)
            } else {
                String::new()
            };

            ChildContext {
                name: child_name,
                node: child_node,
                field,
                base_result,
                is_leaf,
                should_inline,
                inline_type_name,
            }
        })
        .collect();

    Some(TreeNodeContext { children })
}
