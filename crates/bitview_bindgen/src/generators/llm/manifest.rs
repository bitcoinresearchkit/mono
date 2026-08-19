use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io,
    path::Path,
};

use serde::Serialize;
use serde_json::{Map, Value, json};

use crate::{Endpoint, Parameter, TypeSchemas, generators::write_if_changed};

const MANIFEST_SCHEMA_VERSION: u32 = 1;
const JSON_SCHEMA_2020_12: &str = "https://json-schema.org/draft/2020-12/schema";
const COMPONENT_REF_PREFIX: &str = "#/components/schemas/";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    schema_version: u32,
    operations: Vec<ManifestOperation>,
}

#[derive(Serialize)]
struct ManifestOperation {
    tool: ManifestTool,
    http: ManifestHttpOperation,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestTool {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    input_schema: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_schema: Option<Value>,
    annotations: ManifestToolAnnotations,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestToolAnnotations {
    read_only_hint: bool,
    destructive_hint: bool,
    idempotent_hint: bool,
    open_world_hint: bool,
}

#[derive(Serialize)]
struct ManifestHttpOperation {
    method: String,
    path: String,
    parameters: Vec<ManifestParameter>,
}

#[derive(Serialize)]
struct ManifestParameter {
    name: String,
    location: ParameterLocation,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum ParameterLocation {
    Path,
    Query,
}

/// Generate the immutable machine-readable tool catalog in the LLM bundle.
/// Every MCP-visible, non-deprecated operation is included. Operations marked
/// with `x-mcp-ignore: true` are excluded regardless of their HTTP method.
pub(super) fn generate_tool_manifest(
    endpoints: &[Endpoint],
    schemas: &TypeSchemas,
    path: &Path,
) -> io::Result<()> {
    let content = render_tool_manifest(endpoints, schemas)?;
    write_if_changed(path, &content)
}

fn render_tool_manifest(endpoints: &[Endpoint], schemas: &TypeSchemas) -> io::Result<String> {
    let mut names = BTreeSet::new();
    let mut operations = endpoints
        .iter()
        .filter(|endpoint| !endpoint.deprecated && !endpoint.mcp_ignored)
        .map(|endpoint| operation_from_endpoint(endpoint, schemas, &mut names))
        .collect::<io::Result<Vec<_>>>()?;

    operations.sort_by(|a, b| a.tool.name.cmp(&b.tool.name));

    let manifest = Manifest {
        schema_version: MANIFEST_SCHEMA_VERSION,
        operations,
    };
    let mut content = serde_json::to_string_pretty(&manifest)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    content.push('\n');
    Ok(content)
}

fn operation_from_endpoint(
    endpoint: &Endpoint,
    schemas: &TypeSchemas,
    names: &mut BTreeSet<String>,
) -> io::Result<ManifestOperation> {
    let name = endpoint.operation_id.clone().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "MCP generation requires a stable operationId for {} {}",
                endpoint.method, endpoint.path
            ),
        )
    })?;
    validate_tool_name(&name)?;
    if !names.insert(name.clone()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("duplicate MCP tool name: {name}"),
        ));
    }

    let input_schema = build_input_schema(endpoint, schemas)?;
    let output_schema = build_output_schema(endpoint, schemas)?;
    let parameters = endpoint
        .path_params
        .iter()
        .map(|parameter| manifest_parameter(parameter, ParameterLocation::Path))
        .chain(
            endpoint
                .query_params
                .iter()
                .map(|parameter| manifest_parameter(parameter, ParameterLocation::Query)),
        )
        .collect();

    Ok(ManifestOperation {
        tool: ManifestTool {
            name,
            title: endpoint.summary.clone(),
            description: tool_description(endpoint),
            input_schema,
            output_schema,
            annotations: ManifestToolAnnotations {
                read_only_hint: true,
                destructive_hint: false,
                idempotent_hint: true,
                open_world_hint: true,
            },
        },
        http: ManifestHttpOperation {
            method: endpoint.method.clone(),
            path: endpoint.path.clone(),
            parameters,
        },
    })
}

fn tool_description(endpoint: &Endpoint) -> Option<String> {
    let description = endpoint
        .description
        .as_deref()
        .or(endpoint.summary.as_deref())?;
    Some(format!(
        "{description}\n\nREST operation: `{} {}`.",
        endpoint.method, endpoint.path
    ))
}

fn build_output_schema(endpoint: &Endpoint, schemas: &TypeSchemas) -> io::Result<Option<Value>> {
    let Some(json_schema) = endpoint.json_response_schema.clone() else {
        return Ok(None);
    };

    // Series endpoints can return either JSON or CSV depending on `format`.
    // Both successful representations must satisfy the advertised contract.
    let root = if endpoint.supports_csv {
        json!({
            "anyOf": [
                json_schema,
                {
                    "type": "string",
                    "contentMediaType": "text/csv"
                }
            ]
        })
    } else {
        json_schema
    };

    let mut schema = standalone_schema(root, schemas)?;
    strip_output_annotations(&mut schema);
    Ok(Some(schema))
}

/// Output schemas are validation contracts, not a second documentation
/// bundle. Remove annotation-only keywords while preserving every structural
/// and validation keyword. Traversal is schema-aware so an output property
/// literally named `description`, `title`, or `default` is never removed.
fn strip_output_annotations(schema: &mut Value) {
    let Some(object) = schema.as_object_mut() else {
        return;
    };
    for key in [
        "title",
        "description",
        "default",
        "examples",
        "deprecated",
        "readOnly",
        "writeOnly",
        "$comment",
    ] {
        object.remove(key);
    }

    for key in [
        "$defs",
        "definitions",
        "properties",
        "patternProperties",
        "dependentSchemas",
    ] {
        if let Some(schemas) = object.get_mut(key).and_then(Value::as_object_mut) {
            for schema in schemas.values_mut() {
                strip_output_annotations(schema);
            }
        }
    }
    for key in [
        "additionalProperties",
        "contains",
        "contentSchema",
        "else",
        "if",
        "items",
        "not",
        "propertyNames",
        "then",
        "unevaluatedItems",
        "unevaluatedProperties",
    ] {
        if let Some(schema) = object.get_mut(key) {
            strip_output_annotations(schema);
        }
    }
    for key in ["allOf", "anyOf", "oneOf", "prefixItems"] {
        if let Some(schemas) = object.get_mut(key).and_then(Value::as_array_mut) {
            for schema in schemas {
                strip_output_annotations(schema);
            }
        }
    }
}

fn manifest_parameter(parameter: &Parameter, location: ParameterLocation) -> ManifestParameter {
    ManifestParameter {
        name: parameter.name.clone(),
        location,
    }
}

fn build_input_schema(endpoint: &Endpoint, schemas: &TypeSchemas) -> io::Result<Value> {
    let mut properties = Map::new();
    let mut required = Vec::new();

    for parameter in endpoint
        .path_params
        .iter()
        .chain(endpoint.query_params.iter())
    {
        let mut schema = if parameter.schema.as_object().is_some_and(Map::is_empty) {
            schema_from_type_name(&parameter.param_type)
        } else {
            parameter.schema.clone()
        };
        if let Some(description) = &parameter.description
            && let Some(object) = schema.as_object_mut()
        {
            object
                .entry("description")
                .or_insert_with(|| Value::String(description.clone()));
        }
        properties.insert(parameter.name.clone(), schema);
        if parameter.required {
            required.push(Value::String(parameter.name.clone()));
        }
    }

    let mut root = Map::from_iter([
        ("type".to_string(), Value::String("object".to_string())),
        ("properties".to_string(), Value::Object(properties)),
        ("additionalProperties".to_string(), Value::Bool(false)),
    ]);
    if !required.is_empty() {
        root.insert("required".to_string(), Value::Array(required));
    }

    standalone_schema(Value::Object(root), schemas)
}

fn schema_from_type_name(name: &str) -> Value {
    if let Some(inner) = name.strip_suffix("[]") {
        return json!({
            "type": "array",
            "items": schema_from_type_name(inner)
        });
    }

    match name {
        "*" | "Object" => json!({}),
        "string" => json!({ "type": "string" }),
        "number" => json!({ "type": "number" }),
        "integer" => json!({ "type": "integer" }),
        "boolean" => json!({ "type": "boolean" }),
        "null" => json!({ "type": "null" }),
        other => json!({ "$ref": format!("{COMPONENT_REF_PREFIX}{other}") }),
    }
}

/// Turn an OpenAPI schema into a self-contained JSON Schema. MCP clients must
/// not fetch external `$ref` targets, so component references are rewritten to
/// local `$defs` and only the transitively used definitions are copied.
fn standalone_schema(mut root: Value, schemas: &TypeSchemas) -> io::Result<Value> {
    if root.is_boolean() {
        root = if root == Value::Bool(true) {
            json!({})
        } else {
            json!({ "not": {} })
        };
    }
    if !root.is_object() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "MCP schemas must be JSON objects",
        ));
    }

    let mut pending = VecDeque::new();
    let mut seen = BTreeSet::new();
    collect_component_refs(&root, &mut pending)?;

    let mut definitions = BTreeMap::new();
    while let Some(name) = pending.pop_front() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let schema = schemas.get(&name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("OpenAPI schema references missing component {name}"),
            )
        })?;
        collect_component_refs(schema, &mut pending)?;
        definitions.insert(name, schema.clone());
    }

    rewrite_component_refs(&mut root)?;
    for schema in definitions.values_mut() {
        rewrite_component_refs(schema)?;
    }

    let object = root.as_object_mut().expect("root checked above");
    object.insert(
        "$schema".to_string(),
        Value::String(JSON_SCHEMA_2020_12.to_string()),
    );
    if !definitions.is_empty() {
        object.insert(
            "$defs".to_string(),
            Value::Object(definitions.into_iter().collect()),
        );
    }
    Ok(root)
}

fn collect_component_refs(value: &Value, pending: &mut VecDeque<String>) -> io::Result<()> {
    match value {
        Value::Object(object) => {
            if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
                if let Some(name) = reference.strip_prefix(COMPONENT_REF_PREFIX) {
                    pending.push_back(unescape_json_pointer(name));
                } else if !reference.starts_with("#/$defs/") && !reference.starts_with("#/") {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("external JSON Schema reference is not supported: {reference}"),
                    ));
                }
            }
            for nested in object.values() {
                collect_component_refs(nested, pending)?;
            }
        }
        Value::Array(array) => {
            for nested in array {
                collect_component_refs(nested, pending)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn rewrite_component_refs(value: &mut Value) -> io::Result<()> {
    match value {
        Value::Object(object) => {
            if let Some(reference) = object.get_mut("$ref") {
                let replacement = reference
                    .as_str()
                    .and_then(|reference| reference.strip_prefix(COMPONENT_REF_PREFIX))
                    .map(|name| {
                        format!(
                            "#/$defs/{}",
                            escape_json_pointer(&unescape_json_pointer(name))
                        )
                    });
                if let Some(replacement) = replacement {
                    *reference = Value::String(replacement);
                }
            }
            for nested in object.values_mut() {
                rewrite_component_refs(nested)?;
            }
        }
        Value::Array(array) => {
            for nested in array {
                rewrite_component_refs(nested)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_tool_name(name: &str) -> io::Result<()> {
    let valid = (1..=128).contains(&name.len())
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'));
    if valid {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid MCP tool name: {name}"),
        ))
    }
}

fn escape_json_pointer(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}

fn unescape_json_pointer(value: &str) -> String {
    value.replace("~1", "/").replace("~0", "~")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Parameter, ResponseKind, TextSchema};

    fn endpoint(name: &str, method: &str) -> Endpoint {
        Endpoint {
            method: method.to_string(),
            path: "/api/items/{id}".to_string(),
            operation_id: Some(name.to_string()),
            summary: Some("Read item".to_string()),
            description: Some("Returns one item.".to_string()),
            path_params: vec![Parameter {
                name: "id".to_string(),
                required: true,
                param_type: "ItemId".to_string(),
                description: Some("Item identifier".to_string()),
                schema: json!({ "$ref": "#/components/schemas/ItemId" }),
            }],
            query_params: Vec::new(),
            request_body: None,
            response_kind: ResponseKind::Text(Some(TextSchema {
                name: "ItemId".to_string(),
                is_numeric: true,
            })),
            json_response_schema: Some(json!({
                "$ref": "#/components/schemas/Item"
            })),
            deprecated: false,
            mcp_ignored: false,
            supports_csv: false,
        }
    }

    #[test]
    fn emits_all_mcp_visible_active_operations_in_name_order() {
        let mut post = endpoint("post_item", "POST");
        post.path = "/api/items".to_string();
        let mut deprecated = endpoint("old_item", "GET");
        deprecated.deprecated = true;
        let mut ignored = endpoint("ignored_item", "GET");
        ignored.mcp_ignored = true;
        assert!(ignored.should_generate());
        let get = endpoint("get_item", "GET");
        let schemas = TypeSchemas::from(BTreeMap::from([
            ("ItemId".to_string(), json!({ "type": "integer" })),
            (
                "Item".to_string(),
                json!({
                    "description": "Item response documentation",
                    "type": "object",
                    "properties": {
                        "id": { "$ref": "#/components/schemas/ItemId" },
                        "description": {
                            "description": "Human-readable item description",
                            "type": "string",
                            "examples": ["example"],
                            "default": "example"
                        }
                    },
                    "required": ["id"]
                }),
            ),
        ]));

        let manifest = render_tool_manifest(&[post, deprecated, ignored, get], &schemas).unwrap();
        let value: Value = serde_json::from_str(&manifest).unwrap();
        let operations = value["operations"].as_array().unwrap();

        assert_eq!(operations.len(), 2);
        assert_eq!(operations[0]["tool"]["name"], "get_item");
        assert_eq!(operations[0]["http"]["method"], "GET");
        assert_eq!(
            operations[0]["tool"]["description"],
            "Returns one item.\n\nREST operation: `GET /api/items/{id}`."
        );
        assert_eq!(operations[1]["tool"]["name"], "post_item");
        assert_eq!(operations[1]["http"]["method"], "POST");
        assert_eq!(
            operations[1]["tool"]["description"],
            "Returns one item.\n\nREST operation: `POST /api/items`."
        );
        assert_eq!(
            operations[0]["tool"]["inputSchema"]["properties"]["id"]["$ref"],
            "#/$defs/ItemId"
        );
        assert_eq!(
            operations[0]["tool"]["inputSchema"]["$defs"]["ItemId"]["type"],
            "integer"
        );
        assert_eq!(
            operations[0]["tool"]["outputSchema"]["$ref"],
            "#/$defs/Item"
        );
        assert_eq!(
            operations[0]["tool"]["outputSchema"]["$defs"]["Item"]["properties"]["id"]["$ref"],
            "#/$defs/ItemId"
        );
        assert_eq!(
            operations[0]["tool"]["outputSchema"]["$defs"]["ItemId"]["type"],
            "integer"
        );
        assert!(
            operations[0]["tool"]["outputSchema"]["$defs"]["Item"]
                .get("description")
                .is_none()
        );
        let description_property =
            &operations[0]["tool"]["outputSchema"]["$defs"]["Item"]["properties"]["description"];
        assert_eq!(description_property["type"], "string");
        assert!(description_property.get("description").is_none());
        assert!(description_property.get("examples").is_none());
        assert!(description_property.get("default").is_none());
    }

    #[test]
    fn csv_output_schema_accepts_json_or_csv_text() {
        let mut endpoint = endpoint("get_item", "GET");
        endpoint.supports_csv = true;
        let schemas = TypeSchemas::from(BTreeMap::from([
            ("ItemId".to_string(), json!({ "type": "integer" })),
            (
                "Item".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "id": { "$ref": "#/components/schemas/ItemId" }
                    }
                }),
            ),
        ]));

        let manifest = render_tool_manifest(&[endpoint], &schemas).unwrap();
        let value: Value = serde_json::from_str(&manifest).unwrap();
        let output = &value["operations"][0]["tool"]["outputSchema"];

        assert_eq!(output["anyOf"][0]["$ref"], "#/$defs/Item");
        assert_eq!(output["anyOf"][1]["type"], "string");
        assert_eq!(output["anyOf"][1]["contentMediaType"], "text/csv");
    }

    #[test]
    fn omits_output_schema_without_a_json_response_schema() {
        let mut endpoint = endpoint("get_item", "GET");
        endpoint.json_response_schema = None;
        let schemas = TypeSchemas::from(BTreeMap::from([(
            "ItemId".to_string(),
            json!({ "type": "integer" }),
        )]));

        let manifest = render_tool_manifest(&[endpoint], &schemas).unwrap();
        let value: Value = serde_json::from_str(&manifest).unwrap();

        assert!(value["operations"][0]["tool"].get("outputSchema").is_none());
    }

    #[test]
    fn rejects_missing_operation_id() {
        let mut endpoint = endpoint("get_item", "GET");
        endpoint.operation_id = None;
        let error = render_tool_manifest(&[endpoint], &TypeSchemas::default()).unwrap_err();
        assert!(error.to_string().contains("requires a stable operationId"));
    }

    #[test]
    fn rejects_duplicate_operation_ids() {
        let first = endpoint("get_item", "GET");
        let second = endpoint("get_item", "GET");
        let schemas = TypeSchemas::from(BTreeMap::from([
            ("ItemId".to_string(), json!({ "type": "integer" })),
            ("Item".to_string(), json!({ "type": "object" })),
        ]));
        let error = render_tool_manifest(&[first, second], &schemas).unwrap_err();
        assert!(error.to_string().contains("duplicate MCP tool name"));
    }
}
