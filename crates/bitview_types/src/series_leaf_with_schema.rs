use std::collections::BTreeSet;

use brk_types::Index;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::SeriesLeaf;

/// Series leaf with JSON Schema for client generation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SeriesLeafWithSchema {
    /// The core series metadata.
    #[serde(flatten)]
    pub leaf: SeriesLeaf,
    /// JSON Schema type (e.g., "integer", "number", "string", "boolean", "array", "object").
    #[serde(rename = "type")]
    pub openapi_type: String,
    /// JSON Schema for the value type.
    #[serde(skip)]
    pub schema: Value,
}

impl SeriesLeafWithSchema {
    pub fn new(leaf: SeriesLeaf, schema: Value) -> Self {
        let openapi_type = extract_json_type(&schema);
        Self {
            leaf,
            openapi_type,
            schema,
        }
    }

    /// The OpenAPI/JSON Schema type.
    pub fn openapi_type(&self) -> &str {
        &self.openapi_type
    }

    /// The series name/identifier.
    pub fn name(&self) -> &str {
        &self.leaf.name
    }

    /// The Rust type (e.g., "Sats", "StoredF64").
    pub fn kind(&self) -> &str {
        &self.leaf.kind
    }

    /// Available indexes for this series.
    pub fn indexes(&self) -> &BTreeSet<Index> {
        &self.leaf.indexes
    }

    /// Human-readable metric definition, when documented.
    pub fn description(&self) -> Option<&str> {
        self.leaf.description.as_deref()
    }

    /// Check if this leaf refers to the same series as another.
    pub fn is_same_series(&self, other: &Self) -> bool {
        self.leaf.name == other.leaf.name
    }

    /// Merge compatible metadata for another occurrence of the same series.
    pub fn merge(&mut self, other: &Self) -> Option<()> {
        self.leaf.merge(&other.leaf)
    }
}

impl PartialEq for SeriesLeafWithSchema {
    fn eq(&self, other: &Self) -> bool {
        self.leaf == other.leaf
    }
}

impl Eq for SeriesLeafWithSchema {}

/// Extract JSON type from a root schema, following `$ref` and composition keywords.
pub fn extract_json_type(schema: &Value) -> String {
    extract_json_type_inner(schema, schema)
}

fn extract_json_type_inner(node: &Value, root: &Value) -> String {
    if let Some(value_type) = node.get("type").and_then(Value::as_str) {
        return value_type.to_string();
    }

    if let Some(reference) = node.get("$ref").and_then(Value::as_str)
        && let Some(name) = reference.rsplit('/').next()
    {
        for definitions_key in &["$defs", "definitions"] {
            if let Some(definitions) = root.get(definitions_key)
                && let Some(definition) = definitions.get(name)
            {
                return extract_json_type_inner(definition, root);
            }
        }
    }

    if let Some(all_of) = node.get("allOf").and_then(Value::as_array)
        && all_of.len() == 1
    {
        return extract_json_type_inner(&all_of[0], root);
    }

    for key in &["anyOf", "oneOf"] {
        if let Some(variants) = node.get(key).and_then(Value::as_array) {
            for variant in variants {
                let value_type = extract_json_type_inner(variant, root);
                if value_type != "null" {
                    return value_type;
                }
            }
        }
    }

    "object".to_string()
}
