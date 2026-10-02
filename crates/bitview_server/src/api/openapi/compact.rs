use std::mem;

use aide::openapi::OpenApi;
use axum::body::Bytes;
use serde_json::{Map, Value, to_value, to_vec};

/// Compact OpenAPI spec optimized for LLM consumption.
/// Pre-serialized at startup, served as raw bytes per request.
#[derive(Clone)]
pub struct ApiJson(Bytes);

impl ApiJson {
    pub fn new(openapi: &OpenApi) -> Self {
        Self(Bytes::from(compact_json(to_value(openapi).unwrap())))
    }

    pub fn bytes(&self) -> Bytes {
        self.0.clone()
    }
}

/// Compacts an OpenAPI spec JSON to reduce size for LLM consumption.
/// Removes redundant fields while preserving essential API information.
///
/// Transformations applied (in order):
/// 1. Remove deprecated endpoints
/// 2. Remove contact/license from info
/// 3. Remove *Param schemas
/// 3. Remove error responses (304, 400, 404, 500)
/// 4. Compact responses to "returns": "Type"
/// 5. Remove per-endpoint tags and style
/// 6. Simplify parameter schema to type, remove param descriptions
/// 7. Remove summary and operationId
/// 8. Remove examples, replace $ref with type
/// 9. Flatten single-item allOf
/// 10. Flatten anyOf to type array
/// 11. Remove format
/// 12. Remove property descriptions
/// 13. Simplify properties to direct types
/// 14. Remove min/max constraints
/// 15. Trim descriptions to first paragraph, strip mempool.space links
/// 16. Remove required arrays from schemas
/// 17. Remove redundant "type": "object" when properties exist
/// 18. Flatten single-element type arrays
/// 19. Replace large enums (>40 values) with string type
fn compact_json(mut spec: Value) -> Vec<u8> {
    // Step 1: Remove deprecated endpoints from paths
    if let Some(Value::Object(paths)) = spec.get_mut("paths") {
        paths.retain(|_, v| {
            if let Value::Object(path_obj) = v
                && let Some(Value::Object(get_obj)) = path_obj.get("get")
            {
                return get_obj.get("deprecated") != Some(&Value::Bool(true));
            }
            true
        });
    }

    // Step 2: Remove contact/license from info
    if let Some(Value::Object(info)) = spec.get_mut("info") {
        info.remove("contact");
        info.remove("license");
    }

    // Step 3: Remove *Param schemas from components
    if let Some(Value::Object(components)) = spec.get_mut("components")
        && let Some(Value::Object(schemas)) = components.get_mut("schemas")
    {
        schemas.retain(|name, _| !name.ends_with("Param"));
    }

    compact_value(&mut spec);
    to_vec(&spec).unwrap()
}

fn compact_value(value: &mut Value) {
    match value {
        Value::Object(obj) => {
            // Step 1: Remove error responses
            if let Some(Value::Object(responses)) = obj.get_mut("responses") {
                for code in &["304", "400", "404", "500"] {
                    responses.remove(*code);
                }
            }

            // Step 2: Compact responses to "returns": "Type"
            if let Some(Value::Object(responses)) = obj.remove("responses")
                && let Some(returns) = extract_return_type(&responses)
            {
                obj.insert("returns".to_string(), Value::String(returns));
            }

            // Step 3: Remove per-endpoint tags and style
            // (only remove "tags" if it's an array, not if it's the top-level tags definition)
            if let Some(Value::Array(_)) = obj.get("tags") {
                // This is a per-endpoint tags array like ["Addresses"], remove it
                obj.remove("tags");
            }
            obj.remove("style");

            // Step 4: Simplify parameters (schema to type, remove descriptions)
            if let Some(Value::Array(params)) = obj.get_mut("parameters") {
                for param in params {
                    simplify_parameter(param);
                }
            }

            // Step 7: Remove summary and operationId
            obj.remove("summary");
            obj.remove("operationId");

            // Step 6: Remove examples, replace $ref with type
            obj.remove("example");
            obj.remove("examples");
            if let Some(Value::String(ref_path)) = obj.remove("$ref") {
                let type_name = ref_path.split('/').next_back().unwrap_or("any");
                obj.insert("type".to_string(), Value::String(type_name.to_string()));
            }

            // Step 7: Flatten single-item allOf
            if let Some(Value::Array(all_of)) = obj.remove("allOf")
                && all_of.len() == 1
                && let Some(Value::Object(inner)) = all_of.into_iter().next()
            {
                for (k, v) in inner {
                    obj.insert(k, v);
                }
            }

            // Step 8: Flatten anyOf to type array
            if let Some(Value::Array(any_of)) = obj.remove("anyOf") {
                let types = union_types(any_of);
                if !types.is_empty() {
                    obj.insert("type".to_string(), Value::Array(types));
                }
            }

            // Step 11: Remove format
            obj.remove("format");

            // Step 14: Remove min/max constraints
            obj.remove("minimum");
            obj.remove("maximum");

            // Step 16: Remove required arrays from schemas (but keep boolean required on params)
            if let Some(Value::Array(_)) = obj.get("required") {
                obj.remove("required");
            }

            // Step 17: Flatten single-element type arrays: ["object"] -> "object"
            if let Some(value) = obj.get_mut("type")
                && let Value::Array(arr) = value
                && arr.len() == 1
            {
                *value = arr.pop().unwrap();
            }

            // Step 18: Remove "type": "object" when properties exist (it's redundant)
            if obj.contains_key("properties")
                && obj.get("type").and_then(Value::as_str) == Some("object")
            {
                obj.remove("type");
            }

            // Step 19: Replace large enums (>40 values) with just string type
            if let Some(Value::Array(enum_values)) = obj.get("enum")
                && enum_values.len() > 40
            {
                obj.remove("enum");
            }

            // Step 15: Strip mempool.space links and keep only first paragraph of descriptions
            if let Some(Value::String(desc)) = obj.get_mut("description") {
                *desc = trim_description(desc);
            }

            // Step 12 & 13: Simplify properties (remove descriptions, simplify to direct types)
            if let Some(Value::Object(props)) = obj.get_mut("properties") {
                simplify_properties(props);
            }

            // Recurse into remaining values
            for (_, v) in obj.iter_mut() {
                compact_value(v);
            }
        }
        Value::Array(arr) => {
            for item in arr {
                compact_value(item);
            }
        }
        _ => {}
    }
}

/// Trim description to first paragraph and strip mempool.space endpoint links.
fn trim_description(desc: &str) -> String {
    // First, strip mempool.space docs links (endpoint pattern with asterisks)
    let desc = if let Some(idx) = desc.find("*[Mempool.space docs]") {
        desc[..idx].trim()
    } else {
        desc
    };

    // Keep only the first paragraph (up to \n\n)
    if let Some(idx) = desc.find("\n\n") {
        desc[..idx].trim().to_string()
    } else {
        desc.trim().to_string()
    }
}

fn extract_return_type(responses: &Map<String, Value>) -> Option<String> {
    let resp_200 = responses.get("200")?;
    let content = resp_200.get("content")?;
    let json_content = content.get("application/json")?;
    let schema = json_content.get("schema")?;
    Some(schema_to_type_string(schema))
}

fn schema_to_type_string(schema: &Value) -> String {
    if let Some(Value::String(ref_path)) = schema.get("$ref") {
        return ref_path.split('/').next_back().unwrap_or("any").to_string();
    }
    if let Some(Value::String(t)) = schema.get("type") {
        if t == "array"
            && let Some(items) = schema.get("items")
        {
            return format!("array[{}]", schema_to_type_string(items));
        }
        return t.clone();
    }
    "any".to_string()
}

fn simplify_parameter(param: &mut Value) {
    if let Value::Object(obj) = param {
        // Remove description
        obj.remove("description");

        // Extract type from schema
        if let Some(schema) = obj.remove("schema") {
            let type_val = extract_type_from_schema(schema);
            obj.insert("type".to_string(), type_val);
        }
    }
}

fn simple_type(schema: Value) -> Option<Value> {
    let Value::Object(mut obj) = schema else {
        return None;
    };
    if let Some(Value::String(path)) = obj.remove("$ref") {
        return Some(Value::String(
            path.rsplit('/').next().unwrap_or("any").to_owned(),
        ));
    }
    obj.remove("type")
}

fn union_types(schemas: Vec<Value>) -> Vec<Value> {
    schemas.into_iter().filter_map(simple_type).collect()
}

fn extract_type_from_schema(mut schema: Value) -> Value {
    if let Value::Object(obj) = &mut schema
        && let Some(Value::Array(any_of)) = obj.remove("anyOf")
    {
        let mut types = union_types(any_of);
        return if types.len() == 1 {
            types.pop().unwrap()
        } else {
            Value::Array(types)
        };
    }
    simple_type(schema).unwrap_or_else(|| Value::String("any".to_owned()))
}

fn simplify_properties(props: &mut Map<String, Value>) {
    for value in props.values_mut() {
        if let Value::Object(obj) = value {
            *value = simplify_property_value(mem::take(obj));
        }
    }
}

fn simplify_property_value(mut obj: Map<String, Value>) -> Value {
    // Remove validation constraints, format, and examples
    for key in &[
        "default",
        "minItems",
        "maxItems",
        "uniqueItems",
        "minimum",
        "maximum",
        "format",
        "examples",
        "example",
        "description",
    ] {
        obj.remove(*key);
    }

    // Remove "items": true (means any type, not useful)
    if obj.get("items") == Some(&Value::Bool(true)) {
        obj.remove("items");
    }

    // Handle $ref - convert to type (runs before recursion would)
    if let Some(Value::String(ref_path)) = obj.remove("$ref") {
        let type_name = ref_path.split('/').next_back().unwrap_or("any");
        return Value::String(type_name.to_string());
    }

    // Handle single-item allOf - flatten and extract type
    if let Some(Value::Array(all_of)) = obj.remove("allOf")
        && all_of.len() == 1
        && let Some(Value::Object(inner)) = all_of.into_iter().next()
        && let Some(t) = simple_type(Value::Object(inner))
    {
        return t;
    }

    // Handle anyOf - flatten to type array (runs before recursion would)
    if let Some(Value::Array(any_of)) = obj.remove("anyOf") {
        let types = union_types(any_of);
        return Value::Array(types);
    }

    // If only "type" remains, return just the type value
    if obj.len() == 1
        && let Some(t) = obj.remove("type")
    {
        return t;
    }

    // Handle array with items
    if obj.get("type").and_then(Value::as_str) == Some("array")
        && let Some(items) = obj.get("items")
        && let Value::Object(items_obj) = items
        && items_obj.len() == 1
    {
        // Items can have either "type" or "$ref"
        if let Some(Value::String(item_type)) = items_obj.get("type") {
            return Value::String(format!("array[{}]", item_type));
        }
        if let Some(Value::String(ref_path)) = items_obj.get("$ref") {
            let type_name = ref_path.split('/').next_back().unwrap_or("any");
            return Value::String(format!("array[{}]", type_name));
        }
    }

    Value::Object(obj)
}

#[cfg(test)]
mod tests {
    use aide::axum::ApiRouter;
    use serde_json::{from_slice, from_str, json, to_string};

    use super::*;
    use crate::{ApiRoutes, AppState, finish_openapi};

    #[test]
    fn generated_schema_compaction_matches_the_json_round_trip() {
        let (_, spec) = finish_openapi(ApiRouter::<AppState>::new().add_api_routes());
        let serialized = to_string(&spec).unwrap();
        let through_json = compact_json(from_str(&serialized).unwrap());
        assert_eq!(ApiJson::new(&spec).bytes().as_ref(), through_json);
    }

    #[test]
    fn nested_properties_preserve_extra_fields_and_union_shape() {
        let spec = json!({
            "properties": {
                "object": {"type": "object", "properties": {
                    "id": {"$ref": "#/components/schemas/Txid", "description": "drop"}
                }, "additionalProperties": false},
                "single": {"anyOf": [{"type": "string"}]},
                "empty": {"anyOf": [true, {}, {"$ref": 1}]},
                "non_object": false
            },
            "parameters": [{"schema": {"anyOf": [{"type": "string"}]}, "required": true}]
        });
        let value: Value = from_slice(&compact_json(spec)).unwrap();
        assert_eq!(
            value,
            json!({
                "properties": {
                    "object": {"properties": {"id": "Txid"}, "additionalProperties": false},
                    "single": ["string"],
                    "empty": [],
                    "non_object": false
                },
                "parameters": [{"type": "string", "required": true}]
            })
        );
    }
}
