//! Python base client and leaf accessors.

use std::fmt::Write;

use crate::{ClientConstants, IndexSetPattern, cohort_constants, format_json, index_to_field_name};

/// Generate class-level constants for the BitviewClient class.
pub(crate) fn generate_class_constants(output: &mut String) {
    let constants = ClientConstants::collect();

    // VERSION
    writeln!(output, "    VERSION = \"{}\"\n", constants.version).unwrap();

    // INDEXES, POOL_ID_TO_POOL_NAME
    write_class_const(output, "INDEXES", &format_json(&constants.indexes));
    write_class_const(
        output,
        "POOL_ID_TO_POOL_NAME",
        &format_json(&constants.pool_map),
    );

    // Cohort constants (no camelCase conversion for Python)
    for (name, value) in cohort_constants() {
        write_class_const(output, name, &format_json(&value));
    }
}

fn write_class_const(output: &mut String, name: &str, json: &str) {
    for (i, line) in json.lines().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        output.push_str("    ");
        if i == 0 {
            write!(output, "{name} = ").unwrap();
        }
        output.push_str(line);
    }
    output.push_str("\n\n");
}

/// The fixed runtime: base client, series data and endpoint classes (`runtime.py`).
pub(crate) fn generate_runtime(output: &mut String) {
    output.push_str(include_str!("runtime.py"));
}

/// Generate index accessor classes
pub(crate) fn generate_index_accessors(output: &mut String, patterns: &[IndexSetPattern]) {
    if patterns.is_empty() {
        return;
    }

    // Generate static index tuples
    writeln!(output, "# Static index tuples").unwrap();
    for (i, pattern) in patterns.iter().enumerate() {
        write!(output, "_i{} = (", i + 1).unwrap();
        for (j, index) in pattern.indexes.iter().enumerate() {
            if j > 0 {
                write!(output, ", ").unwrap();
            }
            write!(output, "'{}'", index.name()).unwrap();
        }
        // Single-element tuple needs trailing comma
        if pattern.indexes.len() == 1 {
            write!(output, ",").unwrap();
        }
        writeln!(output, ")").unwrap();
    }
    writeln!(output).unwrap();

    // Generate helper functions
    writeln!(
        output,
        r#"def _ep(c: BitviewClient, n: str, i: Index) -> SeriesEndpoint[Any]:
    return SeriesEndpoint(c, n, i)

def _dep(c: BitviewClient, n: str, i: Index) -> DateSeriesEndpoint[Any]:
    return DateSeriesEndpoint(c, n, i)

def _endpoint(c: BitviewClient, n: str, i: Index) -> Union[SeriesEndpoint[Any], DateSeriesEndpoint[Any]]:
    """The endpoint for any index: with date helpers for a date-based one."""
    return _dep(c, n, i) if i in _DATE_INDEXES else _ep(c, n, i)
"#
    )
    .unwrap();

    writeln!(output, "# Index accessor classes\n").unwrap();

    for (i, pattern) in patterns.iter().enumerate() {
        let by_class_name = format!("_{}By", pattern.name);
        let idx_var = format!("_i{}", i + 1);

        // Generate the By class with compact methods
        writeln!(output, "class {}(Generic[T]):", by_class_name).unwrap();
        writeln!(
            output,
            "    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n"
        )
        .unwrap();
        for index in &pattern.indexes {
            let method_name = index_to_field_name(index);
            let index_name = index.name();
            let (builder_type, helper) = if index.is_date_based() {
                ("DateSeriesEndpoint", "_dep")
            } else {
                ("SeriesEndpoint", "_ep")
            };
            let value = if pattern.missing.contains(index) {
                "Optional[T]"
            } else {
                "T"
            };
            writeln!(
                output,
                "    def {}(self) -> {}[{}]: return {}(self._c, self._n, '{}')",
                method_name, builder_type, value, helper, index_name
            )
            .unwrap();
        }
        writeln!(output).unwrap();

        // Generate the main accessor class
        writeln!(output, "class {}(Generic[T]):", pattern.name).unwrap();
        writeln!(output, "    by: {}[T]", by_class_name).unwrap();
        writeln!(
            output,
            "    def __init__(self, c: BitviewClient, n: str): self._n, self.by = n, {}(c, n)",
            by_class_name
        )
        .unwrap();
        writeln!(output, "    @property").unwrap();
        writeln!(output, "    def name(self) -> str: return self._n").unwrap();
        writeln!(
            output,
            "    def indexes(self) -> List[str]: return list({})",
            idx_var
        )
        .unwrap();
        // A dynamic index may be any of them: the weakest type.
        let any = if pattern.missing.is_empty() {
            "T"
        } else {
            "Optional[T]"
        };
        writeln!(
            output,
            "    def get(self, index: Index) -> Optional[Union[SeriesEndpoint[{0}], DateSeriesEndpoint[{0}]]]: return _endpoint(self.by._c, self._n, index) if index in {1} else None",
            any, idx_var
        )
        .unwrap();
        writeln!(output).unwrap();
    }
}
