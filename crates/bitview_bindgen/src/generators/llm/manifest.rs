use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io,
    path::Path,
};

use serde::Serialize;
use serde_json::{Map, Value, json, to_string_pretty};

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

/// The MCP tool catalog: a tool per non-deprecated operation without `x-mcp-ignore: true`, each of
/// which must be a `GET` ([`validate_http`]).
pub fn generate_tool_manifest(
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
    let mut content = to_string_pretty(&manifest)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    content.push('\n');
    Ok(content)
}

fn operation_from_endpoint<'a>(
    endpoint: &'a Endpoint,
    schemas: &TypeSchemas,
    names: &mut BTreeSet<&'a str>,
) -> io::Result<ManifestOperation> {
    let name = &endpoint.operation_id;
    validate_tool_name(name)?;
    validate_http(endpoint, name)?;
    if !names.insert(name) {
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
            name: name.clone(),
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
        let mut schema = parameter.schema.clone();
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

    let schema = standalone_schema(Value::Object(root), schemas)?;
    check_validated_keywords(&schema)?;
    Ok(schema)
}

/// Validation keywords the MCP bridge's argument validator does not implement.
const UNVALIDATED_KEYWORDS: [&str; 7] = [
    "const",
    "allOf",
    "oneOf",
    "not",
    "exclusiveMinimum",
    "exclusiveMaximum",
    "uniqueItems",
];

/// Reject an input schema the MCP bridge would validate only partially.
fn check_validated_keywords(schema: &Value) -> io::Result<()> {
    let Some(object) = schema.as_object() else {
        return Ok(());
    };
    if let Some(keyword) = UNVALIDATED_KEYWORDS
        .iter()
        .find(|&&key| object.contains_key(key))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("MCP argument validation does not implement `{keyword}`"),
        ));
    }
    for (key, value) in object {
        match (key.as_str(), value) {
            ("properties" | "$defs", Value::Object(schemas)) => {
                schemas.values().try_for_each(check_validated_keywords)?
            }
            ("anyOf", Value::Array(schemas)) => {
                schemas.iter().try_for_each(check_validated_keywords)?
            }
            ("items" | "additionalProperties", schema) => check_validated_keywords(schema)?,
            _ => {}
        }
    }
    Ok(())
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
    localize_refs(&mut root, &mut pending)?;
    let mut definitions = BTreeMap::new();
    while let Some(name) = pending.pop_front() {
        if definitions.contains_key(&name) {
            continue;
        }
        let mut schema = schemas
            .get(&name)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("OpenAPI schema references missing component {name}"),
                )
            })?
            .clone();
        localize_refs(&mut schema, &mut pending)?;
        definitions.insert(name, schema);
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

/// Point every component reference at its local `$defs` copy and queue the component to copy.
/// The MCP bridge resolves only local `#/$defs/<name>` references.
fn localize_refs(value: &mut Value, pending: &mut VecDeque<String>) -> io::Result<()> {
    match value {
        Value::Object(object) => {
            if let Some(Value::String(reference)) = object.get_mut("$ref") {
                if let Some(name) = reference.strip_prefix(COMPONENT_REF_PREFIX) {
                    let name = unescape_json_pointer(name);
                    *reference = format!("#/$defs/{}", escape_json_pointer(&name));
                    pending.push_back(name);
                } else if !reference.starts_with("#/$defs/") {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("unsupported JSON Schema reference: {reference}"),
                    ));
                }
            }
            for nested in object.values_mut() {
                localize_refs(nested, pending)?;
            }
        }
        Value::Array(array) => {
            for nested in array {
                localize_refs(nested, pending)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// The MCP bridge forwards tools as-is: each must be a read-only GET on a safe local path whose
/// `{placeholders}` are exactly its path parameters, with unique parameter names.
fn validate_http(endpoint: &Endpoint, name: &str) -> io::Result<()> {
    let invalid = |reason: &str| {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "MCP tool {name} ({} {}) {reason}",
                endpoint.method, endpoint.path
            ),
        ))
    };
    if endpoint.method != "GET" {
        return invalid("is not a read-only GET operation; mark it x-mcp-ignore");
    }
    let path = &endpoint.path;
    if !path.starts_with('/')
        || path.starts_with("//")
        || path.contains(['?', '#'])
        || path.split('/').any(|segment| segment == "..")
    {
        return invalid("has an unsafe path");
    }
    let placeholders = path
        .split('/')
        .filter_map(|segment| segment.strip_prefix('{')?.strip_suffix('}'))
        .collect::<BTreeSet<_>>();
    let path_params = endpoint
        .path_params
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<BTreeSet<_>>();
    if placeholders != path_params
        || path_params.len() != endpoint.path_params.len()
        || path.matches('{').count() != placeholders.len()
    {
        return invalid("path placeholders do not match its path parameters");
    }
    let mut parameters = path_params;
    if !endpoint
        .query_params
        .iter()
        .all(|parameter| parameters.insert(&parameter.name))
    {
        return invalid("repeats a parameter name");
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
