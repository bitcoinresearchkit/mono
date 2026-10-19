mod api;
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
use crate::{Endpoint, IndexSetPattern, ValueTypes, model::Model};

pub(crate) fn generate_rust_client(
    model: &Model,
    shape_names: &[String],
    accessors: &[IndexSetPattern],
    value_types: &ValueTypes,
    endpoints: &[Endpoint],
    output_path: &Path,
) -> io::Result<()> {
    let mut output = String::new();

    writeln!(output, "// Auto-generated Bitview Rust client").unwrap();
    writeln!(output, "// Do not edit manually\n").unwrap();
    writeln!(output, "#![allow(non_snake_case)]").unwrap();
    writeln!(output, "#![allow(clippy::useless_format)]\n").unwrap();

    client::generate_imports(&mut output);
    client::generate_index_accessors(&mut output, accessors);
    tree::generate_tree(&mut output, model, shape_names, accessors, value_types);
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
