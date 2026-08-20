mod endpoint;
mod parameter;
mod request_body;
mod response_kind;
mod text_schema;

pub use endpoint::Endpoint;
pub use parameter::Parameter;
pub use request_body::RequestBody;
pub use response_kind::ResponseKind;
pub use text_schema::TextSchema;

use std::{collections::BTreeMap, io};

use crate::ref_to_type_name;
use derive_more::{Deref, DerefMut};
use oas3::Spec;
use oas3::spec::{
    ObjectOrReference, ObjectSchema, Operation, ParameterIn, Schema, SchemaType, SchemaTypeSet,
};
use serde_json::Value;

/// Type schema extracted from OpenAPI components
#[derive(Default, Deref, DerefMut)]
pub struct TypeSchemas(BTreeMap<String, Value>);

impl From<BTreeMap<String, Value>> for TypeSchemas {
    fn from(schemas: BTreeMap<String, Value>) -> Self {
        Self(schemas)
    }
}

/// Parse OpenAPI spec from JSON string
///
/// Pre-processes the JSON to handle oas3 limitations:
/// - Removes unsupported siblings from `$ref` objects (oas3 only supports `summary` and `description`)
pub fn parse_openapi_json(json: &str) -> io::Result<Spec> {
    let mut value: Value =
        serde_json::from_str(json).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Clean up for oas3 compatibility
    clean_for_oas3(&mut value);

    let cleaned_json =
        serde_json::to_string(&value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    oas3::from_json(&cleaned_json).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Extract type schemas from OpenAPI JSON
pub fn extract_schemas(json: &str) -> TypeSchemas {
    let Ok(value) = serde_json::from_str::<Value>(json) else {
        return TypeSchemas::default();
    };

    TypeSchemas(
        value
            .get("components")
            .and_then(|c| c.get("schemas"))
            .and_then(|s| s.as_object())
            .map(|schemas| {
                schemas
                    .iter()
                    .map(|(name, schema)| (name.clone(), schema.clone()))
                    .collect()
            })
            .unwrap_or_default(),
    )
}

/// Clean up OpenAPI spec for oas3 compatibility.
/// - Removes unsupported siblings from $ref objects (oas3 only supports summary and description)
/// - Converts boolean schemas to object schemas (oas3 doesn't handle `"schema": true`)
fn clean_for_oas3(value: &mut Value) {
    match value {
        Value::Object(map) => {
            // Handle $ref with unsupported siblings
            if map.contains_key("$ref") {
                map.retain(|k, _| k == "$ref" || k == "summary" || k == "description");
            } else {
                // Convert boolean schemas to empty object schemas
                if let Some(schema) = map.get_mut("schema")
                    && schema.is_boolean()
                {
                    *schema = Value::Object(serde_json::Map::new());
                }
                for v in map.values_mut() {
                    clean_for_oas3(v);
                }
            }
        }
        Value::Array(arr) => {
            for v in arr {
                clean_for_oas3(v);
            }
        }
        _ => {}
    }
}

/// Extract all endpoints from OpenAPI spec
pub fn extract_endpoints(spec: &Spec) -> Vec<Endpoint> {
    let mut endpoints = Vec::new();

    let Some(paths) = &spec.paths else {
        return endpoints;
    };

    for (path, path_item) in paths {
        for (method, operation) in path_item.methods() {
            if let Some(endpoint) = extract_endpoint(path, method.as_str(), operation, spec) {
                endpoints.push(endpoint);
            }
        }
    }

    endpoints
}

fn extract_endpoint(
    path: &str,
    method: &str,
    operation: &Operation,
    spec: &Spec,
) -> Option<Endpoint> {
    let path_params = extract_path_parameters(path, operation);
    let query_params = extract_parameters(operation, ParameterIn::Query);

    let response_kind = extract_response_kind(operation, spec);
    let json_response_schema = extract_json_response_schema(operation);
    let request_body = extract_request_body(operation);
    let supports_csv = check_csv_support(operation);

    Some(Endpoint {
        method: method.to_string(),
        path: path.to_string(),
        operation_id: operation.operation_id.clone(),
        summary: operation.summary.clone(),
        description: operation.description.clone(),
        path_params,
        query_params,
        request_body,
        response_kind,
        json_response_schema,
        deprecated: operation.deprecated.unwrap_or(false),
        mcp_ignored: operation.extensions.get("mcp-ignore") == Some(&Value::Bool(true)),
        supports_csv,
    })
}

/// Preserve the complete JSON response schema for MCP output schema
/// generation. The regular clients only need [`ResponseKind`], while MCP
/// clients benefit from the exact self-contained response shape.
fn extract_json_response_schema(operation: &Operation) -> Option<Value> {
    let response =
        operation
            .responses
            .as_ref()?
            .get("200")
            .and_then(|response| match response {
                ObjectOrReference::Object(response) => Some(response),
                ObjectOrReference::Ref { .. } => None,
            })?;
    let schema = response.content.get("application/json")?.schema.as_ref()?;
    serde_json::to_value(schema).ok()
}

/// Extract the request body shape and media type, if any.
/// Prefers `text/plain` over `application/json` when an operation accepts both.
fn extract_request_body(operation: &Operation) -> Option<RequestBody> {
    let req = operation.request_body.as_ref()?;
    let req = match req {
        ObjectOrReference::Object(rb) => rb,
        ObjectOrReference::Ref { .. } => return None,
    };

    let (content_type, content) = [
        "text/plain; charset=utf-8",
        "text/plain",
        "application/json",
    ]
    .into_iter()
    .find_map(|content_type| {
        req.content
            .get(content_type)
            .map(|content| (content_type, content))
    })
    .or_else(|| {
        req.content
            .iter()
            .next()
            .map(|(content_type, content)| (content_type.as_str(), content))
    })?;

    let body_type = schema_name_from_content(content).unwrap_or_else(|| {
        if content_type.starts_with("text/") {
            "string".to_string()
        } else {
            "Object".to_string()
        }
    });

    Some(RequestBody {
        body_type,
        content_type: content_type.to_owned(),
        required: req.required.unwrap_or(false),
    })
}

/// Check if the endpoint supports CSV format (has text/csv in 200 response content types).
fn check_csv_support(operation: &Operation) -> bool {
    let Some(responses) = operation.responses.as_ref() else {
        return false;
    };
    let Some(response) = responses.get("200") else {
        return false;
    };
    match response {
        ObjectOrReference::Object(response) => response.content.contains_key("text/csv"),
        ObjectOrReference::Ref { .. } => false,
    }
}

/// Extract path parameters in the order they appear in the path URL.
fn extract_path_parameters(path: &str, operation: &Operation) -> Vec<Parameter> {
    // Extract parameter names from the path in order (e.g., "/api/series/{series}/{index}" -> ["series", "index"])
    let path_order: Vec<&str> = path
        .split('/')
        .filter_map(|segment| segment.strip_prefix('{').and_then(|s| s.strip_suffix('}')))
        .collect();

    // Get all path parameters from the operation
    let params = extract_parameters(operation, ParameterIn::Path);

    // Sort by position in the path
    let mut sorted_params: Vec<Parameter> = params;
    sorted_params.sort_by_key(|p| {
        path_order
            .iter()
            .position(|&name| name == p.name)
            .unwrap_or(usize::MAX)
    });

    sorted_params
}

fn extract_parameters(operation: &Operation, location: ParameterIn) -> Vec<Parameter> {
    operation
        .parameters
        .iter()
        .filter_map(|p| match p {
            ObjectOrReference::Object(param) if param.location == location => {
                let param_type = param
                    .schema
                    .as_ref()
                    .and_then(schema_type_from_schema)
                    .unwrap_or_else(|| "string".to_string());
                Some(Parameter {
                    name: param.name.clone(),
                    required: param.required.unwrap_or(false),
                    param_type,
                    description: param.description.clone(),
                    schema: param
                        .schema
                        .as_ref()
                        .and_then(|schema| serde_json::to_value(schema).ok())
                        .unwrap_or_else(|| Value::Object(serde_json::Map::new())),
                })
            }
            _ => None,
        })
        .collect()
}

fn extract_response_kind(operation: &Operation, spec: &Spec) -> ResponseKind {
    let response = operation
        .responses
        .as_ref()
        .and_then(|r| r.get("200"))
        .and_then(|r| match r {
            ObjectOrReference::Object(o) => Some(o),
            ObjectOrReference::Ref { .. } => None,
        });
    let Some(response) = response else {
        return ResponseKind::Text(None);
    };

    if response.content.contains_key("application/octet-stream") {
        return ResponseKind::Binary;
    }
    if let Some(content) = response.content.get("application/json") {
        return ResponseKind::Json(
            schema_name_from_content(content).unwrap_or_else(|| "*".to_string()),
        );
    }
    if let Some(content) = response.content.get("text/plain; charset=utf-8") {
        let schema = schema_name_from_content(content).map(|name| {
            let is_numeric = is_numeric_schema(spec, &name);
            TextSchema { name, is_numeric }
        });
        return ResponseKind::Text(schema);
    }
    ResponseKind::Text(None)
}

fn schema_name_from_content(content: &oas3::spec::MediaType) -> Option<String> {
    schema_type_from_schema(content.schema.as_ref()?)
}

/// Resolves `name` against `components.schemas` and reports whether the
/// underlying primitive is `integer` or `number`.
fn is_numeric_schema(spec: &Spec, name: &str) -> bool {
    let Some(components) = spec.components.as_ref() else {
        return false;
    };
    let Some(Schema::Object(obj_or_ref)) = components.schemas.get(name) else {
        return false;
    };
    let ObjectOrReference::Object(schema) = obj_or_ref.as_ref() else {
        return false;
    };
    matches!(
        schema.schema_type.as_ref(),
        Some(SchemaTypeSet::Single(
            SchemaType::Integer | SchemaType::Number
        ))
    )
}

fn schema_type_from_schema(schema: &Schema) -> Option<String> {
    match schema {
        Schema::Boolean(_) => Some("boolean".to_string()),
        Schema::Object(obj_or_ref) => match obj_or_ref.as_ref() {
            ObjectOrReference::Object(obj_schema) => schema_to_type_name(obj_schema),
            ObjectOrReference::Ref { ref_path, .. } => {
                // Return the type name as-is (e.g., "Height", "Address")
                // These should have definitions generated from schemas
                ref_to_type_name(ref_path).map(|s| s.to_string())
            }
        },
    }
}

fn schema_to_type_name(schema: &ObjectSchema) -> Option<String> {
    if let Some(schema_type) = schema.schema_type.as_ref() {
        return match schema_type {
            SchemaTypeSet::Single(t) => single_type_to_name(t, schema),
            SchemaTypeSet::Multiple(types) => {
                // For nullable types like ["integer", "null"], return the non-null type
                types
                    .iter()
                    .find(|t| !matches!(t, SchemaType::Null))
                    .and_then(|t| single_type_to_name(t, schema))
                    .or(Some("*".to_string()))
            }
        };
    }

    // Handle anyOf/oneOf unions (e.g., Option<RangeIndex> → anyOf: [$ref, null])
    let variants = if !schema.any_of.is_empty() {
        &schema.any_of
    } else if !schema.one_of.is_empty() {
        &schema.one_of
    } else {
        return None;
    };

    let types: Vec<String> = variants
        .iter()
        .filter_map(|v| match v {
            Schema::Boolean(_) => None,
            Schema::Object(obj_or_ref) => match obj_or_ref.as_ref() {
                ObjectOrReference::Ref { ref_path, .. } => {
                    ref_to_type_name(ref_path).map(|s| s.to_string())
                }
                ObjectOrReference::Object(obj) => {
                    if matches!(
                        obj.schema_type.as_ref(),
                        Some(SchemaTypeSet::Single(SchemaType::Null))
                    ) {
                        return None;
                    }
                    schema_to_type_name(obj)
                }
            },
        })
        .collect();

    match types.len() {
        0 => None,
        1 => Some(types.into_iter().next().unwrap()),
        _ => Some(types.join(" | ")),
    }
}

fn single_type_to_name(t: &SchemaType, schema: &ObjectSchema) -> Option<String> {
    match t {
        SchemaType::String => Some("string".to_string()),
        SchemaType::Number => Some("number".to_string()),
        SchemaType::Integer => Some("integer".to_string()),
        SchemaType::Boolean => Some("boolean".to_string()),
        SchemaType::Array => {
            let inner = match &schema.items {
                Some(boxed_schema) => schema_type_from_schema(boxed_schema),
                None => Some("*".to_string()),
            };
            inner.map(|t| format!("{}[]", t))
        }
        SchemaType::Object => Some("Object".to_string()),
        SchemaType::Null => Some("null".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_mcp_ignore_without_disabling_client_generation() {
        let spec = parse_openapi_json(
            r#"{
                "openapi": "3.1.0",
                "info": { "title": "Test", "version": "1" },
                "paths": {
                    "/api/items": {
                        "get": {
                            "operationId": "get_items",
                            "x-mcp-ignore": true,
                            "responses": {
                                "200": { "description": "Successful response" }
                            }
                        }
                    }
                }
            }"#,
        )
        .unwrap();
        let endpoint = extract_endpoints(&spec).into_iter().next().unwrap();

        assert!(endpoint.mcp_ignored);
        assert!(endpoint.should_generate());
    }

    #[test]
    fn extracts_request_body_media_type() {
        let spec = parse_openapi_json(
            r#"{
                "openapi": "3.1.0",
                "info": { "title": "Test", "version": "1" },
                "paths": {
                    "/api/items": {
                        "post": {
                            "operationId": "post_items",
                            "requestBody": {
                                "required": true,
                                "content": {
                                    "application/json": {
                                        "schema": { "type": "object" }
                                    }
                                }
                            },
                            "responses": {
                                "200": { "description": "Successful response" }
                            }
                        }
                    }
                }
            }"#,
        )
        .unwrap();
        let body = extract_endpoints(&spec)
            .into_iter()
            .next()
            .unwrap()
            .request_body
            .unwrap();

        assert_eq!(body.body_type, "Object");
        assert_eq!(body.content_type, "application/json");
        assert!(body.required);
    }

    #[test]
    fn extracts_every_openapi_http_method() {
        let spec = parse_openapi_json(
            r#"{
                "openapi": "3.1.0",
                "info": { "title": "Test", "version": "1" },
                "paths": {
                    "/api/trace": {
                        "trace": {
                            "operationId": "trace_items",
                            "responses": {
                                "200": { "description": "Successful response" }
                            }
                        }
                    }
                }
            }"#,
        )
        .unwrap();
        let endpoint = extract_endpoints(&spec).into_iter().next().unwrap();

        assert_eq!(endpoint.method, "TRACE");
    }
}
