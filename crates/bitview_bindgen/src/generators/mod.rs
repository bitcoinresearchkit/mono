//! Code generators for client libraries.
//!
//! Each client has its own submodule. Language clients use focused files:
//! - `types.rs` - Type definitions
//! - `client.rs` - Base client and pattern factories
//! - `tree.rs` - Tree structure generation
//! - `api.rs` - API method generation
//! - `mod.rs` - Entry point

use std::{fmt::Write, fs, io, path::Path};

pub mod cli;
pub mod javascript;
pub mod llm;
pub mod python;
pub mod rust;

pub use cli::generate_cli;
pub use javascript::generate_javascript_client;
pub use llm::generate_llm_clients;
pub use python::generate_python_client;
pub use rust::generate_rust_client;

/// Types that are manually defined as generics in client code, not from schema.
pub const MANUAL_GENERIC_TYPES: &[&str] = &["SeriesData", "SeriesEndpoint"];

/// Write a multi-line description with the given prefix for each line.
/// `empty_prefix` is used for blank lines (e.g., "   *" without trailing space).
pub fn write_description(output: &mut String, desc: &str, prefix: &str, empty_prefix: &str) {
    for line in desc.lines() {
        if line.is_empty() {
            writeln!(output, "{}", empty_prefix).unwrap();
        } else {
            writeln!(output, "{}{}", prefix, line).unwrap();
        }
    }
}

/// Replace generic types with their Any variants in return types.
/// Used by JS and Python generators.
pub fn normalize_return_type(return_type: &str) -> String {
    let mut result = return_type.to_string();
    for type_name in MANUAL_GENERIC_TYPES {
        result = result.replace(type_name, &format!("Any{}", type_name));
    }
    result
}

/// Write content to a file only if it differs from existing content.
/// Preserves mtime when unchanged, avoiding unnecessary cargo rebuilds.
pub fn write_if_changed(path: &Path, content: &str) -> io::Result<()> {
    if let Ok(existing) = fs::read_to_string(path)
        && existing == content
    {
        return Ok(());
    }
    fs::write(path, content)
}
