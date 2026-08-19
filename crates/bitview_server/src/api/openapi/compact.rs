use aide::openapi::OpenApi;
use axum::body::Bytes;
use serde_json::{Map, Value};

/// Compact OpenAPI spec optimized for LLM consumption.
/// Pre-serialized at startup, served as raw bytes per request.
#[derive(Clone)]
pub struct ApiJson(Bytes);

impl ApiJson {
    pub fn new(openapi: &OpenApi) -> Self {
        let json = serde_json::to_string(openapi).unwrap();
        Self(Bytes::from(compact_json(&json)))
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
fn compact_json(json: &str) -> String {
    let mut spec: Value = serde_json::from_str(json).expect("Invalid OpenAPI JSON");

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
    serde_json::to_string(&spec).unwrap()
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
                let types: Vec<Value> = any_of
                    .into_iter()
                    .filter_map(|item| {
                        if let Value::Object(o) = item {
                            if let Some(Value::String(ref_path)) = o.get("$ref") {
                                return Some(Value::String(
                                    ref_path.split('/').next_back().unwrap_or("any").to_string(),
                                ));
                            }
                            o.get("type").cloned()
                        } else {
                            None
                        }
                    })
                    .collect();
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
            if let Some(Value::Array(arr)) = obj.get("type").cloned()
                && arr.len() == 1
            {
                obj.insert("type".to_string(), arr.into_iter().next().unwrap());
            }

            // Step 18: Remove "type": "object" when properties exist (it's redundant)
            if obj.contains_key("properties")
                && obj.get("type") == Some(&Value::String("object".to_string()))
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
            let type_val = extract_type_from_schema(&schema);
            obj.insert("type".to_string(), type_val);
        }
    }
}

fn extract_type_from_schema(schema: &Value) -> Value {
    if let Value::Object(obj) = schema {
        // Handle anyOf (optional fields)
        if let Some(Value::Array(any_of)) = obj.get("anyOf") {
            let types: Vec<Value> = any_of
                .iter()
                .filter_map(|item| {
                    if let Value::Object(o) = item {
                        if let Some(Value::String(ref_path)) = o.get("$ref") {
                            return Some(Value::String(
                                ref_path.split('/').next_back().unwrap_or("any").to_string(),
                            ));
                        }
                        o.get("type").cloned()
                    } else {
                        None
                    }
                })
                .collect();
            if types.len() == 1 {
                return types.into_iter().next().unwrap();
            }
            return Value::Array(types);
        }

        // Handle $ref
        if let Some(Value::String(ref_path)) = obj.get("$ref") {
            return Value::String(ref_path.split('/').next_back().unwrap_or("any").to_string());
        }

        // Handle type
        if let Some(t) = obj.get("type") {
            return t.clone();
        }
    }
    Value::String("any".to_string())
}

fn simplify_properties(props: &mut Map<String, Value>) {
    let keys: Vec<String> = props.keys().cloned().collect();
    for key in keys {
        if let Some(prop_value) = props.get_mut(&key)
            && let Value::Object(prop_obj) = prop_value
        {
            // Remove description
            prop_obj.remove("description");

            // Check if we can simplify to just the type
            let simplified = simplify_property_value(prop_obj);
            *prop_value = simplified;
        }
    }
}

fn simplify_property_value(obj: &mut Map<String, Value>) -> Value {
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
    {
        if let Some(Value::String(ref_path)) = inner.get("$ref") {
            let type_name = ref_path.split('/').next_back().unwrap_or("any");
            return Value::String(type_name.to_string());
        }
        if let Some(t) = inner.get("type") {
            return t.clone();
        }
    }

    // Handle anyOf - flatten to type array (runs before recursion would)
    if let Some(Value::Array(any_of)) = obj.remove("anyOf") {
        let types: Vec<Value> = any_of
            .into_iter()
            .filter_map(|item| {
                if let Value::Object(o) = item {
                    if let Some(Value::String(ref_path)) = o.get("$ref") {
                        return Some(Value::String(
                            ref_path.split('/').next_back().unwrap_or("any").to_string(),
                        ));
                    }
                    o.get("type").cloned()
                } else {
                    None
                }
            })
            .collect();
        return Value::Array(types);
    }

    // If only "type" remains, return just the type value
    if obj.len() == 1
        && let Some(t) = obj.get("type")
    {
        return t.clone();
    }

    // Handle array with items
    if obj.get("type") == Some(&Value::String("array".to_string()))
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

    Value::Object(obj.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_property_anyof() {
        let input = r##"{
            "type": "object",
            "properties": {
                "index": {
                    "anyOf": [
                        {"type": "TxIndex"},
                        {"type": "null"}
                    ]
                }
            }
        }"##;

        let result = compact_json(input);
        let parsed: Value = serde_json::from_str(&result).unwrap();

        // Property should be simplified to array, not {"type": [...]}
        let index = &parsed["properties"]["index"];
        assert!(index.is_array(), "Expected array, got: {}", index);
        assert_eq!(index[0], "TxIndex");
        assert_eq!(index[1], "null");
    }

    #[test]
    fn test_trim_parameter_anyof() {
        let input = r##"{
            "parameters": [{
                "in": "query",
                "name": "after_txid",
                "schema": {
                    "anyOf": [
                        {"$ref": "#/components/schemas/Txid"},
                        {"type": "null"}
                    ]
                }
            }]
        }"##;

        let result = compact_json(input);
        let parsed: Value = serde_json::from_str(&result).unwrap();

        // Parameter should have type array including null
        let param = &parsed["parameters"][0];
        assert_eq!(param["name"], "after_txid");
        assert_eq!(param["type"][0], "Txid");
        assert_eq!(param["type"][1], "null");
    }

    #[test]
    fn test_trim_property_ref() {
        let input = r##"{
            "type": "object",
            "properties": {
                "txid": {
                    "$ref": "#/components/schemas/Txid"
                }
            }
        }"##;

        let result = compact_json(input);
        let parsed: Value = serde_json::from_str(&result).unwrap();

        // Property with $ref should be simplified to just the type name
        assert_eq!(parsed["properties"]["txid"], "Txid");
    }

    #[test]
    fn test_trim_nested_component_schema() {
        // This matches the actual API structure: components > schemas > Type > properties
        let input = r##"{
            "components": {
                "schemas": {
                    "AddressStats": {
                        "type": "object",
                        "properties": {
                            "address": {
                                "$ref": "#/components/schemas/Address"
                            },
                            "chain_stats": {
                                "$ref": "#/components/schemas/AddressChainStats"
                            }
                        }
                    }
                }
            }
        }"##;

        let result = compact_json(input);
        let parsed: Value = serde_json::from_str(&result).unwrap();

        let props = &parsed["components"]["schemas"]["AddressStats"]["properties"];
        assert_eq!(props["address"], "Address", "address should be simplified");
        assert_eq!(
            props["chain_stats"], "AddressChainStats",
            "chain_stats should be simplified"
        );
    }

    #[test]
    fn test_trim_property_allof_with_ref() {
        // Real API uses allOf wrapper around $ref
        let input = r##"{
            "type": "object",
            "properties": {
                "address": {
                    "description": "Bitcoin address string",
                    "allOf": [
                        {"$ref": "#/components/schemas/Address"}
                    ]
                }
            }
        }"##;

        let result = compact_json(input);
        let parsed: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["properties"]["address"], "Address");
    }

    #[test]
    fn test_trim_property_array_with_ref() {
        let input = r##"{
            "type": "object",
            "properties": {
                "vin": {
                    "type": "array",
                    "items": {
                        "$ref": "#/components/schemas/TxIn"
                    }
                }
            }
        }"##;

        let result = compact_json(input);
        let parsed: Value = serde_json::from_str(&result).unwrap();

        // Array with $ref items should be simplified to "array[Type]"
        assert_eq!(parsed["properties"]["vin"], "array[TxIn]");
    }

    #[test]
    fn test_trim_responses_to_returns() {
        let input = r##"{
            "responses": {
                "200": {
                    "content": {
                        "application/json": {
                            "schema": {"$ref": "#/components/schemas/Block"}
                        }
                    }
                },
                "400": {"description": "Bad request"},
                "500": {"description": "Error"}
            }
        }"##;

        let result = compact_json(input);
        let parsed: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["returns"], "Block");
        assert!(parsed.get("responses").is_none());
    }
}
