// //! Tests that verify pattern analysis using the real catalog.

// use std::collections::{BTreeMap, BTreeSet};
// use std::fmt::Write;

// use bitview_bindgen::ClientMetadata;
// use brk_types::TreeNode;

// /// Load the catalog from the JSON file.
// fn load_catalog() -> TreeNode {
//     let path = concat!(env!("CARGO_MANIFEST_DIR"), "/catalog.json");
//     let catalog_json = std::fs::read_to_string(path).expect("Failed to read catalog.json");
//     serde_json::from_str(&catalog_json).expect("Failed to parse catalog.json")
// }

// /// Load OpenAPI spec from openapi.json.
// fn load_openapi_json() -> String {
//     let path = concat!(env!("CARGO_MANIFEST_DIR"), "/openapi.json");
//     std::fs::read_to_string(path).expect("Failed to read openapi.json")
// }

// /// Load metadata from the catalog.
// #[allow(unused)]
// fn load_metadata() -> ClientMetadata {
//     ClientMetadata::from_catalog(load_catalog())
// }

// /// Collect all leaf metric names from a tree.
// fn collect_leaf_names(node: &TreeNode, names: &mut BTreeSet<String>) {
//     match node {
//         TreeNode::Leaf(leaf) => {
//             names.insert(leaf.name().to_string());
//         }
//         TreeNode::Branch(children) => {
//             for child in children.values() {
//                 collect_leaf_names(child, names);
//             }
//         }
//     }
// }

// #[test]
// fn test_catalog_loads() {
//     let catalog = load_catalog();

//     // Should be a branch with top-level categories
//     let TreeNode::Branch(categories) = &catalog else {
//         panic!("Expected catalog to be a branch");
//     };

//     // Check some expected top-level categories exist
//     assert!(
//         categories.contains_key("addresses"),
//         "Missing addresses category"
//     );
//     assert!(categories.contains_key("blocks"), "Missing blocks category");
//     assert!(categories.contains_key("market"), "Missing market category");
//     assert!(categories.contains_key("supply"), "Missing supply category");

//     println!("Catalog has {} top-level categories", categories.len());
// }

// #[test]
// fn test_all_leaves_have_names() {
//     let catalog = load_catalog();
//     let mut names = BTreeSet::new();
//     collect_leaf_names(&catalog, &mut names);

//     println!("Catalog has {} unique metric names", names.len());
//     assert!(!names.is_empty(), "Should have at least some metrics");

//     // All names should be non-empty
//     for name in &names {
//         assert!(!name.is_empty(), "Found empty metric name");
//     }
// }

// #[test]
// fn test_pattern_detection() {
//     let catalog = load_catalog();

//     let (patterns, concrete_to_pattern, concrete_to_type_param, _node_bases) =
//         bitview_bindgen::detect_structural_patterns(&catalog);

//     println!("Detected {} structural patterns", patterns.len());
//     println!(
//         "Concrete to pattern mappings: {}",
//         concrete_to_pattern.len()
//     );
//     println!("Type parameter mappings: {}", concrete_to_type_param.len());

//     // Print pattern details
//     for pattern in &patterns {
//         let mode_str = match &pattern.mode {
//             Some(bitview_bindgen::PatternMode::Suffix { relatives }) => {
//                 format!("Suffix({})", relatives.len())
//             }
//             Some(bitview_bindgen::PatternMode::Prefix { prefixes }) => {
//                 format!("Prefix({})", prefixes.len())
//             }
//             None => "None".to_string(),
//         };
//         println!(
//             "  {} (fields: {}, generic: {}, mode: {})",
//             pattern.name,
//             pattern.fields.len(),
//             pattern.is_generic,
//             mode_str
//         );
//     }

//     // Should have detected some patterns
//     assert!(!patterns.is_empty(), "Should detect at least some patterns");

//     // Check that parameterizable patterns have valid modes
//     for pattern in &patterns {
//         if pattern.is_parameterizable() {
//             let mode = pattern.mode.as_ref().unwrap();
//             match mode {
//                 bitview_bindgen::PatternMode::Suffix { relatives } => {
//                     assert_eq!(
//                         relatives.len(),
//                         pattern.fields.len(),
//                         "Pattern {} should have relative for each field",
//                         pattern.name
//                     );
//                 }
//                 bitview_bindgen::PatternMode::Prefix { prefixes } => {
//                     assert_eq!(
//                         prefixes.len(),
//                         pattern.fields.len(),
//                         "Pattern {} should have prefix for each field",
//                         pattern.name
//                     );
//                 }
//             }
//         }
//     }
// }

// #[test]
// fn test_cost_basis_pattern() {
//     let catalog = load_catalog();

//     let (patterns, _, _, _) = bitview_bindgen::detect_structural_patterns(&catalog);

//     // Find CostBasisPattern2 and inspect it
//     let cost_basis = patterns
//         .iter()
//         .find(|p| p.name == "CostBasisPattern2")
//         .expect("CostBasisPattern2 should exist");

//     println!("CostBasisPattern2:");
//     println!(
//         "  Fields: {:?}",
//         cost_basis
//             .fields
//             .iter()
//             .map(|f| &f.name)
//             .collect::<Vec<_>>()
//     );
//     println!("  Mode: {:?}", cost_basis.mode);
//     println!("  Is generic: {}", cost_basis.is_generic);

//     // With suffix naming convention (cost_basis_max, cost_basis_min, cost_basis):
//     //
//     // At root level: common prefix is "cost_basis_" -> suffix mode
//     //   max -> "max"
//     //   min -> "min"
//     //   percentiles -> "" (identity)
//     //
//     // At lth_ level: common prefix is "lth_cost_basis_" -> suffix mode
//     //   max -> "max"
//     //   min -> "min"
//     //   percentiles -> "" (identity)
//     //
//     // Both use suffix mode with same relatives, so pattern IS parameterizable!
//     assert!(
//         cost_basis.is_parameterizable(),
//         "CostBasisPattern2 should be parameterizable with consistent suffix mode"
//     );
// }

// #[test]
// fn test_realized_pattern3_fields() {
//     let catalog = load_catalog();
//     let metadata = ClientMetadata::from_catalog(catalog);

//     let pattern = metadata
//         .find_pattern("RealizedPattern3")
//         .expect("RealizedPattern3 should exist");

//     println!("RealizedPattern3 fields:");
//     for field in &pattern.fields {
//         let is_branch = field.is_branch();
//         let is_pattern = metadata.find_pattern(&field.rust_type).is_some();
//         let is_param = metadata.is_parameterizable(&field.rust_type);
//         println!(
//             "  {} -> {} (branch={}, pattern={}, param={})",
//             field.name, field.rust_type, is_branch, is_pattern, is_param
//         );
//     }

//     // Check if RealizedPattern3 is considered parameterizable
//     println!(
//         "\nRealizedPattern3 is_parameterizable (metadata): {}",
//         metadata.is_parameterizable("RealizedPattern3")
//     );
// }

// #[test]
// fn test_parameterizable_patterns_have_mode() {
//     let catalog = load_catalog();
//     let (patterns, _, _, _) = bitview_bindgen::detect_structural_patterns(&catalog);

//     // All patterns that appear 2+ times should either:
//     // 1. Be parameterizable (have a mode)
//     // 2. Or have inconsistent instances (mode = None)
//     //
//     // Patterns with mode = None should be inlined, not generate factories

//     let parameterizable: Vec<_> = patterns.iter().filter(|p| p.is_parameterizable()).collect();
//     let non_parameterizable: Vec<_> = patterns
//         .iter()
//         .filter(|p| !p.is_parameterizable())
//         .collect();

//     println!("\nParameterizable patterns ({}):", parameterizable.len());
//     for p in &parameterizable {
//         let mode = p.mode.as_ref().unwrap();
//         let mode_type = match mode {
//             bitview_bindgen::PatternMode::Suffix { .. } => "Suffix",
//             bitview_bindgen::PatternMode::Prefix { .. } => "Prefix",
//         };
//         println!("  {} ({} fields, {})", p.name, p.fields.len(), mode_type);
//     }

//     println!(
//         "\nNon-parameterizable patterns ({}):",
//         non_parameterizable.len()
//     );
//     for p in &non_parameterizable {
//         println!("  {} ({} fields)", p.name, p.fields.len());
//     }

//     // Verify all parameterizable patterns have valid modes with all fields
//     for pattern in &parameterizable {
//         let mode = pattern.mode.as_ref().unwrap();
//         let field_names: BTreeSet<_> = pattern.fields.iter().map(|f| f.name.clone()).collect();

//         match mode {
//             bitview_bindgen::PatternMode::Suffix { relatives } => {
//                 let mode_fields: BTreeSet<_> = relatives.keys().cloned().collect();
//                 assert_eq!(
//                     field_names, mode_fields,
//                     "Pattern {} suffix mode should have all fields",
//                     pattern.name
//                 );
//             }
//             bitview_bindgen::PatternMode::Prefix { prefixes } => {
//                 let mode_fields: BTreeSet<_> = prefixes.keys().cloned().collect();
//                 assert_eq!(
//                     field_names, mode_fields,
//                     "Pattern {} prefix mode should have all fields",
//                     pattern.name
//                 );
//             }
//         }
//     }
// }

// #[test]
// fn test_fee_rate_pattern_relatives() {
//     let catalog = load_catalog();
//     let (patterns, _, _, _) = bitview_bindgen::detect_structural_patterns(&catalog);

//     let fee_rate_pattern = patterns
//         .iter()
//         .find(|p| p.name == "FeeRatePattern")
//         .expect("FeeRatePattern should exist");

//     println!("FeeRatePattern mode:");
//     if let Some(mode) = &fee_rate_pattern.mode {
//         match mode {
//             bitview_bindgen::PatternMode::Suffix { relatives } => {
//                 println!("  Suffix mode:");
//                 for (field, relative) in relatives {
//                     println!("    {} -> '{}'", field, relative);
//                 }
//             }
//             bitview_bindgen::PatternMode::Prefix { prefixes } => {
//                 println!("  Prefix mode:");
//                 for (field, prefix) in prefixes {
//                     println!("    {} -> '{}'", field, prefix);
//                 }
//             }
//         }
//     } else {
//         println!("  No mode (not parameterizable)");
//     }

//     // Check that relatives are correct - should be "average", "max", etc.
//     // NOT "tx_weight_average", "tx_weight_max", etc.
//     if let Some(bitview_bindgen::PatternMode::Suffix { relatives }) = &fee_rate_pattern.mode {
//         assert_eq!(
//             relatives.get("average"),
//             Some(&"average".to_string()),
//             "average relative should be 'average', not 'tx_weight_average'"
//         );
//     }
// }

// #[test]
// fn test_index_patterns() {
//     let catalog = load_catalog();

//     let index_patterns = bitview_bindgen::detect_index_patterns(&catalog);

//     // println!("Used indexes: {:?}", used_indexes);
//     println!("Index set patterns: {}", index_patterns.len());

//     for pattern in &index_patterns {
//         println!("  {} -> {:?}", pattern.name, pattern.indexes);
//     }

//     // Should have detected some index patterns
//     assert!(!index_patterns.is_empty(), "Should detect index patterns");
// }

// #[test]
// fn test_generated_rust_output() {
//     let catalog = load_catalog();
//     let metadata = ClientMetadata::from_catalog(catalog.clone());

//     // Collect all metric names from the catalog
//     let mut all_metrics = BTreeSet::new();
//     collect_leaf_names(&catalog, &mut all_metrics);

//     // Generate Rust client output
//     let mut rust_output = String::new();
//     bitview_bindgen::rust::client::generate_imports(&mut rust_output);
//     bitview_bindgen::rust::client::generate_base_client(&mut rust_output);
//     bitview_bindgen::rust::client::generate_metric_pattern_trait(&mut rust_output);
//     bitview_bindgen::rust::client::generate_endpoint(&mut rust_output);
//     bitview_bindgen::rust::client::generate_index_accessors(
//         &mut rust_output,
//         &metadata.index_set_patterns,
//     );
//     bitview_bindgen::rust::client::generate_pattern_structs(
//         &mut rust_output,
//         &metadata.structural_patterns,
//         &metadata,
//     );
//     bitview_bindgen::rust::tree::generate_tree(&mut rust_output, &metadata.catalog, &metadata);
//     bitview_bindgen::rust::api::generate_main_client(&mut rust_output, &[]);

//     // Count metrics that appear as direct string literals
//     let mut direct_metrics = 0;
//     for metric in &all_metrics {
//         if rust_output.contains(&format!("\"{}\"", metric)) {
//             direct_metrics += 1;
//         }
//     }

//     println!("\nGenerated Rust output stats:");
//     println!("  Total metrics in catalog: {}", all_metrics.len());
//     println!("  Direct string literals: {}", direct_metrics);
//     println!(
//         "  Via pattern factories: {}",
//         all_metrics.len() - direct_metrics
//     );
//     println!("  Output size: {} bytes", rust_output.len());

//     // Write output to test directory (not actual client)
//     let output_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/output");
//     std::fs::create_dir_all(output_dir).ok();
//     let output_path = format!("{}/rust_client.rs", output_dir);
//     std::fs::write(&output_path, &rust_output).expect("Failed to write client output");
//     println!("  Wrote output to: {}", output_path);

//     // Verify the output contains the key components
//     assert!(rust_output.contains("fn _m("), "Should define _m helper");
//     assert!(
//         rust_output.contains("pub struct MetricsTree"),
//         "Should have MetricsTree"
//     );
//     assert!(
//         rust_output.contains("impl MetricsTree"),
//         "Should have MetricsTree impl"
//     );

//     // Count parameterizable patterns (these use _m for dynamic metric names)
//     // Use metadata.is_parameterizable() for full recursive check
//     let parameterizable_count = metadata
//         .structural_patterns
//         .iter()
//         .filter(|p| metadata.is_parameterizable(&p.name))
//         .count();
//     println!("  Parameterizable patterns: {}", parameterizable_count);

//     // Verify all pattern structs are generated (parameterizable and non)
//     for pattern in &metadata.structural_patterns {
//         assert!(
//             rust_output.contains(&format!("pub struct {}", pattern.name)),
//             "Missing pattern struct: {}",
//             pattern.name
//         );
//     }

//     println!("\nGenerated Rust client is complete!");
// }

// #[test]
// fn test_generated_javascript_output() {
//     let catalog = load_catalog();
//     let metadata = ClientMetadata::from_catalog(catalog.clone());

//     // Collect all metric names from the catalog
//     let mut all_metrics = BTreeSet::new();
//     collect_leaf_names(&catalog, &mut all_metrics);

//     // Load schemas from OpenAPI spec only (catalog schemas require runtime data)
//     let openapi_json = load_openapi_json();
//     let schemas = bitview_bindgen::extract_schemas(&openapi_json);

//     // Generate JavaScript client output
//     let mut js_output = String::new();
//     writeln!(js_output, "// Auto-generated Bitview JavaScript client").unwrap();
//     writeln!(js_output, "// Do not edit manually\n").unwrap();
//     bitview_bindgen::javascript::types::generate_type_definitions(&mut js_output, &schemas);
//     bitview_bindgen::javascript::client::generate_base_client(&mut js_output);
//     bitview_bindgen::javascript::client::generate_index_accessors(
//         &mut js_output,
//         &metadata.index_set_patterns,
//     );
//     bitview_bindgen::javascript::client::generate_structural_patterns(
//         &mut js_output,
//         &metadata.structural_patterns,
//         &metadata,
//     );
//     bitview_bindgen::javascript::tree::generate_tree_typedefs(
//         &mut js_output,
//         &metadata.catalog,
//         &metadata,
//     );
//     bitview_bindgen::javascript::tree::generate_main_client(
//         &mut js_output,
//         &metadata.catalog,
//         &metadata,
//         &[],
//     );

//     // Count metrics that appear as direct string literals
//     let mut direct_metrics = 0;
//     for metric in &all_metrics {
//         if js_output.contains(&format!("'{}'", metric))
//             || js_output.contains(&format!("\"{}\"", metric))
//         {
//             direct_metrics += 1;
//         }
//     }

//     println!("\nGenerated JavaScript output stats:");
//     println!("  Total metrics in catalog: {}", all_metrics.len());
//     println!("  Direct string literals: {}", direct_metrics);
//     println!(
//         "  Via pattern factories: {}",
//         all_metrics.len() - direct_metrics
//     );
//     println!("  Output size: {} bytes", js_output.len());
//     println!("  Output lines: {}", js_output.lines().count());

//     // Write output to test directory (not actual client)
//     let output_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/output");
//     std::fs::create_dir_all(output_dir).ok();
//     let output_path = format!("{}/js_client.js", output_dir);
//     std::fs::write(&output_path, &js_output).expect("Failed to write JS client output");
//     println!("  Wrote output to: {}", output_path);

//     // Verify the output contains key components
//     assert!(js_output.contains("const _m ="), "Should define _m helper");
//     assert!(js_output.contains("const _p ="), "Should define _p helper");
//     assert!(
//         js_output.contains("@typedef {Object} MetricsTree"),
//         "Should have MetricsTree typedef"
//     );
//     assert!(
//         js_output.contains("class BitviewClient"),
//         "Should have BitviewClient class"
//     );

//     // Verify all pattern factories are generated
//     for pattern in &metadata.structural_patterns {
//         assert!(
//             js_output.contains(&format!("function create{}(", pattern.name)),
//             "Missing pattern factory: {}",
//             pattern.name
//         );
//     }

//     println!("\nGenerated JavaScript client is complete!");
// }

// #[test]
// fn test_generated_python_output() {
//     let catalog = load_catalog();
//     let metadata = ClientMetadata::from_catalog(catalog.clone());

//     // Collect all metric names from the catalog
//     let mut all_metrics = BTreeSet::new();
//     collect_leaf_names(&catalog, &mut all_metrics);

//     // Load schemas from OpenAPI spec only (catalog schemas require runtime data)
//     let openapi_json = load_openapi_json();
//     let schemas = bitview_bindgen::extract_schemas(&openapi_json);

//     // Generate Python client output
//     let mut py_output = String::new();
//     writeln!(py_output, "# Auto-generated Bitview Python client").unwrap();
//     writeln!(py_output, "# Do not edit manually\n").unwrap();
//     writeln!(py_output, "from typing import TypeVar, Generic, Any, Optional, List, Literal, TypedDict, Union, Protocol, overload, Iterator, Tuple, TYPE_CHECKING").unwrap();
//     writeln!(py_output, "\nif TYPE_CHECKING:").unwrap();
//     writeln!(py_output, "    import pandas as pd  # type: ignore[import-not-found]").unwrap();
//     writeln!(py_output, "    import polars as pl  # type: ignore[import-not-found]").unwrap();
//     writeln!(
//         py_output,
//         "from http.client import HTTPSConnection, HTTPConnection"
//     )
//     .unwrap();
//     writeln!(py_output, "from urllib.parse import urlparse").unwrap();
//     writeln!(py_output, "from datetime import date, timedelta").unwrap();
//     writeln!(py_output, "from dataclasses import dataclass").unwrap();
//     writeln!(py_output, "import json\n").unwrap();
//     writeln!(py_output, "T = TypeVar('T')\n").unwrap();

//     bitview_bindgen::python::types::generate_type_definitions(&mut py_output, &schemas);
//     bitview_bindgen::python::client::generate_base_client(&mut py_output);
//     bitview_bindgen::python::client::generate_endpoint_class(&mut py_output);
//     bitview_bindgen::python::client::generate_index_accessors(
//         &mut py_output,
//         &metadata.index_set_patterns,
//     );
//     bitview_bindgen::python::client::generate_structural_patterns(
//         &mut py_output,
//         &metadata.structural_patterns,
//         &metadata,
//     );
//     bitview_bindgen::python::tree::generate_tree_classes(&mut py_output, &metadata.catalog, &metadata);
//     bitview_bindgen::python::api::generate_main_client(&mut py_output, &[]);

//     // Count metrics that appear as direct string literals
//     let mut direct_metrics = 0;
//     for metric in &all_metrics {
//         if py_output.contains(&format!("'{}'", metric))
//             || py_output.contains(&format!("\"{}\"", metric))
//         {
//             direct_metrics += 1;
//         }
//     }

//     println!("\nGenerated Python output stats:");
//     println!("  Total metrics in catalog: {}", all_metrics.len());
//     println!("  Direct string literals: {}", direct_metrics);
//     println!(
//         "  Via pattern factories: {}",
//         all_metrics.len() - direct_metrics
//     );
//     println!("  Output size: {} bytes", py_output.len());
//     println!("  Output lines: {}", py_output.lines().count());

//     // Write output to test directory (not actual client)
//     let output_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/output");
//     std::fs::create_dir_all(output_dir).ok();
//     let output_path = format!("{}/python_client.py", output_dir);
//     std::fs::write(&output_path, &py_output).expect("Failed to write Python client output");
//     println!("  Wrote output to: {}", output_path);

//     // Verify the output contains key components
//     assert!(py_output.contains("def _m("), "Should define _m helper");
//     assert!(py_output.contains("def _p("), "Should define _p helper");
//     assert!(
//         py_output.contains("class MetricsTree:"),
//         "Should have MetricsTree class"
//     );
//     assert!(
//         py_output.contains("class BitviewClient"),
//         "Should have BitviewClient class"
//     );

//     // Verify all pattern classes have constructors
//     for pattern in &metadata.structural_patterns {
//         assert!(
//             py_output.contains(&format!("class {}:", pattern.name))
//                 || py_output.contains(&format!("class {}(", pattern.name)),
//             "Missing pattern class: {}",
//             pattern.name
//         );
//     }

//     println!("\nGenerated Python client is complete!");
// }

// #[test]
// fn test_cost_basis_relatives() {
//     let catalog = load_catalog();

//     // Find cost_basis branches that have 3 direct children (max, min, percentiles)
//     fn find_cost_basis_with_percentiles(
//         node: &TreeNode,
//         path: &str,
//     ) -> Vec<(String, Vec<(String, String)>)> {
//         let mut results = Vec::new();
//         if let TreeNode::Branch(children) = node {
//             for (name, child) in children {
//                 let child_path = if path.is_empty() {
//                     name.clone()
//                 } else {
//                     format!("{}.{}", path, name)
//                 };

//                 if name == "cost_basis"
//                     && let TreeNode::Branch(cb_children) = child
//                     && cb_children.contains_key("percentiles")
//                 {
//                     // Found a cost_basis with percentiles
//                     let mut metrics = Vec::new();
//                     for (field_name, field_node) in cb_children {
//                         match field_node {
//                             TreeNode::Leaf(leaf) => {
//                                 metrics.push((field_name.clone(), leaf.name().to_string()));
//                             }
//                             TreeNode::Branch(pct_children) => {
//                                 // Get first percentile as example
//                                 if let Some((_, TreeNode::Leaf(first))) = pct_children.iter().next()
//                                 {
//                                     metrics.push((
//                                         format!("{}.first", field_name),
//                                         first.name().to_string(),
//                                     ));
//                                 }
//                             }
//                         }
//                     }
//                     results.push((child_path.clone(), metrics));
//                 }
//                 results.extend(find_cost_basis_with_percentiles(child, &child_path));
//             }
//         }
//         results
//     }

//     let instances = find_cost_basis_with_percentiles(&catalog, "");

//     println!("\nCostBasisPattern2 instances (with percentiles):");
//     for (path, metrics) in instances.iter().take(10) {
//         println!("  {}:", path);
//         for (field, metric) in metrics {
//             println!("    {} -> {}", field, metric);
//         }
//     }

//     // Now compute what relatives the pattern detection would see
//     // The key is: percentiles returns its BASE (common prefix of pct05, pct10, etc.)
//     // not the individual percentile metrics
//     use bitview_bindgen::find_common_prefix;

//     println!("\nComputing relatives (simulating branch base returns):");
//     for (path, metrics) in instances.iter().take(5) {
//         println!("  Instance: {}", path);

//         // For leaves (max, min), the base is the metric name
//         // For branches (percentiles), the base is the common prefix of its children
//         let mut child_bases: std::collections::BTreeMap<String, String> =
//             std::collections::BTreeMap::new();
//         for (field, metric) in metrics {
//             if field.starts_with("percentiles.") {
//                 // This is a percentile metric - compute what the percentiles branch would return
//                 // The base is the metric name with the pct suffix stripped
//                 let base = metric
//                     .strip_suffix("_pct05")
//                     .or_else(|| metric.strip_suffix("_pct10"))
//                     .unwrap_or(metric)
//                     .to_string();
//                 child_bases.insert("percentiles".to_string(), base);
//             } else {
//                 child_bases.insert(field.clone(), metric.clone());
//             }
//         }

//         let bases: Vec<&str> = child_bases.values().map(|s| s.as_str()).collect();
//         println!("    Child bases:");
//         for (field, base) in &child_bases {
//             println!("      {} -> {}", field, base);
//         }

//         if let Some(prefix) = find_common_prefix(&bases) {
//             println!("    Common prefix: '{}'", prefix);
//             for (field, base) in &child_bases {
//                 let relative = base.strip_prefix(&prefix).unwrap_or(base);
//                 println!("      {} -> relative '{}'", field, relative);
//             }
//         } else {
//             println!("    No common prefix found!");
//         }
//     }
// }

// #[test]
// fn test_debug_cost_basis_pattern2_mode() {
//     // Debug why CostBasisPattern2 has mode=None
//     let catalog = load_catalog();
//     let metadata = bitview_bindgen::ClientMetadata::from_catalog(catalog.clone());
//     let pattern_lookup = metadata.pattern_lookup();

//     let pattern = metadata
//         .find_pattern("CostBasisPattern2")
//         .expect("CostBasisPattern2 should exist");

//     println!("\nCostBasisPattern2 fields:");
//     for field in &pattern.fields {
//         println!("  {} (type: {})", field.name, field.rust_type);
//     }
//     println!("Mode: {:?}", pattern.mode);

//     // Now debug the instance collection
//     #[derive(Debug, Clone)]
//     struct DebugInstanceAnalysis {
//         base: String,
//         field_parts: std::collections::BTreeMap<String, String>,
//         is_suffix_mode: bool,
//     }

//     fn collect_debug(
//         node: &TreeNode,
//         pattern_lookup: &std::collections::BTreeMap<Vec<bitview_bindgen::PatternField>, String>,
//         all_analyses: &mut std::collections::BTreeMap<String, Vec<DebugInstanceAnalysis>>,
//     ) -> Option<String> {
//         match node {
//             TreeNode::Leaf(leaf) => Some(leaf.name().to_string()),
//             TreeNode::Branch(children) => {
//                 let mut child_bases: std::collections::BTreeMap<String, String> =
//                     std::collections::BTreeMap::new();
//                 for (field_name, child_node) in children {
//                     if let Some(base) = collect_debug(child_node, pattern_lookup, all_analyses) {
//                         child_bases.insert(field_name.clone(), base);
//                     }
//                 }

//                 if child_bases.is_empty() {
//                     return None;
//                 }

//                 // Analyze this instance
//                 let bases: Vec<&str> = child_bases.values().map(|s| s.as_str()).collect();
//                 let (base, field_parts, is_suffix_mode) =
//                     if let Some(common_prefix) = bitview_bindgen::find_common_prefix(&bases) {
//                         let base = common_prefix.trim_end_matches('_').to_string();
//                         let mut parts = std::collections::BTreeMap::new();
//                         for (field_name, child_base) in &child_bases {
//                             let relative = if *child_base == base {
//                                 String::new()
//                             } else {
//                                 child_base
//                                     .strip_prefix(&common_prefix)
//                                     .unwrap_or(child_base)
//                                     .to_string()
//                             };
//                             parts.insert(field_name.clone(), relative);
//                         }
//                         (base, parts, true)
//                     } else {
//                         let base = child_bases.values().next().cloned().unwrap_or_default();
//                         let parts = child_bases
//                             .iter()
//                             .map(|(k, v)| (k.clone(), v.clone()))
//                             .collect();
//                         (base, parts, true)
//                     };

//                 let analysis = DebugInstanceAnalysis {
//                     base: base.clone(),
//                     field_parts,
//                     is_suffix_mode,
//                 };

//                 // Get the pattern name for this node
//                 let fields = bitview_bindgen::get_node_fields(children, pattern_lookup);
//                 if let Some(pattern_name) = pattern_lookup.get(&fields) {
//                     all_analyses
//                         .entry(pattern_name.clone())
//                         .or_default()
//                         .push(analysis);
//                 }

//                 Some(base)
//             }
//         }
//     }

//     let mut all_analyses: BTreeMap<String, Vec<DebugInstanceAnalysis>> = BTreeMap::new();
//     collect_debug(&catalog, &pattern_lookup, &mut all_analyses);

//     if let Some(analyses) = all_analyses.get("CostBasisPattern2") {
//         println!(
//             "\nCollected {} instances of CostBasisPattern2:",
//             analyses.len()
//         );
//         for (i, a) in analyses.iter().enumerate() {
//             println!("  Instance {}:", i);
//             println!("    base: {}", a.base);
//             println!("    is_suffix: {}", a.is_suffix_mode);
//             println!("    field_parts:");
//             for (f, p) in &a.field_parts {
//                 println!("      {} -> '{}'", f, p);
//             }
//         }

//         // Check consistency
//         if analyses.len() >= 2 {
//             let first = &analyses[0];
//             for (i, a) in analyses.iter().enumerate().skip(1) {
//                 if a.is_suffix_mode != first.is_suffix_mode {
//                     println!("  INCONSISTENT: Instance {} has different mode", i);
//                 }
//                 for (field, part) in &a.field_parts {
//                     if first.field_parts.get(field) != Some(part) {
//                         println!(
//                             "  INCONSISTENT: Instance {} field '{}' has part '{}' vs '{}'",
//                             i,
//                             field,
//                             part,
//                             first
//                                 .field_parts
//                                 .get(field)
//                                 .unwrap_or(&"<missing>".to_string())
//                         );
//                     }
//                 }
//             }
//         }
//     } else {
//         println!("\nNo instances collected for CostBasisPattern2!");
//     }
// }

// #[test]
// fn test_root_cost_basis_prefix() {
//     use bitview_bindgen::find_common_prefix;

//     // Root-level cost_basis has:
//     // max -> "max_cost_basis"
//     // min -> "min_cost_basis"
//     // percentiles -> "cost_basis" (base of pct05, pct10, etc.)

//     let bases = vec!["max_cost_basis", "min_cost_basis", "cost_basis"];
//     let prefix = find_common_prefix(&bases);
//     println!("Root cost_basis prefix: {:?}", prefix);

//     // Compare with nested cost_basis
//     let nested_bases = vec![
//         "utxos_at_least_15y_old_max_cost_basis",
//         "utxos_at_least_15y_old_min_cost_basis",
//         "utxos_at_least_15y_old_cost_basis",
//     ];
//     let nested_prefix = find_common_prefix(&nested_bases);
//     println!("Nested cost_basis prefix: {:?}", nested_prefix);
// }

// #[test]
// fn test_utxo_cohorts_all_activity_base() {
//     // Test that distribution.utxo_cohorts.all.activity uses empty base
//     // because its children (coinblocks_destroyed, coindays_destroyed, etc.)
//     // have no common prefix or suffix.
//     let catalog = load_catalog();
//     let metadata = ClientMetadata::from_catalog(catalog.clone());

//     // Generate JavaScript output
//     let mut js_output = String::new();
//     writeln!(js_output, "// Test output").unwrap();
//     bitview_bindgen::javascript::client::generate_base_client(&mut js_output);
//     bitview_bindgen::javascript::client::generate_index_accessors(
//         &mut js_output,
//         &metadata.index_set_patterns,
//     );
//     bitview_bindgen::javascript::client::generate_structural_patterns(
//         &mut js_output,
//         &metadata.structural_patterns,
//         &metadata,
//     );
//     bitview_bindgen::javascript::tree::generate_tree_typedefs(
//         &mut js_output,
//         &metadata.catalog,
//         &metadata,
//     );
//     bitview_bindgen::javascript::tree::generate_main_client(
//         &mut js_output,
//         &metadata.catalog,
//         &metadata,
//         &[],
//     );

//     // The all.activity should use empty base, so metrics don't get duplicated
//     // Look for: activity: createActivityPattern2(this, '')
//     // NOT: activity: createActivityPattern2(this, 'coinblocks_destroyed')
//     assert!(
//         !js_output.contains("createActivityPattern2(this, 'coinblocks_destroyed')"),
//         "all.activity should NOT use 'coinblocks_destroyed' as base (causes duplication)"
//     );

//     // Check that it uses empty string as base
//     assert!(
//         js_output.contains("activity: createActivityPattern2(this, '')"),
//         "all.activity should use empty base"
//     );

//     println!("utxo_cohorts.all.activity base test passed!");
// }
