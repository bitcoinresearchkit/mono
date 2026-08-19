//! Comprehensive test that fetches all endpoints in the tree.
//!
//! This example demonstrates how to recursively traverse the metrics catalog tree
//! and fetch data from each endpoint. Run with: cargo run --example tree

use bitview_client::BitviewClient;
use brk_types::{Index, RangeIndex, TreeNode};
use std::collections::BTreeSet;

/// A collected metric with its path and available indexes.
struct CollectedMetric {
    path: String,
    name: String,
    indexes: BTreeSet<Index>,
}

/// Recursively collect all metrics from the tree.
fn collect_metrics(node: &TreeNode, path: &str) -> Vec<CollectedMetric> {
    let mut metrics = Vec::new();

    match node {
        TreeNode::Branch(children) => {
            for (key, child) in children {
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                metrics.extend(collect_metrics(child, &child_path));
            }
        }
        TreeNode::Leaf(leaf) => {
            metrics.push(CollectedMetric {
                path: path.to_string(),
                name: leaf.name().to_string(),
                indexes: leaf.indexes().clone(),
            });
        }
    }

    metrics
}

fn main() -> bitview_client::Result<()> {
    let client = BitviewClient::new("http://localhost:3110");

    // Get the metrics catalog tree
    let tree = client.get_series_tree()?;

    // Recursively collect all metrics
    let metrics = collect_metrics(&tree, "");
    println!("\nFound {} metrics", metrics.len());

    let mut success = 0;

    for metric in &metrics {
        for index in &metric.indexes {
            let index_str = index.name();
            let full_path = format!("{}.by.{}", metric.path, index_str);

            match client.get_series(
                metric.name.as_str().into(),
                *index,
                None,
                Some(RangeIndex::Int(0)),
                None,
                None,
            ) {
                Ok(_) => {
                    success += 1;
                    println!("OK: {}", full_path);
                }
                Err(e) => {
                    println!("FAIL: {} -> {}", full_path, e);
                    return Err(e);
                }
            }
        }
    }

    println!("\n=== Results ===");
    println!("Success: {}", success);

    Ok(())
}
