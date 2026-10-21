use std::collections::BTreeSet;

use bitview_primitives::Index;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{JsonType, SeriesLeaf};

/// Series leaf with JSON Schema for client generation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SeriesLeafWithSchema {
    #[serde(flatten)]
    pub leaf: SeriesLeaf,
    /// What the values are on the wire.
    #[serde(rename = "type")]
    pub(crate) openapi_type: JsonType,
    /// JSON Schema for the value type.
    #[serde(skip)]
    pub schema: Value,
    /// Indexes whose values can be missing (e.g. a period without blocks), written as `null`.
    #[serde(skip)]
    pub missing: BTreeSet<Index>,
    /// The value type has an undefined value of its own (NaN, or a sentinel), written as `null`.
    #[serde(skip)]
    pub undefined: bool,
}

impl SeriesLeafWithSchema {
    pub fn new(leaf: SeriesLeaf, schema: Value) -> Self {
        let openapi_type = extract_json_type(&schema);
        Self {
            leaf,
            openapi_type,
            schema,
            missing: BTreeSet::new(),
            undefined: false,
        }
    }

    /// Merge compatible metadata for another occurrence of the same series.
    pub(crate) fn merge(&mut self, other: &Self) -> Option<()> {
        self.missing.extend(other.missing.iter().copied());
        self.undefined |= other.undefined;
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
pub fn extract_json_type(schema: &Value) -> JsonType {
    extract_json_type_inner(schema, schema)
}

fn extract_json_type_inner(node: &Value, root: &Value) -> JsonType {
    if let Some(value_type) = node
        .get("type")
        .and_then(Value::as_str)
        .and_then(JsonType::parse)
    {
        return value_type;
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
                if value_type != JsonType::Null {
                    return value_type;
                }
            }
        }
    }

    JsonType::Object
}
