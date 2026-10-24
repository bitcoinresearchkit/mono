use std::collections::BTreeMap;

use rmcp::model::Tool;
use serde::Deserialize;
use serde_json::{Map, Number, Value, from_str};

mod http_operation;
mod operation;
mod parameter;
mod pattern;

pub use http_operation::HttpOperation;
pub use operation::Operation;
pub use parameter::{Parameter, ParameterLocation};
pub(crate) use pattern::Pattern;

const MANIFEST_JSON: &str = include_str!("../generated/manifest.json");
const MAX_SCHEMA_DEPTH: usize = 32;
const MAX_PARAMETER_BYTES: usize = 8 * 1024;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    schema_version: u32,
    operations: Vec<Operation>,
}

#[derive(Debug)]
pub struct Catalog {
    tools: Vec<Tool>,
    operations: BTreeMap<String, Operation>,
}

impl Catalog {
    /// The manifest is generated and checked by bitview_bindgen; here it is only indexed.
    pub fn embedded() -> Self {
        let manifest: Manifest =
            from_str(MANIFEST_JSON).expect("generated LLM manifest is valid; run `cargo bindgen`");
        debug_assert_eq!(manifest.schema_version, 1);
        debug_assert!(!manifest.operations.is_empty());

        let tools = manifest
            .operations
            .iter()
            .map(|operation| operation.tool.clone())
            .collect::<Vec<_>>();
        let operations = manifest
            .operations
            .into_iter()
            .map(|operation| (operation.tool.name.to_string(), operation))
            .collect::<BTreeMap<_, _>>();
        debug_assert_eq!(
            operations.len(),
            tools.len(),
            "duplicate generated tool name"
        );
        debug_assert!(tools.is_sorted_by(|left, right| left.name < right.name));

        Self { tools, operations }
    }

    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }

    pub fn operation(&self, name: &str) -> Option<&Operation> {
        self.operations.get(name)
    }
}

impl Operation {
    /// Whether a text body is a number (`Height`): its output schema, or the one it references,
    /// is an integer or number.
    pub fn text_is_number(&self) -> bool {
        let Some(root) = self.tool.output_schema.as_deref() else {
            return false;
        };
        let schema = root
            .get("$ref")
            .and_then(Value::as_str)
            .and_then(|reference| reference.strip_prefix("#/$defs/"))
            .and_then(|name| root.get("$defs")?.get(name)?.as_object())
            .unwrap_or(root);
        matches!(
            schema.get("type").and_then(Value::as_str),
            Some("integer" | "number")
        )
    }

    pub fn validate_arguments(&self, arguments: &Value) -> Result<(), String> {
        let root = self.tool.input_schema.as_ref();
        validate_schema(root, root, arguments, 0)
            .map_err(|error| format!("invalid arguments: {error}"))
    }
}

fn as_schema(schema: &Value) -> Result<&Map<String, Value>, String> {
    schema
        .as_object()
        .ok_or_else(|| "encountered a non-object schema".to_string())
}

fn validate_schema(
    root: &Map<String, Value>,
    object: &Map<String, Value>,
    value: &Value,
    depth: usize,
) -> Result<(), String> {
    if depth > MAX_SCHEMA_DEPTH {
        return Err("value exceeds the maximum schema depth".to_string());
    }

    // bitview_bindgen emits only local `#/$defs/<name>` references.
    if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
        let target = reference
            .strip_prefix("#/$defs/")
            .and_then(|name| root.get("$defs")?.get(name))
            .ok_or_else(|| format!("missing schema reference {reference}"))?;
        validate_schema(root, as_schema(target)?, value, depth + 1)?;
    }

    if let Some(values) = object.get("enum").and_then(Value::as_array)
        && !values.contains(value)
    {
        return Err("value is not in the allowed enum".to_string());
    }

    if let Some(schemas) = object.get("anyOf").and_then(Value::as_array)
        && !schemas
            .iter()
            .any(|nested| validate(root, nested, value, depth + 1))
    {
        return Err("value does not match any allowed schema".to_string());
    }

    if let Some(expected) = object.get("type") {
        let valid = match expected {
            Value::String(expected) => matches_type(expected, value),
            Value::Array(expected) => expected
                .iter()
                .filter_map(Value::as_str)
                .any(|expected| matches_type(expected, value)),
            _ => false,
        };
        if !valid {
            return Err(format!("expected type {expected}"));
        }
    }

    match value {
        Value::String(value) if value.len() > MAX_PARAMETER_BYTES => Err(format!(
            "string exceeds the {MAX_PARAMETER_BYTES}-byte parameter limit"
        )),
        Value::String(value) => validate_string(object, value),
        Value::Number(value) => validate_number(object, value),
        Value::Array(value) => validate_array(root, object, value, depth),
        Value::Object(value) => validate_object(root, object, value, depth),
        Value::Null | Value::Bool(_) => Ok(()),
    }
}

fn validate(root: &Map<String, Value>, schema: &Value, value: &Value, depth: usize) -> bool {
    as_schema(schema).is_ok_and(|schema| validate_schema(root, schema, value, depth).is_ok())
}

fn matches_type(expected: &str, value: &Value) -> bool {
    match expected {
        "null" => value.is_null(),
        "boolean" => value.is_boolean(),
        "object" => value.is_object(),
        "array" => value.is_array(),
        "number" => value.is_number(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "string" => value.is_string(),
        _ => false,
    }
}

/// A schema's whole-number keyword, if set.
fn bound(schema: &Map<String, Value>, keyword: &str) -> Option<u64> {
    schema.get(keyword).and_then(Value::as_u64)
}

fn validate_string(schema: &Map<String, Value>, value: &str) -> Result<(), String> {
    // JSON Schema counts characters, not bytes.
    let length = value.chars().count() as u64;
    if let Some(min) = bound(schema, "minLength")
        && length < min
    {
        return Err(format!("string is shorter than {min} characters"));
    }
    if let Some(max) = bound(schema, "maxLength")
        && length > max
    {
        return Err(format!("string is longer than {max} characters"));
    }
    if let Some(pattern) = schema.get("pattern").and_then(Value::as_str)
        && !Pattern::compile(pattern)?.matches(value)
    {
        return Err(format!("string does not match {pattern}"));
    }
    Ok(())
}

fn validate_number(schema: &Map<String, Value>, value: &Number) -> Result<(), String> {
    let value = value
        .as_f64()
        .ok_or_else(|| "number cannot be represented safely".to_string())?;
    if !value.is_finite() {
        return Err("number must be finite".to_string());
    }
    if let Some(minimum) = schema.get("minimum").and_then(Value::as_f64)
        && value < minimum
    {
        return Err(format!("number is less than {minimum}"));
    }
    if let Some(maximum) = schema.get("maximum").and_then(Value::as_f64)
        && value > maximum
    {
        return Err(format!("number is greater than {maximum}"));
    }
    Ok(())
}

fn validate_array(
    root: &Map<String, Value>,
    schema: &Map<String, Value>,
    values: &[Value],
    depth: usize,
) -> Result<(), String> {
    let count = values.len() as u64;
    if let Some(min) = bound(schema, "minItems")
        && count < min
    {
        return Err(format!("array has fewer than {min} items"));
    }
    if let Some(max) = bound(schema, "maxItems")
        && count > max
    {
        return Err(format!("array has more than {max} items"));
    }
    if let Some(items) = schema.get("items") {
        for value in values {
            validate_schema(root, as_schema(items)?, value, depth + 1)?;
        }
    }
    Ok(())
}

fn validate_object(
    root: &Map<String, Value>,
    schema: &Map<String, Value>,
    values: &Map<String, Value>,
    depth: usize,
) -> Result<(), String> {
    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        for name in required.iter().filter_map(Value::as_str) {
            if !values.contains_key(name) {
                return Err(format!("missing required property {name}"));
            }
        }
    }

    let properties = schema.get("properties").and_then(Value::as_object);
    for (name, value) in values {
        if let Some(property_schema) = properties.and_then(|properties| properties.get(name)) {
            validate_schema(root, as_schema(property_schema)?, value, depth + 1)?;
            continue;
        }
        match schema.get("additionalProperties") {
            Some(Value::Bool(false)) => return Err(format!("unknown property {name}")),
            Some(Value::Object(additional)) => {
                validate_schema(root, additional, value, depth + 1)?;
            }
            _ => {}
        }
    }
    Ok(())
}
