use std::fmt::Write;

use crate::{
    ClientConstants, IndexSetPattern, Ty, TypeSchemas, address_payload_lengths, calendars,
    camel_case_keys, format_json, generators::javascript::keep::keep_expr, name_constants,
};

/// The core module's file name, beside `index.js`, which imports it.
pub(crate) const CORE_FILE: &str = "core.js";

/// The client's content stamp as written until both files' whole text is, its value empty (see `generate_javascript_client`):
/// matched with its comment, whose `*/` no description can hold.
pub(crate) const HASH_STAMP: &str = "  /** The client's content stamp: the SHA-256 of `core.js`, then this file with this value empty, so any other byte of either changes it. */\n  static HASH = \"\";\n";

/// The client's core module (`core.js`): the fixed runtime (`runtime.js`: base client, pinning,
/// endpoints, caching and address hashing) after the server's tables it reads. A page that only
/// makes requests imports it alone; `index.js` imports it under the generated API.
pub(crate) fn generate_core(output: &mut String, schemas: &TypeSchemas) {
    let series_keep = keep_expr(&Ty::Ref("SeriesData".to_owned()), schemas)
        .unwrap_or_else(|| "undefined".to_owned());
    let calendars: String = calendars()
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
        r#"// Auto-generated Bitview JavaScript client core
// Do not edit manually

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

/** Where a series response keeps its keys as sent (`_camelKeys`'s `keep`): its values. */
const _SERIES_KEEP = {series_keep};

"#
    )
    .unwrap();
    output.push_str(include_str!("runtime.js"));
}

/// The runtime's types, beside the generated ones so `index.js` exports them all (`core.js`
/// imports what it uses), then the core's names the generated API uses: every name it exports.
pub(crate) fn generate_core_imports(output: &mut String) {
    let date_indexes = calendars()
        .iter()
        .map(|(index, _)| format!("'{index}'"))
        .collect::<Vec<_>>()
        .join("|");
    writeln!(output, "/** @typedef {{{date_indexes}}} DateIndex */\n").unwrap();
    output.push_str(include_str!("runtime_types.js"));
    let runtime = include_str!("runtime.js");
    let exports = runtime
        .rsplit_once("\nexport { ")
        .and_then(|(_, list)| list.strip_suffix(" };\n"))
        .expect("runtime.js ends with its one `export { ... };` line");
    writeln!(output, "\nimport {{ {exports} }} from './{CORE_FILE}';\n").unwrap();
}

/// The client's constant fields and date conversion methods.
pub(crate) fn generate_constants(output: &mut String) {
    let constants = ClientConstants::collect();

    writeln!(output, "  VERSION = \"{}\";\n", constants.version).unwrap();
    writeln!(output, "{HASH_STAMP}").unwrap();
    write_const(output, "INDEXES", &format_json(&constants.indexes));
    write_const(
        output,
        "POOL_SLUG_TO_NAME",
        &format_json(&constants.pool_map),
    );
    for (name, value) in name_constants() {
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
  return {{
    name,
    /** Each index's endpoint: made when first read, as the tree's nodes are (its thousands of series are walked, few read). */
    get by() {{
      const by = {{}};
      for (const idx of indexes) {{
        Object.defineProperty(by, idx, {{
          get() {{ return _endpoint(client, name, idx); }},
          enumerable: true,
          configurable: true
        }});
      }}
      return _lazy(this, 'by', () => by);
    }},
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
