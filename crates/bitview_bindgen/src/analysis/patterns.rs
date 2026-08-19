//! Structural pattern detection using bottom-up analysis.
//!
//! This module detects repeating tree structures and analyzes them
//! using the bottom-up name deconstruction algorithm.

use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
};

use brk_types::{TreeNode, extract_json_type};

use super::analyze_pattern_modes;
use crate::{PatternBaseResult, PatternField, StructuralPattern, to_pascal_case};

/// Context for pattern detection, holding all intermediate state.
struct PatternContext {
    /// Maps field signatures to pattern names
    signature_to_pattern: BTreeMap<Vec<PatternField>, String>,
    /// Counts how many times each signature appears
    signature_counts: BTreeMap<Vec<PatternField>, usize>,
    /// Maps normalized signatures to pattern names (for naming consistency)
    normalized_to_name: BTreeMap<Vec<PatternField>, String>,
    /// Counts pattern name usage (for unique naming)
    name_counts: BTreeMap<String, usize>,
    /// Maps signatures to their child field lists
    signature_to_child_fields: BTreeMap<Vec<PatternField>, Vec<Vec<PatternField>>>,
}

impl PatternContext {
    fn new() -> Self {
        Self {
            signature_to_pattern: BTreeMap::new(),
            signature_counts: BTreeMap::new(),
            normalized_to_name: BTreeMap::new(),
            name_counts: BTreeMap::new(),
            signature_to_child_fields: BTreeMap::new(),
        }
    }
}

/// Detect structural patterns in the tree using a bottom-up approach.
///
/// Returns (patterns, concrete_to_pattern, concrete_to_type_param, node_bases).
/// Each pattern has its `mode` set based on analysis of all instances.
/// `node_bases` maps tree paths to their computed PatternBaseResult for use during generation.
pub fn detect_structural_patterns(
    tree: &TreeNode,
) -> (
    Vec<StructuralPattern>,
    BTreeMap<Vec<PatternField>, String>,
    BTreeMap<Vec<PatternField>, String>,
    BTreeMap<String, PatternBaseResult>,
) {
    let mut ctx = PatternContext::new();
    resolve_branch_patterns(tree, &mut ctx);

    let (generic_patterns, generic_mappings, type_mappings) =
        detect_generic_patterns(&ctx.signature_to_pattern);

    // Only include patterns that appear 2+ times for the patterns list
    let mut patterns: Vec<StructuralPattern> = ctx
        .signature_to_pattern
        .iter()
        .filter(|(sig, _)| {
            ctx.signature_counts.get(*sig).copied().unwrap_or(0) >= 2
                && !generic_mappings.contains_key(*sig)
        })
        .map(|(fields, name)| {
            let child_fields_list = ctx.signature_to_child_fields.get(fields);
            let fields_with_type_params = fields
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let type_param = child_fields_list
                        .and_then(|list| list.get(i))
                        .and_then(|cf| type_mappings.get(cf).cloned());
                    PatternField {
                        type_param,
                        ..f.clone()
                    }
                })
                .collect();
            StructuralPattern {
                name: name.clone(),
                fields: fields_with_type_params,
                mode: None, // Will be determined by analyze_pattern_modes
                is_generic: false,
            }
        })
        .collect();

    // Deduplicate patterns by name - different signatures can map to the same name
    // when their normalized forms match but they can't be unified as generics
    {
        let mut seen_names: BTreeSet<String> = BTreeSet::new();
        patterns.retain(|p| seen_names.insert(p.name.clone()));
    }

    patterns.extend(generic_patterns);

    // Build pattern lookup for mode analysis (patterns appearing 2+ times)
    let mut pattern_lookup: BTreeMap<Vec<PatternField>, String> = BTreeMap::new();
    for (sig, name) in &ctx.signature_to_pattern {
        if ctx.signature_counts.get(sig).copied().unwrap_or(0) >= 2 {
            pattern_lookup.insert(sig.clone(), name.clone());
        }
    }
    pattern_lookup.extend(generic_mappings.clone());

    let concrete_to_pattern = pattern_lookup.clone();

    // Analyze pattern modes (suffix vs prefix) from all instances
    // Also collects node bases for each tree path
    let node_bases = analyze_pattern_modes(tree, &mut patterns, &pattern_lookup);

    patterns.sort_by_key(|p| Reverse(p.fields.len()));
    (patterns, concrete_to_pattern, type_mappings, node_bases)
}

/// Detect generic patterns by grouping signatures by their normalized form.
fn detect_generic_patterns(
    signature_to_pattern: &BTreeMap<Vec<PatternField>, String>,
) -> (
    Vec<StructuralPattern>,
    BTreeMap<Vec<PatternField>, String>,
    BTreeMap<Vec<PatternField>, String>,
) {
    let mut normalized_groups: BTreeMap<
        Vec<PatternField>,
        Vec<(Vec<PatternField>, String, String)>,
    > = BTreeMap::new();

    for (fields, name) in signature_to_pattern {
        if let Some((normalized, extracted_type)) = normalize_fields_for_generic(fields) {
            normalized_groups.entry(normalized).or_default().push((
                fields.clone(),
                name.clone(),
                extracted_type,
            ));
        }
    }

    let mut patterns = Vec::new();
    let mut pattern_mappings: BTreeMap<Vec<PatternField>, String> = BTreeMap::new();
    let mut type_mappings: BTreeMap<Vec<PatternField>, String> = BTreeMap::new();

    for (normalized_fields, group) in normalized_groups {
        if group.len() >= 2 {
            let generic_name = group[0].1.clone();
            for (concrete_fields, _, extracted_type) in &group {
                pattern_mappings.insert(concrete_fields.clone(), generic_name.clone());
                type_mappings.insert(concrete_fields.clone(), extracted_type.clone());
            }
            patterns.push(StructuralPattern {
                name: generic_name,
                fields: normalized_fields,
                mode: None, // Will be determined by analyze_pattern_modes
                is_generic: true,
            });
        }
    }

    (patterns, pattern_mappings, type_mappings)
}

/// Normalize fields by replacing concrete value types with "T".
///
/// Handles two cases:
/// 1. All leaves have identical types (e.g., all `Sats`) -> normalize to `T`
/// 2. All leaves have wrapper types with the same inner type (e.g., `Open<Sats>`, `High<Sats>`)
///    -> normalize to `Open<T>`, `High<T>`, etc.
fn normalize_fields_for_generic(fields: &[PatternField]) -> Option<(Vec<PatternField>, String)> {
    let leaf_types: Vec<&str> = fields
        .iter()
        .filter(|f| f.is_leaf())
        .map(|f| f.rust_type.as_str())
        .collect();

    if leaf_types.is_empty() {
        return None;
    }

    let first_type = leaf_types[0];

    // Case 1: All leaf types are identical
    if leaf_types.iter().all(|t| *t == first_type) {
        let normalized = fields
            .iter()
            .map(|f| {
                if f.is_branch() {
                    f.clone()
                } else {
                    PatternField {
                        name: f.name.clone(),
                        rust_type: "T".to_string(),
                        json_type: "T".to_string(),
                        indexes: f.indexes.clone(),
                        type_param: None,
                    }
                }
            })
            .collect();
        return Some((normalized, crate::extract_inner_type(first_type)));
    }

    // Case 2: Check if all leaves have wrapper types with the same inner type
    // e.g., Open<Sats>, High<Sats>, Low<Sats>, Close<Sats> all have inner type Sats
    let inner_types: Vec<String> = leaf_types
        .iter()
        .map(|t| crate::extract_inner_type(t))
        .collect();

    let first_inner = &inner_types[0];

    // Only proceed if inner types differ from originals (meaning they had wrappers)
    // and all inner types are the same
    if inner_types.iter().all(|t| t == first_inner)
        && inner_types
            .iter()
            .zip(leaf_types.iter())
            .any(|(inner, orig)| inner != *orig)
    {
        let normalized = fields
            .iter()
            .map(|f| {
                if f.is_branch() {
                    f.clone()
                } else {
                    PatternField {
                        name: f.name.clone(),
                        rust_type: replace_inner_type(&f.rust_type, "T"),
                        json_type: replace_inner_type(&f.json_type, "T"),
                        indexes: f.indexes.clone(),
                        type_param: None,
                    }
                }
            })
            .collect();
        return Some((normalized, first_inner.clone()));
    }

    None
}

/// Replace the inner type of a wrapper generic with a new type.
/// e.g., `Open<Sats>` with replacement `T` -> `Open<T>`
fn replace_inner_type(type_str: &str, replacement: &str) -> String {
    if let Some(start) = type_str.find('<')
        && let Some(end) = type_str.rfind('>')
        && start < end
    {
        format!("{}<{}>", &type_str[..start], replacement)
    } else {
        replacement.to_string()
    }
}

/// Recursively resolve branch patterns bottom-up.
fn resolve_branch_patterns(
    node: &TreeNode,
    ctx: &mut PatternContext,
) -> Option<(String, Vec<PatternField>)> {
    let TreeNode::Branch(children) = node else {
        return None;
    };

    // Convert to sorted BTreeMap for consistent pattern detection
    let sorted_children: BTreeMap<_, _> = children.iter().collect();

    let mut fields: Vec<PatternField> = Vec::new();
    let mut child_fields_vec: Vec<Vec<PatternField>> = Vec::new();

    for (child_name, child_node) in sorted_children {
        let (rust_type, json_type, indexes, child_fields) = match child_node {
            TreeNode::Leaf(leaf) => (
                leaf.kind().to_string(),
                extract_json_type(&leaf.schema),
                leaf.indexes().clone(),
                Vec::new(),
            ),
            TreeNode::Branch(_) => {
                let (pattern_name, child_pattern_fields) = resolve_branch_patterns(child_node, ctx)
                    .unwrap_or_else(|| ("Unknown".to_string(), Vec::new()));
                (
                    pattern_name.clone(),
                    pattern_name,
                    BTreeSet::new(),
                    child_pattern_fields,
                )
            }
        };
        fields.push(PatternField {
            name: child_name.clone(),
            rust_type,
            json_type,
            indexes,
            type_param: None,
        });
        child_fields_vec.push(child_fields);
    }

    // Fields are already sorted since we iterated over BTreeMap
    *ctx.signature_counts.entry(fields.clone()).or_insert(0) += 1;

    ctx.signature_to_child_fields
        .entry(fields.clone())
        .or_insert(child_fields_vec);

    let pattern_name = if let Some(existing) = ctx.signature_to_pattern.get(&fields) {
        existing.clone()
    } else {
        let normalized = normalize_fields_for_naming(&fields);
        // Generate stable name from first word of each field (deduped, sorted)
        let first_words: BTreeSet<String> = fields
            .iter()
            .filter_map(|f| f.name.split('_').next())
            .map(to_pascal_case)
            .collect();
        let combined: String = first_words.into_iter().collect();
        let name = ctx
            .normalized_to_name
            .entry(normalized)
            .or_insert_with(|| generate_pattern_name(&combined, &mut ctx.name_counts))
            .clone();
        ctx.signature_to_pattern
            .insert(fields.clone(), name.clone());
        name
    };

    Some((pattern_name, fields))
}

/// Normalize fields for naming (same structure = same name).
/// Only erases leaf types when all leaves share the same type — this ensures
/// mixed-type signatures (e.g., StoredU32 raw + StoredU64 cumulative) get a
/// different name than same-type signatures that can be genericized.
fn normalize_fields_for_naming(fields: &[PatternField]) -> Vec<PatternField> {
    let leaf_types: Vec<&str> = fields
        .iter()
        .filter(|f| !f.is_branch())
        .map(|f| f.rust_type.as_str())
        .collect();
    let all_same = !leaf_types.is_empty() && leaf_types.iter().all(|t| *t == leaf_types[0]);

    fields
        .iter()
        .map(|f| {
            if f.is_branch() || !all_same {
                f.clone()
            } else {
                PatternField {
                    name: f.name.clone(),
                    rust_type: "_".to_string(),
                    json_type: "_".to_string(),
                    indexes: f.indexes.clone(),
                    type_param: None,
                }
            }
        })
        .collect()
}

/// Generate a unique pattern name.
fn generate_pattern_name(field_name: &str, name_counts: &mut BTreeMap<String, usize>) -> String {
    let pascal = to_pascal_case(field_name);
    let sanitized = if pascal.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("_{}", pascal)
    } else {
        pascal
    };

    let base_name = format!("{}Pattern", sanitized);
    let count = name_counts.entry(base_name.clone()).or_insert(0);
    *count += 1;

    if *count == 1 {
        base_name
    } else {
        format!("{}{}", base_name, count)
    }
}
