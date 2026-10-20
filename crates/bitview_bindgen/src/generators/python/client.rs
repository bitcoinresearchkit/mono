use std::fmt::Write;

use crate::{
    ClientConstants, IndexSetPattern, address_payload_lengths, calendars, format_json,
    index_to_field_name, name_constants,
};

pub(crate) fn generate_class_constants(output: &mut String) {
    let constants = ClientConstants::collect();

    writeln!(output, "    VERSION = \"{}\"\n", constants.version).unwrap();
    write_class_const(output, "INDEXES", &format_json(&constants.indexes));
    write_class_const(
        output,
        "POOL_ID_TO_POOL_NAME",
        &format_json(&constants.pool_map),
    );

    // Unlike JavaScript's, Python's name-table keys stay snake_case.
    for (name, value) in name_constants() {
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

/// The fixed runtime (`runtime.py`: base client, series data and endpoint classes) after the
/// server's tables it reads.
pub(crate) fn generate_runtime(output: &mut String) {
    let calendars: String = calendars()
        .into_iter()
        .map(|(index, (unit, size, phase))| {
            format!("    '{index}': ('{unit}', {size}, {phase}),\n")
        })
        .collect();
    let payload_lengths: String = address_payload_lengths()
        .map(|(output_type, len)| format!("    '{output_type}': {len},\n"))
        .collect();
    write!(
        output,
        "# Each date-based index's buckets as (unit, size, phase): bucket i starts i * size - phase units
# after 2009-01-01 UTC, except that a month calendar's bucket 0 starts at January 2009.
_CALENDARS: Dict[str, Tuple[str, int, int]] = {{
{calendars}}}

# Each address type's payload length in bytes.
_ADDRESS_PAYLOAD_LENGTHS: Dict[str, int] = {{
{payload_lengths}}}


"
    )
    .unwrap();
    output.push_str(include_str!("runtime.py"));
}

pub(crate) fn generate_index_accessors(output: &mut String, patterns: &[IndexSetPattern]) {
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

    writeln!(
        output,
        r#"def _endpoint(c: BitviewClient, n: str, i: Index) -> Union[SeriesEndpoint[Any], DateSeriesEndpoint[Any]]:
    """The endpoint for any index: with date helpers for a date-based one."""
    return DateSeriesEndpoint(c, n, i) if i in _CALENDARS else SeriesEndpoint(c, n, i)
"#
    )
    .unwrap();

    for (i, pattern) in patterns.iter().enumerate() {
        let by_class_name = format!("_{}By", pattern.name);
        let idx_var = format!("_i{}", i + 1);

        writeln!(output, "class {}(Generic[T]):", by_class_name).unwrap();
        writeln!(
            output,
            "    def __init__(self, c: BitviewClient, n: str): self._c, self._n = c, n"
        )
        .unwrap();
        for index in &pattern.indexes {
            let method_name = index_to_field_name(index);
            let index_name = index.name();
            let builder_type = if index.is_date_based() {
                "DateSeriesEndpoint"
            } else {
                "SeriesEndpoint"
            };
            let value = if pattern.missing.contains(index) {
                "Optional[T]"
            } else {
                "T"
            };
            writeln!(
                output,
                "    def {}(self) -> {}[{}]: return {}(self._c, self._n, '{}')",
                method_name, builder_type, value, builder_type, index_name
            )
            .unwrap();
        }
        writeln!(output).unwrap();

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
