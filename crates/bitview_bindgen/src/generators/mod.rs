use std::{fmt::Write, fs, io, path::Path};

mod cli;
pub(crate) mod javascript;
mod llm;
pub(crate) mod python;
pub(crate) mod reading_rules;
pub(crate) mod rust;

pub(crate) use cli::generate_cli;
pub(crate) use javascript::generate_javascript_client;
pub(crate) use llm::generate_llm_clients;
pub(crate) use python::generate_python_client;
pub(crate) use rust::generate_rust_client;

/// Types that are manually defined as generics in client code, not from schema.
pub(crate) const MANUAL_GENERIC_TYPES: &[&str] = &["SeriesData", "SeriesEndpoint"];

/// Write a multi-line description with the given prefix for each line.
/// `empty_prefix` is used for blank lines (e.g., "   *" without trailing space).
pub(crate) fn write_description(output: &mut String, desc: &str, prefix: &str, empty_prefix: &str) {
    for line in desc.lines() {
        if line.is_empty() {
            writeln!(output, "{}", empty_prefix).unwrap();
        } else {
            writeln!(output, "{}{}", prefix, line).unwrap();
        }
    }
}

/// A description on one JSDoc tag's line, its further lines continuing the comment under
/// `prefix` (its ` *` column): a bare line would sit outside the comment's markup.
pub(crate) fn jsdoc_continued(desc: &str, prefix: &str) -> String {
    desc.lines()
        .collect::<Vec<_>>()
        .join(&format!("\n{prefix}   "))
}

/// Replace generic types with their Any variants in return types.
/// Used by JS and Python generators.
pub(crate) fn normalize_return_type(return_type: &str) -> String {
    let mut result = return_type.to_string();
    for type_name in MANUAL_GENERIC_TYPES {
        result = result.replace(type_name, &format!("Any{}", type_name));
    }
    result
}

/// Write `content` to `path`, creating its parent directories, unless the file already holds it:
/// an unchanged file keeps its mtime, avoiding needless cargo rebuilds.
pub(crate) fn write_if_changed(path: &Path, content: &str) -> io::Result<()> {
    if let Ok(existing) = fs::read_to_string(path)
        && existing == content
    {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}
