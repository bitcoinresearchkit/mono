//! JavaScript client generation.
//!
//! This module generates a JavaScript + JSDoc client for the Bitview API.

use std::{fmt::Write, fs, io, path::Path};

use serde_json::{Value, from_str, json, to_string_pretty};

use super::write_if_changed;
use crate::{Endpoint, IndexSetPattern, TypeSchemas, VERSION, model::Model};

mod api;
pub(crate) mod client;
mod tree;
pub(crate) mod types;

/// Generate JavaScript + JSDoc client from the series-tree model and OpenAPI endpoints.
///
/// `output_path` is the full path to the output file (e.g., "modules/bitview-client/index.js").
pub(crate) fn generate_javascript_client(
    model: &Model,
    shape_names: &[String],
    accessors: &[IndexSetPattern],
    endpoints: &[Endpoint],
    schemas: &TypeSchemas,
    output_path: &Path,
) -> io::Result<()> {
    let mut output = String::new();

    writeln!(output, "// Auto-generated Bitview JavaScript client").unwrap();
    writeln!(output, "// Do not edit manually\n").unwrap();

    types::generate_type_definitions(&mut output, schemas);
    client::generate_base_client(&mut output);
    client::generate_index_accessors(&mut output, accessors);
    tree::generate_tree(&mut output, model, shape_names, accessors);
    tree::generate_main_client(&mut output, endpoints);

    write_if_changed(output_path, &output)?;

    // Update package.json version if it exists in the same directory
    if let Some(parent) = output_path.parent() {
        let package_json_path = parent.join("package.json");
        if package_json_path.exists() {
            update_package_json_version(&package_json_path)?;
        }
    }

    Ok(())
}

fn update_package_json_version(package_json_path: &Path) -> io::Result<()> {
    let content = fs::read_to_string(package_json_path)?;
    let mut package: Value =
        from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if let Some(obj) = package.as_object_mut() {
        obj.insert("version".to_string(), json!(VERSION));
    }

    let updated =
        to_string_pretty(&package).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    write_if_changed(package_json_path, &(updated + "\n"))
}
