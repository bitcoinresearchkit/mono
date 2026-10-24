use std::{collections::BTreeMap, io, mem::take};

use serde_json::{Map, Value, from_str, json};

use crate::Ty;

mod endpoint;
mod parameter;
mod request_body;
mod response_kind;

pub(crate) use endpoint::Endpoint;
pub(crate) use parameter::Parameter;
pub(crate) use request_body::RequestBody;
pub(crate) use response_kind::ResponseKind;

/// Schemas by type name.
pub(crate) type TypeSchemas = BTreeMap<String, Value>;

/// What the generators read of the OpenAPI document: its title and version, and its paths, in
/// the document's order.
pub(crate) struct Spec {
    pub(crate) info: Info,
    paths: Map<String, Value>,
}

pub(crate) struct Info {
    pub(crate) title: String,
    pub(crate) version: String,
}

/// The HTTP methods an OpenAPI path item may hold, in the order endpoints are listed.
const METHODS: [&str; 8] = [
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

/// The spec and the component schemas, as written: only the paths are cleaned.
pub(crate) fn parse_openapi_json(json: &str) -> io::Result<(Spec, TypeSchemas)> {
    let invalid = |message: String| io::Error::new(io::ErrorKind::InvalidData, message);
    let mut value: Value = from_str(json).map_err(|e| invalid(e.to_string()))?;
    let schemas = value
        .get("components")
        .and_then(|c| c.get("schemas"))
        .and_then(Value::as_object)
        .map(|schemas| schemas.clone().into_iter().collect())
        .unwrap_or_default();

    clean(&mut value);
    let text = |pointer: &str| {
        value
            .pointer(pointer)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| invalid(format!("the OpenAPI document has no {pointer}")))
    };
    let info = Info {
        title: text("/info/title")?,
        version: text("/info/version")?,
    };
    let paths = match value.get_mut("paths") {
        Some(Value::Object(paths)) => take(paths),
        _ => Map::new(),
    };
    Ok((Spec { info, paths }, schemas))
}

/// Drop keys set to null, which say nothing (as `"default": null`), and turn boolean schemas
/// (`"schema": true`) into empty objects. A `$ref`'s siblings stay: OpenAPI 3.1 applies them
/// (`"default": "raw"` beside `UrpdAggregation`).
fn clean(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|_, v| !v.is_null());
            if let Some(schema) = map.get_mut("schema")
                && schema.is_boolean()
            {
                *schema = Value::Object(Map::new());
            }
            for v in map.values_mut() {
                clean(v);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                clean(v);
            }
        }
        _ => {}
    }
}

/// An object of the spec, unless it is a `$ref`, which the generators skip.
fn object(value: &Value) -> Option<&Map<String, Value>> {
    value.as_object().filter(|object| !object.contains_key("$ref"))
}

fn text(object: &Map<String, Value>, key: &str) -> Option<String> {
    object.get(key)?.as_str().map(str::to_owned)
}

fn flag(object: &Map<String, Value>, key: &str) -> bool {
    object.get(key).and_then(Value::as_bool).unwrap_or(false)
}

/// An object's media types by name, in the document's order.
fn media_types(object: &Map<String, Value>) -> Map<String, Value> {
    object
        .get("content")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

/// Extract all endpoints from OpenAPI spec. Every operation needs an `operationId`: it names the
/// generated methods, commands and MCP tools.
pub(crate) fn extract_endpoints(spec: &Spec, schemas: &TypeSchemas) -> io::Result<Vec<Endpoint>> {
    let mut endpoints = Vec::new();
    for (path, path_item) in &spec.paths {
        for method in METHODS {
            if let Some(operation) = path_item.get(method).and_then(Value::as_object) {
                endpoints.push(extract_endpoint(
                    path,
                    &method.to_ascii_uppercase(),
                    operation,
                    schemas,
                )?);
            }
        }
    }
    Ok(endpoints)
}

fn extract_endpoint(
    path: &str,
    method: &str,
    operation: &Map<String, Value>,
    schemas: &TypeSchemas,
) -> io::Result<Endpoint> {
    let operation_id = text(operation, "operationId").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{method} {path} has no operationId"),
        )
    })?;
    let path_params = extract_path_parameters(path, operation);
    let query_params = extract_parameters(operation, "query");

    let content = operation
        .get("responses")
        .and_then(|responses| object(responses.get("200")?))
        .map(media_types);
    let content = content.as_ref();
    let response_kind = extract_response_kind(content, schemas);
    // The complete JSON response schema, for MCP output schemas.
    let json_response_schema = content
        .and_then(|content| content.get("application/json")?.get("schema"))
        .cloned();
    // A text body's schema (`BlockHash`, `Height`), for the MCP output schemas of text tools.
    let text_response_schema = content
        .and_then(|content| {
            content
                .get("text/plain; charset=utf-8")
                .or_else(|| content.get("text/plain"))?
                .get("schema")
        })
        .cloned();
    let supports_csv = content.is_some_and(|content| content.contains_key("text/csv"));
    let request_body = extract_request_body(operation);

    Ok(Endpoint {
        method: method.to_string(),
        path: path.to_string(),
        operation_id,
        summary: text(operation, "summary"),
        description: text(operation, "description"),
        path_params,
        query_params,
        request_body,
        response_kind,
        json_response_schema,
        text_response_schema,
        deprecated: flag(operation, "deprecated"),
        mcp_ignored: operation.get("x-mcp-ignore") == Some(&Value::Bool(true)),
        supports_csv,
    })
}

/// Extract the request body shape and media type, if any.
/// Prefers `text/plain` over `application/json` when an operation accepts both.
fn extract_request_body(operation: &Map<String, Value>) -> Option<RequestBody> {
    let req = object(operation.get("requestBody")?)?;
    let media_types = media_types(req);

    let (content_type, content) = [
        "text/plain; charset=utf-8",
        "text/plain",
        "application/json",
    ]
    .into_iter()
    .find_map(|content_type| {
        media_types
            .get(content_type)
            .map(|content| (content_type, content))
    })
    .or_else(|| {
        media_types
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
        required: flag(req, "required"),
    })
}

/// Extract path parameters in the order they appear in the path URL.
fn extract_path_parameters(path: &str, operation: &Map<String, Value>) -> Vec<Parameter> {
    let path_order: Vec<&str> = path
        .split('/')
        .filter_map(|segment| segment.strip_prefix('{').and_then(|s| s.strip_suffix('}')))
        .collect();
    let mut params = extract_parameters(operation, "path");
    params.sort_by_key(|p| {
        path_order
            .iter()
            .position(|&name| name == p.name)
            .unwrap_or(usize::MAX)
    });
    params
}

fn extract_parameters(operation: &Map<String, Value>, location: &str) -> Vec<Parameter> {
    operation
        .get("parameters")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(object)
        .filter(|param| param.get("in").and_then(Value::as_str) == Some(location))
        .map(|param| {
            // A parameter without a schema is a string.
            let schema = param
                .get("schema")
                .cloned()
                .unwrap_or_else(|| json!({ "type": "string" }));
            let name = text(param, "name").unwrap_or_default();
            Parameter {
                arg: text(param, "x-arg-name").unwrap_or_else(|| name.replace(['[', ']'], "")),
                name,
                required: flag(param, "required"),
                ty: Ty::parse(&schema),
                description: text(param, "description"),
                schema,
            }
        })
        .collect()
}

fn extract_response_kind(
    content: Option<&Map<String, Value>>,
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
    .is_some_and(|ty| matches!(ty, Ty::Integer(_) | Ty::Number));
    ResponseKind::Text { ty, numeric }
}

fn content_ty(content: &Value) -> Option<Ty> {
    Some(Ty::parse(content.get("schema")?))
}
