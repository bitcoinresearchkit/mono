//! JavaScript base client and leaf accessors.

use std::fmt::Write;

use crate::{ClientConstants, IndexSetPattern, camel_case_keys, cohort_constants, format_json};

/// The fixed runtime: base client, endpoints, caching and address hashing (`runtime.js`).
pub(crate) fn generate_base_client(output: &mut String) {
    output.push_str(include_str!("runtime.js"));
}

/// Generate static constants for the BitviewClient class.
pub(crate) fn generate_static_constants(output: &mut String) {
    let constants = ClientConstants::collect();

    // VERSION, INDEXES, POOL_ID_TO_POOL_NAME
    writeln!(output, "  VERSION = \"{}\";\n", constants.version).unwrap();
    write_static_const(output, "INDEXES", &format_json(&constants.indexes));
    write_static_const(
        output,
        "POOL_ID_TO_POOL_NAME",
        &format_json(&constants.pool_map),
    );

    // Cohort constants with camelCase keys
    for (name, value) in cohort_constants() {
        write_static_const(output, name, &format_json(&camel_case_keys(value)));
    }

    // Helper methods
    writeln!(
        output,
        r#"  /**
   * Convert an index value to a Date for date-based indexes.
   * @param {{Index}} index - The index type
   * @param {{number}} i - The index value
   * @returns {{globalThis.Date}}
   */
  indexToDate(index, i) {{
    return indexToDate(index, i);
  }}

  /**
   * Convert a Date to an index value for date-based indexes.
   * @param {{Index}} index - The index type
   * @param {{globalThis.Date}} d - The date to convert
   * @returns {{number}}
   */
  dateToIndex(index, d) {{
    return dateToIndex(index, d);
  }}

"#
    )
    .unwrap();
}

fn indent_json_const(json: &str) -> String {
    json.lines()
        .enumerate()
        .map(|(i, line)| {
            if i == 0 {
                line.to_string()
            } else {
                format!("  {}", line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn write_static_const(output: &mut String, name: &str, json: &str) {
    writeln!(
        output,
        "  {} = /** @type {{const}} */ ({});\n",
        name,
        indent_json_const(json)
    )
    .unwrap();
}

/// Generate index accessor factory functions.
pub(crate) fn generate_index_accessors(output: &mut String, patterns: &[IndexSetPattern]) {
    if patterns.is_empty() {
        return;
    }

    writeln!(output, "// Index group constants and factory\n").unwrap();

    // Generate index array constants (e.g., _i1 = ["day1", "height"])
    for (i, pattern) in patterns.iter().enumerate() {
        write!(output, "const _i{} = /** @type {{const}} */ ([", i + 1).unwrap();
        for (j, index) in pattern.indexes.iter().enumerate() {
            if j > 0 {
                write!(output, ", ").unwrap();
            }
            write!(output, "\"{}\"", index.name()).unwrap();
        }
        writeln!(output, "]);").unwrap();
    }
    writeln!(output).unwrap();

    // Generate ONE generic series pattern factory
    writeln!(
        output,
        r#"/**
 * Generic series pattern factory.
 * @template T
 * @param {{BitviewClient}} client
 * @param {{string}} name - The series vec name
 * @param {{readonly Index[]}} indexes - The supported indexes
 */
function _mp(client, name, indexes) {{
  const by = {{}};
  for (const idx of indexes) {{
    Object.defineProperty(by, idx, {{
      get() {{ return _endpoint(client, name, idx); }},
      enumerable: true,
      configurable: true
    }});
  }}
  return {{
    name,
    by,
    /** @returns {{readonly Index[]}} */
    indexes() {{ return indexes; }},
    /** @param {{Index}} index @returns {{SeriesEndpoint<T>|undefined}} */
    get(index) {{ return indexes.includes(index) ? _endpoint(client, name, index) : undefined; }}
  }};
}}
"#
    )
    .unwrap();

    // Typedefs of the leaf accessors
    for pattern in patterns {
        // Generate typedef for type safety
        let by_fields: Vec<String> = pattern
            .indexes
            .iter()
            .map(|idx| {
                let builder = if idx.is_date_based() {
                    "DateSeriesEndpoint"
                } else {
                    "SeriesEndpoint"
                };
                let value = if pattern.missing.contains(idx) {
                    "T | null"
                } else {
                    "T"
                };
                format!("readonly {}: {}<{}>", idx.name(), builder, value)
            })
            .collect();
        let by_type = format!("{{ {} }}", by_fields.join(", "));
        // A dynamic index may be any of them: the weakest type.
        let any = if pattern.missing.is_empty() {
            "T"
        } else {
            "T | null"
        };

        writeln!(
            output,
            "/** @template T @typedef {{{{ name: string, by: {}, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<{}>|undefined }}}} {} */",
            by_type, any, pattern.name
        )
        .unwrap();
    }
    writeln!(output).unwrap();
}
