//! Rust imports and leaf accessors; the fixed runtime lives in `bitview_client`'s own modules.

use std::fmt::Write;

use crate::{IndexSetPattern, index_to_field_name};

/// Generate import statements: the domain crates are re-exported, the runtime is imported.
pub(crate) fn generate_imports(output: &mut String) {
    output.push_str(
        r#"pub use bitview_catalog::*;
pub use bitview_cohort::*;
pub use bitview_primitives::*;
pub use bitview_types::*;
pub use brk_types::*;
use crate::{
    BitviewClientBase, BitviewClientOptions, BitviewError, DateSeriesEndpoint, FormatResponse, Node,
    Result, SeriesEndpoint, SeriesValue, address::validate_address_payload_for_type,
    address_hash_prefix, address_payload_hash_prefix,
    client_base::{parse_text, query_pair},
};
use std::sync::{Arc, OnceLock};

"#,
    );
}

/// Generate index accessor structs.
pub(crate) fn generate_index_accessors(output: &mut String, patterns: &[IndexSetPattern]) {
    if patterns.is_empty() {
        return;
    }

    // Generate static index arrays
    writeln!(output, "// Static index arrays").unwrap();
    for (i, pattern) in patterns.iter().enumerate() {
        write!(output, "const _I{}: &[Index] = &[", i + 1).unwrap();
        for (j, index) in pattern.indexes.iter().enumerate() {
            if j > 0 {
                write!(output, ", ").unwrap();
            }
            write!(output, "Index::{}", index).unwrap();
        }
        writeln!(output, "];").unwrap();
    }
    writeln!(output).unwrap();

    for (i, pattern) in patterns.iter().enumerate() {
        let any = if pattern.missing.is_empty() {
            "T"
        } else {
            "T::Nullable"
        };
        writeln!(
            output,
            "accessor! {{ {0}, {0}By, _I{1}, {any} {{",
            pattern.name,
            i + 1
        )
        .unwrap();
        for index in &pattern.indexes {
            let kind = if index.is_date_based() {
                "date"
            } else {
                "plain"
            };
            let value = if pattern.missing.contains(index) {
                "T::Nullable"
            } else {
                "T"
            };
            writeln!(
                output,
                "    {kind} {}: {index} -> {value},",
                index_to_field_name(index)
            )
            .unwrap();
        }
        writeln!(output, "}} }}").unwrap();
    }
}
