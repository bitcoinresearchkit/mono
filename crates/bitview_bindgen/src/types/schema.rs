use std::collections::BTreeSet;

use serde_json::Value;

/// Unwrap allOf with a single element, returning the inner schema: schemars can wrap a single
/// `$ref` in `allOf`.
pub(crate) fn unwrap_allof(schema: &Value) -> &Value {
    if let Some(all_of) = schema.get("allOf").and_then(|v| v.as_array())
        && all_of.len() == 1
    {
        return &all_of[0];
    }
    schema
}

/// The type inside a wrapper generic: `Close<Dollars>` -> `Dollars`.
pub(crate) fn inner_type(type_str: &str) -> &str {
    if let Some(start) = type_str.find('<')
        && let Some(end) = type_str.rfind('>')
        && start < end
    {
        return &type_str[start + 1..end];
    }
    type_str
}

/// A leaf's value type in a JavaScript or Python client: wrappers flattened to their innermost
/// type (`Close<Cents>` -> `Cents`), and Rust arrays through `list` (`[Cents; 19]` -> `Cents[]`).
pub(crate) fn client_value_type(kind: &str, list: fn(&str) -> String) -> String {
    if let Some(element) = rust_array_element_type(kind) {
        list(&client_value_type(element, list))
    } else if inner_type(kind) != kind {
        client_value_type(inner_type(kind), list)
    } else {
        kind.to_owned()
    }
}

/// The type name of a JSON Schema `$ref`: `#/definitions/MyType` -> `MyType`.
pub(crate) fn ref_to_type_name(ref_path: &str) -> &str {
    ref_path.rsplit_once('/').map_or(ref_path, |(_, name)| name)
}

/// Every type name a schema references through `$ref`, at any depth.
pub(crate) fn collect_schema_refs<'a>(schema: &'a Value, refs: &mut BTreeSet<&'a str>) {
    match schema {
        Value::Object(map) => {
            if let Some(type_name) = map
                .get("$ref")
                .and_then(Value::as_str)
                .map(ref_to_type_name)
            {
                refs.insert(type_name);
            }
            for value in map.values() {
                collect_schema_refs(value, refs);
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_schema_refs(value, refs);
            }
        }
        _ => {}
    }
}

/// Extract the element type from a Rust fixed-array name such as `[Cents; 19]`.
pub(crate) fn rust_array_element_type(type_name: &str) -> Option<&str> {
    let inner = type_name.strip_prefix('[')?.strip_suffix(']')?;
    let (element, length) = inner.rsplit_once(';')?;
    length.trim().parse::<usize>().ok()?;

    let element = element.trim();
    (!element.is_empty()).then_some(element)
}

/// Whether a schema name is a concrete Rust generic such as `Range<Dollars>`.
pub(crate) fn is_rust_concrete_generic(type_name: &str) -> bool {
    type_name.contains('<') && type_name.ends_with('>')
}
