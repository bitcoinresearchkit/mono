//! Pattern mode detection and field part extraction.
//!
//! This module analyzes pattern instances to detect whether they use
//! suffix mode (fields append to acc) or prefix mode (fields prepend to acc),
//! and extracts the field parts (relatives or prefixes) for code generation.

use std::{collections::BTreeMap, iter};

use bitview_catalog::TreeNode;

use super::{
    find_common_prefix, find_common_suffix, get_node_fields, get_shortest_leaf_name,
    normalize_prefix,
};
use crate::{PatternBaseResult, PatternField, PatternMode, StructuralPattern, build_child_path};

/// Result of analyzing a single pattern instance.
#[derive(Debug, Clone)]
struct InstanceAnalysis {
    /// The base to return to parent (used for nesting)
    base: String,
    /// For suffix mode: field -> relative name
    /// For prefix mode: field -> prefix
    field_parts: BTreeMap<String, String>,
    /// Whether this instance appears to be suffix mode
    is_suffix_mode: bool,
    /// Whether children have no common prefix/suffix (outlier naming like sopr/asopr)
    has_outlier: bool,
}

/// Analyze all pattern instances and determine their modes.
///
/// This is the main entry point for mode detection. It processes
/// the tree bottom-up, collecting analysis for each pattern instance,
/// then determines the consistent mode for each pattern.
///
/// Returns a map from tree paths to their computed PatternBaseResult.
/// This map is used during generation to check pattern compatibility.
pub(crate) fn analyze_pattern_modes(
    tree: &TreeNode,
    patterns: &mut [StructuralPattern],
    pattern_lookup: &BTreeMap<Vec<PatternField>, String>,
) -> BTreeMap<String, PatternBaseResult> {
    // Collect analyses from all instances, keyed by pattern name
    let mut all_analyses: BTreeMap<String, Vec<InstanceAnalysis>> = BTreeMap::new();
    // Base results for each node, keyed by tree path
    let mut node_bases: BTreeMap<String, PatternBaseResult> = BTreeMap::new();
    // Track which tree path belongs to which pattern (avoids re-traversal)
    let mut path_to_pattern: BTreeMap<String, String> = BTreeMap::new();

    // Pass 1: bottom-up traversal
    collect_instance_analyses(
        tree,
        "",
        pattern_lookup,
        &mut all_analyses,
        &mut node_bases,
        &mut path_to_pattern,
    );

    // Determine initial modes
    for pattern in patterns.iter_mut() {
        if let Some(analyses) = all_analyses.get(&pattern.name) {
            pattern.mode = determine_pattern_mode(analyses, &pattern.fields);
        }
    }

    // Pass 2: fill mixed-empty field_parts now that pattern modes are known
    fill_mixed_empty_field_parts(tree, "", pattern_lookup, patterns, &mut node_bases);

    // Re-determine modes from updated node_bases (no tree re-traversal needed)
    let mut updated_analyses: BTreeMap<String, Vec<InstanceAnalysis>> = BTreeMap::new();
    for (path, pattern_name) in &path_to_pattern {
        if let Some(br) = node_bases.get(path) {
            updated_analyses
                .entry(pattern_name.clone())
                .or_default()
                .push(InstanceAnalysis {
                    base: br.base.clone(),
                    field_parts: br.field_parts.clone(),
                    is_suffix_mode: br.is_suffix_mode,
                    has_outlier: br.has_outlier,
                });
        }
    }
    for pattern in patterns.iter_mut() {
        if let Some(analyses) = updated_analyses.get(&pattern.name) {
            pattern.mode = determine_pattern_mode(analyses, &pattern.fields);
        }
    }

    node_bases
}

/// Second pass: fill empty field_parts for nodes that have a mix of empty and
/// non-empty parts, using shortest leaf names for children that need disc.
fn fill_mixed_empty_field_parts(
    node: &TreeNode,
    path: &str,
    pattern_lookup: &BTreeMap<Vec<PatternField>, String>,
    patterns: &[StructuralPattern],
    node_bases: &mut BTreeMap<String, PatternBaseResult>,
) {
    let TreeNode::Branch(children) = node else {
        return;
    };

    // Recurse first (bottom-up)
    for (field_name, child_node) in children {
        let child_path = build_child_path(path, field_name);
        fill_mixed_empty_field_parts(
            child_node,
            &child_path,
            pattern_lookup,
            patterns,
            node_bases,
        );
    }

    // Check if this node has mixed empty/non-empty field_parts
    let Some(base_result) = node_bases.get(path) else {
        return;
    };
    let has_empty = base_result.field_parts.values().any(|v| v.is_empty());
    let has_nonempty = base_result.field_parts.values().any(|v| !v.is_empty());
    if !has_empty || !has_nonempty {
        return;
    }

    let prefix = format!("{}_", base_result.base);
    let mut updates: Vec<(String, String)> = Vec::new();

    for (field_name, child_node) in children {
        let part = base_result.field_parts.get(field_name.as_str());
        if !part.is_some_and(|p| p.is_empty()) {
            continue;
        }

        // Check if the child's pattern is templated (needs disc from parent)
        let child_pattern_is_templated = if let TreeNode::Branch(ch) = child_node {
            let child_fields = get_node_fields(ch, pattern_lookup);
            pattern_lookup
                .get(&child_fields)
                .and_then(|name| patterns.iter().find(|p| &p.name == name))
                .is_some_and(|p| p.is_templated())
        } else {
            false
        };

        // Only fill if the child needs disc (templated) or is a leaf
        let is_leaf = matches!(child_node, TreeNode::Leaf(_));
        if !child_pattern_is_templated && !is_leaf {
            continue;
        }

        if let Some(leaf) = get_shortest_leaf_name(child_node)
            && let Some(suffix) = leaf.strip_prefix(&prefix)
            && !suffix.is_empty()
            && suffix.contains(field_name.trim_start_matches('_'))
            && suffix.len() >= field_name.trim_start_matches('_').len()
        {
            updates.push((field_name.clone(), suffix.to_string()));
        }
    }

    if !updates.is_empty() {
        let base_result = node_bases.get_mut(path).unwrap();
        for (field_name, suffix) in updates {
            base_result.field_parts.insert(field_name, suffix);
        }
    }
}

/// Recursively collect instance analyses bottom-up.
/// Returns the "base" for this node (used by parent for its analysis).
///
/// Also stores the PatternBaseResult for each node in `node_bases`, keyed by path.
fn collect_instance_analyses(
    node: &TreeNode,
    path: &str,
    pattern_lookup: &BTreeMap<Vec<PatternField>, String>,
    all_analyses: &mut BTreeMap<String, Vec<InstanceAnalysis>>,
    node_bases: &mut BTreeMap<String, PatternBaseResult>,
    path_to_pattern: &mut BTreeMap<String, String>,
) -> Option<String> {
    match node {
        TreeNode::Leaf(leaf) => {
            // Leaves return their series name as the base
            Some(leaf.name().to_string())
        }
        TreeNode::Branch(children) => {
            // First, process all children recursively (bottom-up)
            let mut child_bases: BTreeMap<String, String> = BTreeMap::new();
            for (field_name, child_node) in children {
                let child_path = build_child_path(path, field_name);
                if let Some(base) = collect_instance_analyses(
                    child_node,
                    &child_path,
                    pattern_lookup,
                    all_analyses,
                    node_bases,
                    path_to_pattern,
                ) {
                    child_bases.insert(field_name.clone(), base);
                }
            }

            if child_bases.is_empty() {
                return None;
            }

            // Analyze this instance
            let mut analysis = if children.field_suffixes {
                analyze_field_suffixes(&child_bases, path)
            } else {
                analyze_instance(&child_bases)
            };

            // When some field_parts are empty (children returned the same base),
            // replace empty parts with discriminators derived from shortest leaf names.
            let all_empty = analysis.field_parts.len() > 1
                && analysis.field_parts.values().all(|v| v.is_empty());
            if all_empty {
                // All-empty case: all children returned the same base.
                // Use shortest leaf to derive field_parts for fields whose key
                // matches the series suffix (e.g., pct1 → suffix "pct1").
                let prefix = format!("{}_", analysis.base);
                let mut any_filled = false;
                for (field_name, child_node) in children {
                    if let Some(part) = analysis.field_parts.get(field_name)
                        && part.is_empty()
                        && let Some(leaf) = get_shortest_leaf_name(child_node)
                        && let Some(suffix) = leaf.strip_prefix(&prefix)
                        && !suffix.is_empty()
                        && suffix.starts_with(field_name.trim_start_matches('_'))
                    {
                        analysis
                            .field_parts
                            .insert(field_name.clone(), suffix.to_string());
                        any_filled = true;
                    }
                }

                // If no fields could be filled and all children are the same type,
                // mark as outlier so the tree inlines instead of using identity
                // (handles patterns like period windows where field keys differ
                // from series suffixes: all/_4y don't match 0sd/0sd_4y).
                // When children are different types (like absolute/rate), identity
                // is correct — each child handles its own suffixes internally.
                if !any_filled {
                    let child_fields = get_node_fields(children, pattern_lookup);
                    let all_same_type = child_fields
                        .windows(2)
                        .all(|w| w[0].rust_type == w[1].rust_type);
                    if all_same_type {
                        analysis.has_outlier = true;
                    }
                }
            }

            // A parent factory also constructs its descendants; any descendant
            // requiring explicit names prevents safely templating the parent.
            analysis.has_outlier |= children.keys().any(|field| {
                node_bases
                    .get(&build_child_path(path, field))
                    .is_some_and(|child| child.has_outlier)
            });

            // Store the base result for this node
            node_bases.insert(
                path.to_string(),
                PatternBaseResult {
                    base: analysis.base.clone(),
                    has_outlier: analysis.has_outlier,
                    is_suffix_mode: analysis.is_suffix_mode,
                    field_parts: analysis.field_parts.clone(),
                },
            );

            // Get the pattern name for this node (if any)
            let fields = get_node_fields(children, pattern_lookup);
            if let Some(pattern_name) = pattern_lookup.get(&fields) {
                path_to_pattern.insert(path.to_string(), pattern_name.clone());
                all_analyses
                    .entry(pattern_name.clone())
                    .or_default()
                    .push(analysis.clone());
            }

            // Return the base for parent.
            // For outlier nodes (no common prefix among children), return the
            // shortest leaf name so the parent can still detect naming patterns.
            if analysis.has_outlier {
                Some(get_shortest_leaf_name(node).unwrap_or(analysis.base))
            } else {
                Some(analysis.base)
            }
        }
    }
}

/// A declared family uses its catalog keys directly, not prefix/suffix guessing.
fn analyze_field_suffixes(child_bases: &BTreeMap<String, String>, path: &str) -> InstanceAnalysis {
    let mut base = None;
    for (field, child_base) in child_bases {
        let suffix = format!("_{field}");
        let parent = child_base.strip_suffix(&suffix).unwrap_or_else(|| {
            panic!("Declared field suffix {suffix:?} does not match {child_base:?} at {path}")
        });
        if let Some(base) = base {
            assert_eq!(base, parent, "Conflicting declared family bases at {path}");
        } else {
            base = Some(parent);
        }
    }
    InstanceAnalysis {
        base: base.unwrap().to_string(),
        field_parts: child_bases
            .keys()
            .map(|key| (key.clone(), key.clone()))
            .collect(),
        is_suffix_mode: true,
        has_outlier: false,
    }
}

/// Try to detect a template pattern when instances have different field_parts.
///
/// Supports two cases:
/// 1. **Embedded discriminator**: a substring varies per instance within field_parts.
///    E.g., `ratio_pct99_ppm` vs `ratio_pct1_ppm` → template `ratio_{disc}_ppm`
/// 2. **Suffix discriminator**: a common suffix is appended to all field_parts.
///    E.g., `ratio_sd` vs `ratio_sd_4y` → template `ratio_sd{disc}`
fn try_detect_template(
    majority: &[&InstanceAnalysis],
    fields: &[PatternField],
) -> Option<PatternMode> {
    if majority.len() < 2 {
        return None;
    }

    // Strategy 1: suffix discriminator (e.g., ratio_sd vs ratio_sd_4y)
    if let Some(mode) = try_suffix_disc(majority, fields) {
        return Some(mode);
    }

    // Strategy 2: embedded discriminator (e.g., ratio_pct99_ppm vs ratio_pct1_ppm)
    try_embedded_disc(majority, fields)
}

/// Strategy 1: embedded discriminator (e.g., pct99 inside ratio_pct99_ppm)
fn try_embedded_disc(
    majority: &[&InstanceAnalysis],
    fields: &[PatternField],
) -> Option<PatternMode> {
    let first = &majority[0];
    let second = &majority[1];

    // Find the discriminator: shortest non-empty field_part that differs
    let disc_field = fields
        .iter()
        .filter_map(|f| first.field_parts.get(&f.name).map(|v| (&f.name, v)))
        .filter(|(_, v)| !v.is_empty())
        .min_by_key(|(_, v)| v.len())?;

    let disc_first = disc_field.1;
    let disc_second = second.field_parts.get(disc_field.0)?;

    if disc_first == disc_second || disc_first.is_empty() || disc_second.is_empty() {
        return None;
    }

    // Build templates by replacing the discriminator with {disc}
    let mut templates = BTreeMap::new();
    for field in fields {
        let part = first.field_parts.get(&field.name)?;
        let template = part.replacen(disc_first, "{disc}", 1);
        templates.insert(field.name.clone(), template);
    }

    // Verify ALL instances match
    for analysis in majority {
        let inst_disc = analysis.field_parts.get(disc_field.0)?;
        for field in fields {
            let part = analysis.field_parts.get(&field.name)?;
            let expected = templates.get(&field.name)?.replace("{disc}", inst_disc);
            if part != &expected {
                return None;
            }
        }
    }

    Some(PatternMode::Templated { templates })
}

/// Strategy 2: suffix discriminator (e.g., all field_parts differ by `_4y` suffix)
fn try_suffix_disc(majority: &[&InstanceAnalysis], fields: &[PatternField]) -> Option<PatternMode> {
    let first = &majority[0];

    // Use a non-empty field to detect the suffix
    let ref_field = fields
        .iter()
        .find(|f| {
            first
                .field_parts
                .get(&f.name)
                .is_some_and(|v| !v.is_empty())
        })
        .map(|f| &f.name)?;
    let ref_first = first.field_parts.get(ref_field)?;

    // Build templates from the first instance
    // Non-empty parts get {disc} appended; empty parts (identity) stay empty
    let mut templates = BTreeMap::new();
    for field in fields {
        let part = first.field_parts.get(&field.name)?;
        if part.is_empty() {
            templates.insert(field.name.clone(), String::new());
        } else {
            templates.insert(field.name.clone(), format!("{part}{{disc}}"));
        }
    }

    // Verify ALL other instances: non-empty parts differ by the same suffix
    for analysis in &majority[1..] {
        let ref_other = analysis.field_parts.get(ref_field)?;
        let suffix = ref_other.strip_prefix(ref_first)?;

        for field in fields {
            let first_part = first.field_parts.get(&field.name)?;
            let other_part = analysis.field_parts.get(&field.name)?;

            if first_part.is_empty() {
                // Identity field — must be empty OR equal to the suffix
                if other_part.is_empty() {
                    // stays empty — ok
                } else if other_part == suffix {
                    // empty in first, equals suffix in other — disc IS the part
                    templates.insert(field.name.clone(), "{disc}".to_string());
                } else {
                    return None;
                }
            } else {
                let expected = format!("{first_part}{suffix}");
                if other_part != &expected {
                    return None;
                }
            }
        }
    }

    Some(PatternMode::Templated { templates })
}

/// Analyze a single pattern instance from its child bases.
fn analyze_instance(child_bases: &BTreeMap<String, String>) -> InstanceAnalysis {
    let bases: Vec<&str> = child_bases.values().map(|s| s.as_str()).collect();

    // Try suffix mode first: look for common prefix among children
    if let Some(common_prefix) = find_common_prefix(&bases) {
        let base = common_prefix.trim_end_matches('_').to_string();
        let mut field_parts = BTreeMap::new();

        for (field_name, child_base) in child_bases {
            // Relative = child_base with common prefix stripped
            // If child_base equals base, relative is empty (identity field)
            let relative = if child_base == &base {
                String::new()
            } else {
                child_base
                    .strip_prefix(&common_prefix)
                    .unwrap_or(child_base)
                    .to_string()
            };
            field_parts.insert(field_name.clone(), relative);
        }

        return InstanceAnalysis {
            base,
            field_parts,
            is_suffix_mode: true,
            has_outlier: false,
        };
    }

    // Try prefix mode: look for common suffix among children
    if let Some(common_suffix) = find_common_suffix(&bases) {
        let base = common_suffix.trim_start_matches('_').to_string();
        let mut field_parts = BTreeMap::new();

        for (field_name, child_base) in child_bases {
            // Prefix = child_base with common suffix stripped, normalized to end with _
            let prefix = child_base
                .strip_suffix(&common_suffix)
                .map(normalize_prefix)
                .unwrap_or_default();
            field_parts.insert(field_name.clone(), prefix);
        }

        return InstanceAnalysis {
            base,
            field_parts,
            is_suffix_mode: false,
            has_outlier: false,
        };
    }

    // No common prefix or suffix - use empty base so _m(base, relative) returns just the relative.
    // No common prefix or suffix — outlier naming (e.g., sopr/asopr/adj_).
    // Children have unrelated series names that can't be parameterized.
    let field_parts = child_bases
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    InstanceAnalysis {
        base: String::new(),
        field_parts,
        is_suffix_mode: true,
        has_outlier: true,
    }
}

/// Determine the consistent mode for a pattern from all its instances.
/// Picks the majority mode (suffix vs prefix), then requires all instances
/// in that mode to agree on field_parts. Minority-mode instances get inlined.
fn determine_pattern_mode(
    analyses: &[InstanceAnalysis],
    fields: &[PatternField],
) -> Option<PatternMode> {
    // Filter out outlier instances — they'll be inlined individually at generation
    // time via the per-instance has_outlier check in prepare_tree_node.
    // Don't let a single outlier poison the entire pattern.
    let non_outlier = analyses.iter().filter(|a| !a.has_outlier);

    // Pick the majority mode
    let suffix_count = non_outlier.clone().filter(|a| a.is_suffix_mode).count();
    let is_suffix = suffix_count * 2 >= non_outlier.clone().count();

    // All instances of the majority mode must agree on field_parts
    let mut majority = non_outlier.filter(|a| a.is_suffix_mode == is_suffix);
    let first_majority = majority.next()?;

    // Verify all required fields have parts
    for field in fields {
        if !first_majority.field_parts.contains_key(&field.name) {
            return None;
        }
    }

    if majority
        .clone()
        .all(|a| a.field_parts == first_majority.field_parts)
    {
        let field_parts = first_majority.field_parts.clone();

        return if is_suffix {
            Some(PatternMode::Suffix {
                relatives: field_parts,
            })
        } else {
            Some(PatternMode::Prefix {
                prefixes: field_parts,
            })
        };
    }

    // Instances disagree on field_parts. Try to detect a template pattern:
    // if each field's value varies by exactly one substring that's different
    // per instance, we can use a Templated mode with {disc} placeholder.
    if is_suffix {
        let majority: Vec<_> = iter::once(first_majority).chain(majority).collect();
        try_detect_template(&majority, fields)
    } else {
        None
    }
}
