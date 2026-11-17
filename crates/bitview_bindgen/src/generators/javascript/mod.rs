use std::{collections::BTreeSet, fmt::Write, fs, io, path::Path};

use serde_json::{Value, from_str, json, to_string_pretty};

use super::{sha256_hex, write_if_changed};
use crate::{Endpoint, IndexSetPattern, TypeSchemas, VERSION, model::Model};

pub(crate) mod api;
pub(crate) mod client;
pub(crate) mod keep;
mod tree;
pub(crate) mod types;

pub(crate) fn generate_javascript_client(
    model: &Model,
    shape_names: &[String],
    accessors: &[IndexSetPattern],
    undefined_types: &BTreeSet<String>,
    endpoints: &[Endpoint],
    schemas: &TypeSchemas,
    output_path: &Path,
) -> io::Result<()> {
    let mut output = String::new();

    writeln!(output, "// Auto-generated Bitview JavaScript client").unwrap();
    writeln!(output, "// Do not edit manually\n").unwrap();

    let mut core = String::new();
    client::generate_core(&mut core, schemas);

    types::generate_type_definitions(&mut output, schemas);
    client::generate_core_imports(&mut output);
    client::generate_index_accessors(&mut output, accessors);
    tree::generate_tree(&mut output, model, shape_names, accessors, undefined_types);
    tree::generate_main_client(&mut output, endpoints, schemas);

    // (The stamp, its value written in last: the hash of the text with it empty, so whatever else changes, it does.)
    if output.matches(client::HASH_STAMP).count() != 1 {
        return Err(io::Error::other(
            "the JavaScript client's HASH stamp isn't written exactly once",
        ));
    }
    let hash = sha256_hex(format!("{core}{output}").as_bytes());
    let stamp = client::HASH_STAMP.replacen("\"\"", &format!("\"{hash}\""), 1);
    let output = output.replacen(client::HASH_STAMP, &stamp, 1);
    write_if_changed(&output_path.with_file_name(client::CORE_FILE), &core)?;
    write_if_changed(output_path, &output)?;

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
