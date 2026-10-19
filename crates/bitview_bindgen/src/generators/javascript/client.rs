use std::fmt::Write;

use crate::{
    ClientConstants, IndexSetPattern, address_payload_lengths, calendars, camel_case_keys,
    cohort_constants, format_json,
};

/// The fixed runtime (`runtime.js`: base client, endpoints, caching and address hashing) after
/// the server's tables it reads.
pub(crate) fn generate_base_client(output: &mut String) {
    let calendars = calendars();
    let date_indexes = calendars
        .iter()
        .map(|(index, _)| format!("'{index}'"))
        .collect::<Vec<_>>()
        .join("|");
    let calendars: String = calendars
        .iter()
        .map(|(index, (unit, size, phase))| {
            format!("  ['{index}', ['{unit}', {size}, {phase}]],\n")
        })
        .collect();
    let payload_lengths: String = address_payload_lengths()
        .map(|(output_type, len)| format!("  ['{output_type}', {len}],\n"))
        .collect();
    write!(
        output,
        r#"/** @typedef {{{date_indexes}}} DateIndex */

/**
 * Each date-based index's buckets as [unit, size, phase]: bucket i starts i * size - phase units
 * after 2009-01-01 UTC, except that a month calendar's bucket 0 starts at January 2009.
 * @type {{ReadonlyMap<string, readonly ['second' | 'day' | 'month', number, number]>}}
 */
const _CALENDARS = new Map([
{calendars}]);

/**
 * Each address type's payload length in bytes.
 * @type {{ReadonlyMap<string, number>}}
 */
const _ADDRESS_PAYLOAD_LENGTHS = new Map([
{payload_lengths}]);

"#
    )
    .unwrap();
    output.push_str(include_str!("runtime.js"));
}

/// The client's constant fields and date conversion methods.
pub(crate) fn generate_constants(output: &mut String) {
    let constants = ClientConstants::collect();

    writeln!(output, "  VERSION = \"{}\";\n", constants.version).unwrap();
    write_const(output, "INDEXES", &format_json(&constants.indexes));
    write_const(
        output,
        "POOL_ID_TO_POOL_NAME",
        &format_json(&constants.pool_map),
    );
    for (name, value) in cohort_constants() {
        write_const(output, name, &format_json(&camel_case_keys(value)));
    }

    writeln!(
        output,
        r#"  /**
   * The UTC start of bucket `i`; throws for an index without a date calendar.
   * @param {{Index}} index
   * @param {{number}} i
   * @returns {{globalThis.Date}}
   */
  indexToDate(index, i) {{
    return indexToDate(index, i);
  }}

  /**
   * The floor index (latest index whose date is <= the given date); throws for dates before the
   * index's first date.
   * @param {{Index}} index
   * @param {{globalThis.Date}} d
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

fn write_const(output: &mut String, name: &str, json: &str) {
    writeln!(
        output,
        "  {} = /** @type {{const}} */ ({});\n",
        name,
        indent_json_const(json)
    )
    .unwrap();
}

pub(crate) fn generate_index_accessors(output: &mut String, patterns: &[IndexSetPattern]) {
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

    writeln!(
        output,
        r#"/**
 * @template T
 * @param {{BitviewClient}} client
 * @param {{string}} name
 * @param {{readonly Index[]}} indexes
 */
function _seriesPattern(client, name, indexes) {{
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

    for pattern in patterns {
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
