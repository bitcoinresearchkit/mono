use serde_json::Value;

/// Unwrap allOf with a single element, returning the inner schema.
/// Schemars uses allOf for composition, but often with just one $ref.
pub(crate) fn unwrap_allof(schema: &Value) -> &Value {
    if let Some(all_of) = schema.get("allOf").and_then(|v| v.as_array())
        && all_of.len() == 1
    {
        return &all_of[0];
    }
    schema
}

/// Extract inner type from a wrapper generic like `Close<Dollars>` -> `Dollars`.
/// Also handles malformed types like `Dollars>` (from vecdb's short_type_name).
pub(crate) fn extract_inner_type(type_str: &str) -> String {
    inner_type(type_str).to_owned()
}

/// Borrow the same normalized inner type when the caller does not need ownership.
fn inner_type(type_str: &str) -> &str {
    // Handle proper generic wrappers like `Close<Dollars>` -> `Dollars`
    if let Some(start) = type_str.find('<')
        && let Some(end) = type_str.rfind('>')
        && start < end
    {
        return &type_str[start + 1..end];
    }
    // Handle malformed types like `Dollars>` (trailing > without <)
    if type_str.ends_with('>') && !type_str.contains('<') {
        return type_str.trim_end_matches('>');
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

/// Extract type name from a JSON Schema $ref path.
/// E.g., "#/definitions/MyType" -> "MyType", "#/$defs/Foo" -> "Foo"
pub(crate) fn ref_to_type_name(ref_path: &str) -> Option<&str> {
    ref_path.rsplit('/').next()
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

/// Get union variants from anyOf or oneOf schema.
pub(crate) fn get_union_variants(schema: &Value) -> Option<&Vec<Value>> {
    schema
        .get("anyOf")
        .or_else(|| schema.get("oneOf"))
        .and_then(|v| v.as_array())
}
