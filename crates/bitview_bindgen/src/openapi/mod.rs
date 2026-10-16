use std::{collections::BTreeMap, io, mem};

use derive_more::{Deref, DerefMut};
use oas3::{
    Map as OasMap, Spec, from_json,
    spec::{MediaType, ObjectOrReference, Operation, ParameterIn},
};
use serde_json::{Map, Value, from_str, json, to_string as SerdeJsonToString, to_value};

use crate::Ty;

mod endpoint;
mod parameter;
mod request_body;
mod response_kind;

pub(crate) use endpoint::Endpoint;
pub(crate) use parameter::Parameter;
pub(crate) use request_body::RequestBody;
pub(crate) use response_kind::ResponseKind;

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
pub(crate) fn parse_openapi_json(json: &str) -> io::Result<Spec> {
    let mut value: Value =
        from_str(json).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Clean up for oas3 compatibility
    clean_for_oas3(&mut value);

    let cleaned_json =
        SerdeJsonToString(&value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    from_json(&cleaned_json).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Extract type schemas from OpenAPI JSON
pub(crate) fn extract_schemas(json: &str) -> TypeSchemas {
    let Ok(mut value) = from_str::<Value>(json) else {
        return TypeSchemas::default();
    };

    TypeSchemas(
        value
            .get_mut("components")
            .and_then(|c| c.get_mut("schemas"))
            .and_then(Value::as_object_mut)
            .map(|schemas| mem::take(schemas).into_iter().collect())
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
                    *schema = Value::Object(Map::new());
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

/// Extract all endpoints from OpenAPI spec. Every operation needs an `operationId`: it names the
/// generated methods, commands and MCP tools.
pub(crate) fn extract_endpoints(spec: &Spec, schemas: &TypeSchemas) -> io::Result<Vec<Endpoint>> {
    let mut endpoints = Vec::new();

    let Some(paths) = &spec.paths else {
        return Ok(endpoints);
    };

    for (path, path_item) in paths {
        for (method, operation) in path_item.methods() {
            endpoints.push(extract_endpoint(path, method.as_str(), operation, schemas)?);
        }
    }

    Ok(endpoints)
}

fn extract_endpoint(
    path: &str,
    method: &str,
    operation: &Operation,
    schemas: &TypeSchemas,
) -> io::Result<Endpoint> {
    let operation_id = operation.operation_id.clone().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{method} {path} has no operationId"),
        )
    })?;
    let path_params = extract_path_parameters(path, operation);
    let query_params = extract_parameters(operation, ParameterIn::Query);

    // The 200 response's content, by media type.
    let content =
        operation
            .responses
            .as_ref()
            .and_then(|responses| match responses.get("200")? {
                ObjectOrReference::Object(response) => Some(&response.content),
                ObjectOrReference::Ref { .. } => None,
            });
    let response_kind = extract_response_kind(content, schemas);
    // The complete JSON response schema, for MCP output schemas.
    let json_response_schema = content
        .and_then(|content| content.get("application/json")?.schema.as_ref())
        .and_then(|schema| to_value(schema).ok());
    let supports_csv = content.is_some_and(|content| content.contains_key("text/csv"));
    let request_body = extract_request_body(operation);

    Ok(Endpoint {
        method: method.to_string(),
        path: path.to_string(),
        operation_id,
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

    let ty = content_ty(content).unwrap_or(if content_type.starts_with("text/") {
        Ty::String
    } else {
        Ty::Object
    });

    Some(RequestBody {
        ty,
        content_type: content_type.to_owned(),
        required: req.required.unwrap_or(false),
    })
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
                // A parameter without a schema is a string.
                let schema = param
                    .schema
                    .as_ref()
                    .and_then(|schema| to_value(schema).ok())
                    .unwrap_or_else(|| json!({ "type": "string" }));
                Some(Parameter {
                    name: param.name.clone(),
                    required: param.required.unwrap_or(false),
                    ty: Ty::parse(&schema),
                    description: param.description.clone(),
                    schema,
                })
            }
            _ => None,
        })
        .collect()
}

fn extract_response_kind(
    content: Option<&OasMap<String, MediaType>>,
    schemas: &TypeSchemas,
) -> ResponseKind {
    let Some(content) = content else {
        return ResponseKind::Text {
            ty: Ty::String,
            numeric: false,
        };
    };
    if content.contains_key("application/octet-stream") {
        return ResponseKind::Binary;
    }
    if let Some(json) = content.get("application/json") {
        return ResponseKind::Json(content_ty(json).unwrap_or(Ty::Any));
    }
    let ty = content
        .get("text/plain; charset=utf-8")
        .or_else(|| content.get("text/plain"))
        .and_then(content_ty)
        .unwrap_or(Ty::String);
    // A number type's text body (`Height`) is parsed as one.
    let numeric = match &ty {
        Ty::Ref(name) => schemas.get(name).map(Ty::parse),
        ty => Some(ty.clone()),
    }
    .is_some_and(|ty| matches!(ty, Ty::Integer | Ty::Number));
    ResponseKind::Text { ty, numeric }
}

fn content_ty(content: &MediaType) -> Option<Ty> {
    Some(Ty::parse(&to_value(content.schema.as_ref()?).ok()?))
}
