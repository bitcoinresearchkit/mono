use serde_json::Value;

/// Unwrap allOf with a single element, returning the inner schema.
/// Schemars uses allOf for composition, but often with just one $ref.
pub fn unwrap_allof(schema: &Value) -> &Value {
    if let Some(all_of) = schema.get("allOf").and_then(|v| v.as_array())
        && all_of.len() == 1
    {
        return &all_of[0];
    }
    schema
}

/// Extract inner type from a wrapper generic like `Close<Dollars>` -> `Dollars`.
/// Also handles malformed types like `Dollars>` (from vecdb's short_type_name).
pub fn extract_inner_type(type_str: &str) -> String {
    // Handle proper generic wrappers like `Close<Dollars>` -> `Dollars`
    if let Some(start) = type_str.find('<')
        && let Some(end) = type_str.rfind('>')
        && start < end
    {
        return type_str[start + 1..end].to_string();
    }
    // Handle malformed types like `Dollars>` (trailing > without <)
    if type_str.ends_with('>') && !type_str.contains('<') {
        return type_str.trim_end_matches('>').to_string();
    }
    type_str.to_string()
}

/// Extract type name from a JSON Schema $ref path.
/// E.g., "#/definitions/MyType" -> "MyType", "#/$defs/Foo" -> "Foo"
pub fn ref_to_type_name(ref_path: &str) -> Option<&str> {
    ref_path.rsplit('/').next()
}

/// Extract the element type from a Rust fixed-array name such as `[Cents; 19]`.
pub fn rust_array_element_type(type_name: &str) -> Option<&str> {
    let inner = type_name.strip_prefix('[')?.strip_suffix(']')?;
    let (element, length) = inner.rsplit_once(';')?;
    length.trim().parse::<usize>().ok()?;

    let element = element.trim();
    (!element.is_empty()).then_some(element)
}

/// Whether a schema name is a concrete Rust generic such as `Range<Dollars>`.
pub fn is_rust_concrete_generic(type_name: &str) -> bool {
    type_name.contains('<') && type_name.ends_with('>')
}

/// Get union variants from anyOf or oneOf schema.
pub fn get_union_variants(schema: &Value) -> Option<&Vec<Value>> {
    schema
        .get("anyOf")
        .or_else(|| schema.get("oneOf"))
        .and_then(|v| v.as_array())
}
