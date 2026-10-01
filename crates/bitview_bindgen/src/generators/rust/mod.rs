//! Rust client generation.
//!
//! This module generates a Rust client with full type safety for the Bitview API.

pub(crate) mod api;
mod client;
mod tree;
mod types;

use std::{
    fmt::Write as _,
    fs,
    io::{self, Write as _},
    path::Path,
    process::Command,
};

use tempfile::Builder;

use super::write_if_changed;
use crate::{CatalogTree, Endpoint, detect_index_patterns};
use bitview_catalog::TreeNode;

/// Generate a Rust client directly from the source catalog and OpenAPI endpoints.
///
/// `output_path` is the full path to the output file (e.g., "crates/bitview_client/src/generated.rs").
pub fn generate_rust_client(
    catalog: &TreeNode,
    endpoints: &[Endpoint],
    output_path: &Path,
) -> io::Result<()> {
    let mut output = String::new();
    let indexes = detect_index_patterns(catalog);
    let tree = CatalogTree::from_catalog(catalog, &indexes);

    writeln!(output, "// Auto-generated Bitview Rust client").unwrap();
    writeln!(output, "// Do not edit manually\n").unwrap();
    writeln!(output, "#![allow(non_camel_case_types)]").unwrap();
    writeln!(output, "#![allow(non_snake_case)]").unwrap();
    writeln!(output, "#![allow(dead_code)]").unwrap();
    writeln!(output, "#![allow(unused_variables)]").unwrap();
    writeln!(output, "#![allow(clippy::useless_format)]").unwrap();
    writeln!(output, "#![allow(clippy::unnecessary_to_owned)]\n").unwrap();

    client::generate_imports(&mut output);
    client::generate_base_client(&mut output);
    client::generate_series_pattern_trait(&mut output);
    client::generate_endpoint(&mut output);
    client::generate_index_accessors(&mut output, &indexes);
    tree::generate_tree(&mut output, &tree, &indexes);
    api::generate_main_client(&mut output, endpoints);

    let output = format_rust(output)?;
    write_if_changed(output_path, &output)
}

fn format_rust(source: String) -> io::Result<String> {
    let mut file = Builder::new().suffix(".rs").tempfile()?;
    file.write_all(source.as_bytes())?;

    let status = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .arg(file.path())
        .status()?;

    if !status.success() {
        return Err(io::Error::other(
            "rustfmt failed to format generated client",
        ));
    }

    fs::read_to_string(file.path())
}
