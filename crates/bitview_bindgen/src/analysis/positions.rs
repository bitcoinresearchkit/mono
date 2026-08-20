//! Pattern mode detection and field part extraction.
//!
//! This module analyzes pattern instances to detect whether they use
//! suffix mode (fields append to acc) or prefix mode (fields prepend to acc),
//! and extracts the field parts (relatives or prefixes) for code generation.

use std::collections::BTreeMap;

use bitview_types::TreeNode;

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
pub fn analyze_pattern_modes(
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
            let mut analysis = analyze_instance(&child_bases);

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
    analyses.first()?;

    // Filter out outlier instances — they'll be inlined individually at generation
    // time via the per-instance has_outlier check in prepare_tree_node.
    // Don't let a single outlier poison the entire pattern.
    let non_outlier: Vec<&InstanceAnalysis> = analyses.iter().filter(|a| !a.has_outlier).collect();
    if non_outlier.is_empty() {
        return None;
    }

    // Pick the majority mode
    let suffix_count = non_outlier.iter().filter(|a| a.is_suffix_mode).count();
    let is_suffix = suffix_count * 2 >= non_outlier.len();

    // All instances of the majority mode must agree on field_parts
    let majority: Vec<&InstanceAnalysis> = non_outlier
        .into_iter()
        .filter(|a| a.is_suffix_mode == is_suffix)
        .collect();
    let first_majority = majority.first()?;

    // Verify all required fields have parts
    for field in fields {
        if !first_majority.field_parts.contains_key(&field.name) {
            return None;
        }
    }

    if majority
        .iter()
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
        try_detect_template(&majority, fields)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_instance_suffix_mode() {
        let mut child_bases = BTreeMap::new();
        child_bases.insert("max".to_string(), "lth_cost_basis_max".to_string());
        child_bases.insert("min".to_string(), "lth_cost_basis_min".to_string());
        child_bases.insert("percentiles".to_string(), "lth_cost_basis".to_string());

        let analysis = analyze_instance(&child_bases);

        assert!(analysis.is_suffix_mode);
        assert_eq!(analysis.base, "lth_cost_basis");
        assert_eq!(analysis.field_parts.get("max"), Some(&"max".to_string()));
        assert_eq!(analysis.field_parts.get("min"), Some(&"min".to_string()));
        assert_eq!(
            analysis.field_parts.get("percentiles"),
            Some(&"".to_string())
        );
    }

    #[test]
    fn test_analyze_instance_prefix_mode() {
        // Period-prefixed series like "1y_lump_sum_stack", "1m_lump_sum_stack"
        // share a common suffix "_lump_sum_stack" with different period prefixes
        let mut child_bases = BTreeMap::new();
        child_bases.insert("_1y".to_string(), "1y_lump_sum_stack".to_string());
        child_bases.insert("_1m".to_string(), "1m_lump_sum_stack".to_string());
        child_bases.insert("_1w".to_string(), "1w_lump_sum_stack".to_string());

        let analysis = analyze_instance(&child_bases);

        assert!(!analysis.is_suffix_mode);
        assert_eq!(analysis.base, "lump_sum_stack");
        assert_eq!(analysis.field_parts.get("_1y"), Some(&"1y_".to_string()));
        assert_eq!(analysis.field_parts.get("_1m"), Some(&"1m_".to_string()));
        assert_eq!(analysis.field_parts.get("_1w"), Some(&"1w_".to_string()));
    }

    #[test]
    fn test_analyze_instance_root_suffix() {
        // At root level with suffix naming convention
        let mut child_bases = BTreeMap::new();
        child_bases.insert("max".to_string(), "cost_basis_max".to_string());
        child_bases.insert("min".to_string(), "cost_basis_min".to_string());
        child_bases.insert("percentiles".to_string(), "cost_basis".to_string());

        let analysis = analyze_instance(&child_bases);

        // With suffix naming, common prefix is "cost_basis_" (since cost_basis is one of the names)
        assert!(analysis.is_suffix_mode);
        assert_eq!(analysis.base, "cost_basis");
        assert_eq!(analysis.field_parts.get("max"), Some(&"max".to_string()));
        assert_eq!(analysis.field_parts.get("min"), Some(&"min".to_string()));
        assert_eq!(
            analysis.field_parts.get("percentiles"),
            Some(&"".to_string())
        );
    }

    #[test]
    fn test_determine_pattern_mode_majority_voting() {
        // Test that majority voting works when instances have mixed modes.
        // This simulates CostBasisPattern2: most instances use suffix mode,
        // but root-level uses prefix mode (max_cost_basis, min_cost_basis, cost_basis).
        use std::collections::BTreeSet;

        let fields = vec![
            PatternField {
                name: "max".to_string(),
                rust_type: "TestType".to_string(),
                json_type: "number".to_string(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
            PatternField {
                name: "min".to_string(),
                rust_type: "TestType".to_string(),
                json_type: "number".to_string(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
            PatternField {
                name: "percentiles".to_string(),
                rust_type: "TestType".to_string(),
                json_type: "number".to_string(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
        ];

        // 3 suffix mode instances (majority)
        let suffix1 = InstanceAnalysis {
            base: "lth_cost_basis".to_string(),
            field_parts: [
                ("max".to_string(), "max".to_string()),
                ("min".to_string(), "min".to_string()),
                ("percentiles".to_string(), "".to_string()),
            ]
            .into_iter()
            .collect(),
            is_suffix_mode: true,
            has_outlier: false,
        };
        let suffix2 = InstanceAnalysis {
            base: "sth_cost_basis".to_string(),
            field_parts: [
                ("max".to_string(), "max".to_string()),
                ("min".to_string(), "min".to_string()),
                ("percentiles".to_string(), "".to_string()),
            ]
            .into_iter()
            .collect(),
            is_suffix_mode: true,
            has_outlier: false,
        };
        let suffix3 = InstanceAnalysis {
            base: "utxo_cost_basis".to_string(),
            field_parts: [
                ("max".to_string(), "max".to_string()),
                ("min".to_string(), "min".to_string()),
                ("percentiles".to_string(), "".to_string()),
            ]
            .into_iter()
            .collect(),
            is_suffix_mode: true,
            has_outlier: false,
        };

        // 1 prefix mode instance (minority - root level)
        let prefix1 = InstanceAnalysis {
            base: "cost_basis".to_string(),
            field_parts: [
                ("max".to_string(), "max_".to_string()),
                ("min".to_string(), "min_".to_string()),
                ("percentiles".to_string(), "".to_string()),
            ]
            .into_iter()
            .collect(),
            is_suffix_mode: false,
            has_outlier: false,
        };

        let analyses = vec![suffix1, suffix2, suffix3, prefix1];

        let mode = determine_pattern_mode(&analyses, &fields);

        // Should pick suffix mode (majority) with the common field_parts
        assert!(mode.is_some());
        match mode.unwrap() {
            PatternMode::Suffix { relatives } => {
                assert_eq!(relatives.get("max"), Some(&"max".to_string()));
                assert_eq!(relatives.get("min"), Some(&"min".to_string()));
                assert_eq!(relatives.get("percentiles"), Some(&"".to_string()));
            }
            PatternMode::Prefix { .. } => panic!("Expected suffix mode, got prefix mode"),
            PatternMode::Templated { .. } => panic!("Expected suffix mode, got templated mode"),
        }
    }

    #[test]
    fn test_determine_pattern_mode_all_same() {
        // Test when all instances agree on mode and field_parts
        use std::collections::BTreeSet;

        let fields = vec![
            PatternField {
                name: "max".to_string(),
                rust_type: "TestType".to_string(),
                json_type: "number".to_string(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
            PatternField {
                name: "min".to_string(),
                rust_type: "TestType".to_string(),
                json_type: "number".to_string(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
        ];

        let instance1 = InstanceAnalysis {
            base: "series_a".to_string(),
            field_parts: [
                ("max".to_string(), "max".to_string()),
                ("min".to_string(), "min".to_string()),
            ]
            .into_iter()
            .collect(),
            is_suffix_mode: true,
            has_outlier: false,
        };
        let instance2 = InstanceAnalysis {
            base: "series_b".to_string(),
            field_parts: [
                ("max".to_string(), "max".to_string()),
                ("min".to_string(), "min".to_string()),
            ]
            .into_iter()
            .collect(),
            is_suffix_mode: true,
            has_outlier: false,
        };

        let analyses = vec![instance1, instance2];
        let mode = determine_pattern_mode(&analyses, &fields);

        assert!(mode.is_some());
        match mode.unwrap() {
            PatternMode::Suffix { relatives } => {
                assert_eq!(relatives.get("max"), Some(&"max".to_string()));
                assert_eq!(relatives.get("min"), Some(&"min".to_string()));
            }
            PatternMode::Prefix { .. } => panic!("Expected suffix mode"),
            PatternMode::Templated { .. } => panic!("Expected suffix mode, got templated"),
        }
    }

    #[test]
    fn test_embedded_disc_percentile_bands() {
        use std::collections::BTreeSet;
        let fields = vec![
            PatternField {
                name: "ppm".into(),
                rust_type: "T".into(),
                json_type: "n".into(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
            PatternField {
                name: "price".into(),
                rust_type: "T".into(),
                json_type: "n".into(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
            PatternField {
                name: "ratio".into(),
                rust_type: "T".into(),
                json_type: "n".into(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
        ];
        let pct99 = InstanceAnalysis {
            base: "realized_price".into(),
            field_parts: [
                ("ppm".into(), "ratio_pct99_ppm".into()),
                ("price".into(), "pct99".into()),
                ("ratio".into(), "ratio_pct99".into()),
            ]
            .into_iter()
            .collect(),
            is_suffix_mode: true,
            has_outlier: false,
        };
        let pct1 = InstanceAnalysis {
            base: "realized_price".into(),
            field_parts: [
                ("ppm".into(), "ratio_pct1_ppm".into()),
                ("price".into(), "pct1".into()),
                ("ratio".into(), "ratio_pct1".into()),
            ]
            .into_iter()
            .collect(),
            is_suffix_mode: true,
            has_outlier: false,
        };
        let mode = determine_pattern_mode(&[pct99, pct1], &fields);
        assert!(mode.is_some());
        match mode.unwrap() {
            PatternMode::Templated { templates } => {
                assert_eq!(templates.get("ppm").unwrap(), "ratio_{disc}_ppm");
                assert_eq!(templates.get("price").unwrap(), "{disc}");
                assert_eq!(templates.get("ratio").unwrap(), "ratio_{disc}");
            }
            other => panic!("Expected Templated, got {:?}", other),
        }
    }

    #[test]
    fn test_suffix_disc_period_windows() {
        use std::collections::BTreeSet;
        let fields = vec![
            PatternField {
                name: "p1sd".into(),
                rust_type: "T".into(),
                json_type: "n".into(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
            PatternField {
                name: "sd".into(),
                rust_type: "T".into(),
                json_type: "n".into(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
            PatternField {
                name: "zscore".into(),
                rust_type: "T".into(),
                json_type: "n".into(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
        ];
        let all_time = InstanceAnalysis {
            base: "realized_price".into(),
            field_parts: [
                ("p1sd".into(), "p1sd".into()),
                ("sd".into(), "ratio_sd".into()),
                ("zscore".into(), "ratio_zscore".into()),
            ]
            .into_iter()
            .collect(),
            is_suffix_mode: true,
            has_outlier: false,
        };
        let four_year = InstanceAnalysis {
            base: "realized_price".into(),
            field_parts: [
                ("p1sd".into(), "p1sd_4y".into()),
                ("sd".into(), "ratio_sd_4y".into()),
                ("zscore".into(), "ratio_zscore_4y".into()),
            ]
            .into_iter()
            .collect(),
            is_suffix_mode: true,
            has_outlier: false,
        };
        let mode = determine_pattern_mode(&[all_time, four_year], &fields);
        assert!(mode.is_some());
        match mode.unwrap() {
            PatternMode::Templated { templates } => {
                assert_eq!(templates.get("p1sd").unwrap(), "p1sd{disc}");
                assert_eq!(templates.get("sd").unwrap(), "ratio_sd{disc}");
                assert_eq!(templates.get("zscore").unwrap(), "ratio_zscore{disc}");
            }
            other => panic!("Expected Templated, got {:?}", other),
        }
    }

    #[test]
    fn test_suffix_disc_with_empty_fields() {
        use std::collections::BTreeSet;
        let fields = vec![
            PatternField {
                name: "band".into(),
                rust_type: "T".into(),
                json_type: "n".into(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
            PatternField {
                name: "sd".into(),
                rust_type: "T".into(),
                json_type: "n".into(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
        ];
        let all_time = InstanceAnalysis {
            base: "price".into(),
            field_parts: [("band".into(), "".into()), ("sd".into(), "ratio_sd".into())]
                .into_iter()
                .collect(),
            is_suffix_mode: true,
            has_outlier: false,
        };
        let four_year = InstanceAnalysis {
            base: "price".into(),
            field_parts: [
                ("band".into(), "".into()),
                ("sd".into(), "ratio_sd_4y".into()),
            ]
            .into_iter()
            .collect(),
            is_suffix_mode: true,
            has_outlier: false,
        };
        let mode = determine_pattern_mode(&[all_time, four_year], &fields);
        assert!(mode.is_some());
        match mode.unwrap() {
            PatternMode::Templated { templates } => {
                assert_eq!(templates.get("band").unwrap(), "");
                assert_eq!(templates.get("sd").unwrap(), "ratio_sd{disc}");
            }
            other => panic!("Expected Templated, got {:?}", other),
        }
    }

    #[test]
    fn test_suffix_disc_empty_to_nonempty() {
        use std::collections::BTreeSet;
        let fields = vec![
            PatternField {
                name: "all".into(),
                rust_type: "T".into(),
                json_type: "n".into(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
            PatternField {
                name: "sth".into(),
                rust_type: "T".into(),
                json_type: "n".into(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
        ];
        let regular = InstanceAnalysis {
            base: "supply".into(),
            field_parts: [("all".into(), "".into()), ("sth".into(), "sth_".into())]
                .into_iter()
                .collect(),
            is_suffix_mode: true,
            has_outlier: false,
        };
        let profitability = InstanceAnalysis {
            base: "utxos_in_profit".into(),
            field_parts: [
                ("all".into(), "supply".into()),
                ("sth".into(), "sth_supply".into()),
            ]
            .into_iter()
            .collect(),
            is_suffix_mode: true,
            has_outlier: false,
        };
        let mode = determine_pattern_mode(&[regular, profitability], &fields);
        assert!(mode.is_some());
        match mode.unwrap() {
            PatternMode::Templated { templates } => {
                assert_eq!(templates.get("all").unwrap(), "{disc}");
                assert_eq!(templates.get("sth").unwrap(), "sth_{disc}");
            }
            other => panic!("Expected Templated, got {:?}", other),
        }
    }

    #[test]
    fn test_outlier_rejects_pattern() {
        use std::collections::BTreeSet;
        let fields = vec![
            PatternField {
                name: "ratio".into(),
                rust_type: "T".into(),
                json_type: "n".into(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
            PatternField {
                name: "value".into(),
                rust_type: "T".into(),
                json_type: "n".into(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
        ];
        // SOPR case: one instance has outlier naming (no common prefix)
        let normal = InstanceAnalysis {
            base: "series".into(),
            field_parts: [
                ("ratio".into(), "ratio".into()),
                ("value".into(), "value".into()),
            ]
            .into_iter()
            .collect(),
            is_suffix_mode: true,
            has_outlier: false,
        };
        let outlier = InstanceAnalysis {
            base: "".into(),
            field_parts: [
                ("ratio".into(), "asopr".into()),
                ("value".into(), "adj_value".into()),
            ]
            .into_iter()
            .collect(),
            is_suffix_mode: true,
            has_outlier: true,
        };
        let mode = determine_pattern_mode(&[normal, outlier], &fields);
        assert!(
            mode.is_some(),
            "Outlier should be filtered out, leaving a valid pattern from non-outlier instances"
        );
    }

    #[test]
    fn test_unanimity_rejects_disagreeing_instances() {
        use std::collections::BTreeSet;
        let fields = vec![
            PatternField {
                name: "a".into(),
                rust_type: "T".into(),
                json_type: "n".into(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
            PatternField {
                name: "b".into(),
                rust_type: "T".into(),
                json_type: "n".into(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
        ];
        let inst1 = InstanceAnalysis {
            base: "x".into(),
            field_parts: [("a".into(), "foo".into()), ("b".into(), "bar".into())]
                .into_iter()
                .collect(),
            is_suffix_mode: true,
            has_outlier: false,
        };
        let inst2 = InstanceAnalysis {
            base: "y".into(),
            field_parts: [("a".into(), "baz".into()), ("b".into(), "qux".into())]
                .into_iter()
                .collect(),
            is_suffix_mode: true,
            has_outlier: false,
        };
        let mode = determine_pattern_mode(&[inst1, inst2], &fields);
        assert!(
            mode.is_none(),
            "Should be non-parameterizable when no pattern detected"
        );
    }

    #[test]
    fn test_all_empty_different_types_uses_identity() {
        // AbsoluteRatePattern: absolute (_1m1w1y24hPattern) and rate (_1m1w1y24hPattern2)
        // have different types. Both return the same base → all-empty field_parts.
        // Should keep identity (empty parts) so both children receive acc unchanged.
        use std::collections::BTreeSet;
        let fields = vec![
            PatternField {
                name: "absolute".into(),
                rust_type: "TypeA".into(),
                json_type: "n".into(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
            PatternField {
                name: "rate".into(),
                rust_type: "TypeB".into(),
                json_type: "n".into(),
                indexes: BTreeSet::new(),
                type_param: None,
            },
        ];
        let inst = InstanceAnalysis {
            base: "supply_delta".into(),
            field_parts: [("absolute".into(), "".into()), ("rate".into(), "".into())]
                .into_iter()
                .collect(),
            is_suffix_mode: true,
            has_outlier: false,
        };
        let mode = determine_pattern_mode(&[inst], &fields);
        assert!(mode.is_some());
        match mode.unwrap() {
            PatternMode::Suffix { relatives } => {
                assert_eq!(
                    relatives.get("absolute"),
                    Some(&"".to_string()),
                    "absolute should be identity"
                );
                assert_eq!(
                    relatives.get("rate"),
                    Some(&"".to_string()),
                    "rate should be identity"
                );
            }
            other => panic!("Expected Suffix with identity, got {:?}", other),
        }
    }

    #[test]
    fn test_all_empty_same_type_marks_outlier() {
        // A wrapper whose children share one type and return the same base
        // produces all-empty field_parts.
        // Should be marked as outlier so the tree inlines instead of using a
        // factory that can't differentiate the children.
        let mut child_bases = BTreeMap::new();
        child_bases.insert("all".to_string(), "realized_price".to_string());
        child_bases.insert("_4y".to_string(), "realized_price".to_string());
        child_bases.insert("_2y".to_string(), "realized_price".to_string());
        child_bases.insert("_1y".to_string(), "realized_price".to_string());

        let analysis = analyze_instance(&child_bases);

        assert_eq!(analysis.base, "realized_price");
        assert!(
            analysis.field_parts.values().all(|v| v.is_empty()),
            "All field_parts should be empty when children return same base"
        );
        // Note: has_outlier is set by collect_instance_analyses based on
        // all_same_type check, not by analyze_instance directly.
        // The test for outlier detection is via determine_pattern_mode
        // with has_outlier flag set.
    }

    #[test]
    fn test_non_parameterizable_cascade() {
        // When a pattern has outlier instances, determine_pattern_mode returns None.
        // Parent patterns containing non-parameterizable children should also
        // be detected via metadata.is_parameterizable (recursive check).
        use std::collections::BTreeSet;
        let fields = vec![PatternField {
            name: "a".into(),
            rust_type: "T".into(),
            json_type: "n".into(),
            indexes: BTreeSet::new(),
            type_param: None,
        }];
        let inst = InstanceAnalysis {
            base: "".into(),
            field_parts: [("a".into(), "standalone_name".into())]
                .into_iter()
                .collect(),
            is_suffix_mode: true,
            has_outlier: true,
        };
        let mode = determine_pattern_mode(&[inst], &fields);
        assert!(
            mode.is_none(),
            "Pattern with outlier should be non-parameterizable"
        );
    }

    #[test]
    fn test_extract_disc_from_instance() {
        // A discriminated instance has field_parts such as "0sd_4y", "p1sd_4y",
        // and "ratio_sd_4y".
        // Templates are "0sd{disc}", "p1sd{disc}", "ratio_sd{disc}".
        // The extracted disc should be "_4y", not "0sd_4y" (the shortest field_part).
        use crate::StructuralPattern;
        use std::collections::BTreeSet;

        let pattern = StructuralPattern {
            name: "TestPattern".into(),
            fields: vec![
                PatternField {
                    name: "_0sd".into(),
                    rust_type: "T".into(),
                    json_type: "n".into(),
                    indexes: BTreeSet::new(),
                    type_param: None,
                },
                PatternField {
                    name: "p1sd".into(),
                    rust_type: "T".into(),
                    json_type: "n".into(),
                    indexes: BTreeSet::new(),
                    type_param: None,
                },
                PatternField {
                    name: "sd".into(),
                    rust_type: "T".into(),
                    json_type: "n".into(),
                    indexes: BTreeSet::new(),
                    type_param: None,
                },
            ],
            mode: Some(PatternMode::Templated {
                templates: [
                    ("_0sd".into(), "0sd{disc}".into()),
                    ("p1sd".into(), "p1sd{disc}".into()),
                    ("sd".into(), "ratio_sd{disc}".into()),
                ]
                .into_iter()
                .collect(),
            }),
            is_generic: false,
        };

        // 4y instance
        let field_parts_4y: BTreeMap<String, String> = [
            ("_0sd".into(), "0sd_4y".into()),
            ("p1sd".into(), "p1sd_4y".into()),
            ("sd".into(), "ratio_sd_4y".into()),
        ]
        .into_iter()
        .collect();

        let disc = pattern.extract_disc_from_instance(&field_parts_4y);
        assert_eq!(disc, Some("4y".to_string()));

        // All-time instance (no period suffix)
        let field_parts_all: BTreeMap<String, String> = [
            ("_0sd".into(), "0sd".into()),
            ("p1sd".into(), "p1sd".into()),
            ("sd".into(), "ratio_sd".into()),
        ]
        .into_iter()
        .collect();

        let disc = pattern.extract_disc_from_instance(&field_parts_all);
        assert_eq!(disc, Some(String::new()));
    }

    #[test]
    fn test_mixed_empty_fills_with_longer_suffix() {
        // CapLossMvrvNetPriceProfitSoprPattern: "loss" field is empty but its
        // shortest leaf is "realized_loss" which contains "loss" and is longer.
        // Should fill with "realized_loss". But "supply" field whose suffix equals
        // the field name exactly should NOT be filled (identity).
        let mut child_bases = BTreeMap::new();
        child_bases.insert("cap".to_string(), "utxos_realized_cap".to_string());
        child_bases.insert("loss".to_string(), "utxos".to_string()); // returns parent base
        child_bases.insert("mvrv".to_string(), "utxos_mvrv".to_string());
        child_bases.insert("price".to_string(), "utxos_realized_price".to_string());
        child_bases.insert("supply".to_string(), "utxos".to_string()); // returns parent base

        let analysis = analyze_instance(&child_bases);
        assert_eq!(analysis.base, "utxos");

        // loss and supply should be empty from common prefix analysis
        assert_eq!(analysis.field_parts.get("loss"), Some(&"".to_string()));
        assert_eq!(analysis.field_parts.get("supply"), Some(&"".to_string()));
        // others should be non-empty
        assert_eq!(
            analysis.field_parts.get("cap"),
            Some(&"realized_cap".to_string())
        );
        assert_eq!(analysis.field_parts.get("mvrv"), Some(&"mvrv".to_string()));
        assert_eq!(
            analysis.field_parts.get("price"),
            Some(&"realized_price".to_string())
        );
    }

    #[test]
    fn test_loss_with_neg_suffix_has_correct_field_parts() {
        // Integration test: "loss" child has suffix-named children (realized_loss,
        // realized_loss_neg) so it returns a proper base that differs from parent.
        use bitview_types::{SeriesLeaf, SeriesLeafWithSchema, TreeNode};

        fn leaf(name: &str) -> TreeNode {
            TreeNode::Leaf(SeriesLeafWithSchema::new(
                SeriesLeaf::new(name.into(), "f32".into(), std::collections::BTreeSet::new()),
                serde_json::Value::Null,
            ))
        }

        let parent = TreeNode::Branch(
            [
                ("cap".into(), leaf("utxos_realized_cap")),
                (
                    "loss".into(),
                    TreeNode::Branch(
                        [
                            ("base".into(), leaf("utxos_realized_loss")),
                            ("negative".into(), leaf("utxos_realized_loss_neg")),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                ),
                ("mvrv".into(), leaf("utxos_mvrv")),
            ]
            .into_iter()
            .collect(),
        );

        let mut all_analyses = BTreeMap::new();
        let mut node_bases = BTreeMap::new();
        let mut path_to_pattern = BTreeMap::new();
        let pattern_lookup = BTreeMap::new();

        collect_instance_analyses(
            &parent,
            "test",
            &pattern_lookup,
            &mut all_analyses,
            &mut node_bases,
            &mut path_to_pattern,
        );

        let result = node_bases
            .get("test")
            .expect("should have node_bases entry");
        assert_eq!(result.base, "utxos");
        assert!(!result.has_outlier);
        assert_eq!(
            result.field_parts.get("cap"),
            Some(&"realized_cap".to_string())
        );
        assert_eq!(result.field_parts.get("mvrv"), Some(&"mvrv".to_string()));
        // loss branch returns base "utxos_realized_loss" which yields field_part "realized_loss"
        assert_eq!(
            result.field_parts.get("loss"),
            Some(&"realized_loss".to_string())
        );
    }
}
