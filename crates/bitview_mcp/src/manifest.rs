use std::collections::{BTreeMap, BTreeSet};

use rmcp::model::Tool;
use serde::Deserialize;
use serde_json::{Map, Number, Value, from_str};

mod http_operation;
mod operation;
mod parameter;

pub use http_operation::HttpOperation;
pub use operation::Operation;
pub use parameter::{Parameter, ParameterLocation};

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
    pub fn embedded() -> Result<Self, String> {
        Self::parse(MANIFEST_JSON)
    }

    fn parse(source: &str) -> Result<Self, String> {
        let manifest: Manifest =
            from_str(source).map_err(|error| format!("invalid generated LLM manifest: {error}"))?;
        if manifest.schema_version != 1 {
            return Err(format!(
                "unsupported generated LLM manifest version {}",
                manifest.schema_version
            ));
        }
        if manifest.operations.is_empty() {
            return Err(
                "generated LLM manifest is empty; run `cargo run -p bitviewd --bin bitview-bindgen --features bindgen` first".to_string(),
            );
        }

        let mut tools = Vec::with_capacity(manifest.operations.len());
        let mut operations = BTreeMap::new();
        for operation in manifest.operations {
            validate_operation(&operation)?;
            let name = operation.tool.name.to_string();
            tools.push(operation.tool.clone());
            if operations.insert(name.clone(), operation).is_some() {
                return Err(format!("duplicate generated tool name {name}"));
            }
        }
        tools.sort_unstable_by(|left, right| left.name.cmp(&right.name));

        Ok(Self { tools, operations })
    }

    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }

    pub fn operation(&self, name: &str) -> Option<&Operation> {
        self.operations.get(name)
    }
}

impl Operation {
    pub fn validate_arguments(&self, arguments: &Map<String, Value>) -> Result<(), String> {
        let root = Value::Object(self.tool.input_schema.as_ref().clone());
        let value = Value::Object(arguments.clone());
        validate_schema(&root, &root, &value, 0)
            .map_err(|error| format!("invalid arguments: {error}"))
    }
}

fn validate_operation(operation: &Operation) -> Result<(), String> {
    if operation.http.method != "GET" {
        return Err(format!(
            "tool {} is not a read-only GET operation",
            operation.tool.name
        ));
    }
    let path = &operation.http.path;
    if !path.starts_with('/')
        || path.starts_with("//")
        || path.contains('?')
        || path.contains('#')
        || path.split('/').any(|segment| segment == "..")
    {
        return Err(format!(
            "tool {} has an unsafe generated path",
            operation.tool.name
        ));
    }

    let root = operation.tool.input_schema.as_ref();
    if root.get("type").and_then(Value::as_str) != Some("object") {
        return Err(format!(
            "tool {} input schema must have object type",
            operation.tool.name
        ));
    }
    let properties = root
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            format!(
                "tool {} input schema has no properties",
                operation.tool.name
            )
        })?;

    let mut names = BTreeSet::new();
    for parameter in &operation.http.parameters {
        if !names.insert(&parameter.name) {
            return Err(format!(
                "tool {} repeats parameter {}",
                operation.tool.name, parameter.name
            ));
        }
        if !properties.contains_key(&parameter.name) {
            return Err(format!(
                "tool {} maps unknown parameter {}",
                operation.tool.name, parameter.name
            ));
        }
        let placeholder = format!("{{{}}}", parameter.name);
        match parameter.location {
            ParameterLocation::Path if !path.contains(&placeholder) => {
                return Err(format!(
                    "tool {} path omits placeholder {placeholder}",
                    operation.tool.name
                ));
            }
            ParameterLocation::Query if path.contains(&placeholder) => {
                return Err(format!(
                    "tool {} query parameter {} is used in its path",
                    operation.tool.name, parameter.name
                ));
            }
            _ => {}
        }
    }
    if properties.len() != names.len() {
        return Err(format!(
            "tool {} contains an unmapped input property",
            operation.tool.name
        ));
    }
    if path.contains('{') || path.contains('}') {
        for segment in path.split('/') {
            if let Some(name) = segment
                .strip_prefix('{')
                .and_then(|value| value.strip_suffix('}'))
                && !operation.http.parameters.iter().any(|parameter| {
                    parameter.location == ParameterLocation::Path && parameter.name == name
                })
            {
                return Err(format!(
                    "tool {} contains unmapped path placeholder {name}",
                    operation.tool.name
                ));
            }
        }
    }

    validate_schema_definition(
        &Value::Object(operation.tool.input_schema.as_ref().clone()),
        0,
    )
    .map_err(|error| {
        format!(
            "tool {} has invalid input schema: {error}",
            operation.tool.name
        )
    })?;
    if let Some(output_schema) = &operation.tool.output_schema {
        validate_schema_definition(&Value::Object(output_schema.as_ref().clone()), 0).map_err(
            |error| {
                format!(
                    "tool {} has invalid output schema: {error}",
                    operation.tool.name
                )
            },
        )?;
    }
    Ok(())
}

fn validate_schema_definition(schema: &Value, depth: usize) -> Result<(), String> {
    if depth > MAX_SCHEMA_DEPTH {
        return Err("schema exceeds the maximum nesting depth".to_string());
    }
    if schema.is_boolean() {
        return Ok(());
    }
    let object = schema
        .as_object()
        .ok_or_else(|| "schemas must be JSON objects".to_string())?;
    if let Some(reference) = object.get("$ref").and_then(Value::as_str)
        && !reference.starts_with("#/$defs/")
    {
        return Err(format!("non-local schema reference {reference}"));
    }
    for key in ["properties", "$defs"] {
        if let Some(values) = object.get(key).and_then(Value::as_object) {
            for nested in values.values() {
                validate_schema_definition(nested, depth + 1)?;
            }
        }
    }
    if let Some(items) = object.get("items") {
        validate_schema_definition(items, depth + 1)?;
    }
    for key in ["allOf", "anyOf", "oneOf"] {
        if let Some(values) = object.get(key).and_then(Value::as_array) {
            for nested in values {
                validate_schema_definition(nested, depth + 1)?;
            }
        }
    }
    if let Some(not) = object.get("not") {
        validate_schema_definition(not, depth + 1)?;
    }
    Ok(())
}

fn validate_schema(
    root: &Value,
    schema: &Value,
    value: &Value,
    depth: usize,
) -> Result<(), String> {
    if depth > MAX_SCHEMA_DEPTH {
        return Err("value exceeds the maximum schema depth".to_string());
    }
    let object = schema
        .as_object()
        .ok_or_else(|| "encountered a non-object schema".to_string())?;

    if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
        let pointer = reference
            .strip_prefix('#')
            .ok_or_else(|| format!("external schema reference {reference} is forbidden"))?;
        let target = root
            .pointer(pointer)
            .ok_or_else(|| format!("missing schema reference {reference}"))?;
        validate_schema(root, target, value, depth + 1)?;
    }

    if let Some(expected) = object.get("const")
        && expected != value
    {
        return Err("value does not match const".to_string());
    }
    if let Some(values) = object.get("enum").and_then(Value::as_array)
        && !values.contains(value)
    {
        return Err("value is not in the allowed enum".to_string());
    }

    if let Some(schemas) = object.get("allOf").and_then(Value::as_array) {
        for nested in schemas {
            validate_schema(root, nested, value, depth + 1)?;
        }
    }
    if let Some(schemas) = object.get("anyOf").and_then(Value::as_array)
        && !schemas
            .iter()
            .any(|nested| validate_schema(root, nested, value, depth + 1).is_ok())
    {
        return Err("value does not match any allowed schema".to_string());
    }
    if let Some(schemas) = object.get("oneOf").and_then(Value::as_array)
        && schemas
            .iter()
            .filter(|nested| validate_schema(root, nested, value, depth + 1).is_ok())
            .count()
            != 1
    {
        return Err("value does not match exactly one allowed schema".to_string());
    }
    if let Some(nested) = object.get("not")
        && validate_schema(root, nested, value, depth + 1).is_ok()
    {
        return Err("value matches a forbidden schema".to_string());
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
        Value::String(value) => validate_string(object, value),
        Value::Number(value) => validate_number(object, value),
        Value::Array(value) => validate_array(root, object, value, depth),
        Value::Object(value) => validate_object(root, object, value, depth),
        Value::Null | Value::Bool(_) => Ok(()),
    }
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

fn validate_string(schema: &Map<String, Value>, value: &str) -> Result<(), String> {
    let length = value.chars().count() as u64;
    if value.len() > MAX_PARAMETER_BYTES {
        return Err(format!(
            "string exceeds the {MAX_PARAMETER_BYTES}-byte parameter limit"
        ));
    }
    if let Some(minimum) = schema.get("minLength").and_then(Value::as_u64)
        && length < minimum
    {
        return Err(format!("string is shorter than {minimum} characters"));
    }
    if let Some(maximum) = schema.get("maxLength").and_then(Value::as_u64)
        && length > maximum
    {
        return Err(format!("string is longer than {maximum} characters"));
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
    if let Some(minimum) = schema.get("exclusiveMinimum").and_then(Value::as_f64)
        && value <= minimum
    {
        return Err(format!("number must be greater than {minimum}"));
    }
    if let Some(maximum) = schema.get("exclusiveMaximum").and_then(Value::as_f64)
        && value >= maximum
    {
        return Err(format!("number must be less than {maximum}"));
    }
    Ok(())
}

fn validate_array(
    root: &Value,
    schema: &Map<String, Value>,
    values: &[Value],
    depth: usize,
) -> Result<(), String> {
    if let Some(minimum) = schema.get("minItems").and_then(Value::as_u64)
        && values.len() < minimum as usize
    {
        return Err(format!("array has fewer than {minimum} items"));
    }
    if let Some(maximum) = schema.get("maxItems").and_then(Value::as_u64)
        && values.len() > maximum as usize
    {
        return Err(format!("array has more than {maximum} items"));
    }
    if schema
        .get("uniqueItems")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        for (index, value) in values.iter().enumerate() {
            if values[..index].contains(value) {
                return Err("array items must be unique".to_string());
            }
        }
    }
    if let Some(items) = schema.get("items") {
        for value in values {
            validate_schema(root, items, value, depth + 1)?;
        }
    }
    Ok(())
}

fn validate_object(
    root: &Value,
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

    let properties = schema
        .get("properties")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for (name, value) in values {
        if let Some(property_schema) = properties.get(name) {
            validate_schema(root, property_schema, value, depth + 1)?;
            continue;
        }
        match schema.get("additionalProperties") {
            Some(Value::Bool(false)) => return Err(format!("unknown property {name}")),
            Some(additional @ Value::Object(_)) => {
                validate_schema(root, additional, value, depth + 1)?;
            }
            _ => {}
        }
    }
    Ok(())
}
