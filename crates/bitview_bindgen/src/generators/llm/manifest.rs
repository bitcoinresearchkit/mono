use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io,
    path::Path,
};

use serde::Serialize;
use serde_json::{Map, Value, to_string_pretty};

use crate::{Endpoint, Parameter, TypeSchemas, generators::write_if_changed};

const MANIFEST_SCHEMA_VERSION: u32 = 2;
const COMPONENT_REF_PREFIX: &str = "#/components/schemas/";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Manifest<'a> {
    schema_version: u32,
    instructions: &'a str,
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
    argument: String,
    location: ParameterLocation,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum ParameterLocation {
    Path,
    Query,
}

/// Generate an MCP tool catalog from the selected endpoints, each of which must be a `GET`
/// ([`validate_http`]).
pub fn generate_tool_manifest(
    endpoints: &[&Endpoint],
    schemas: &TypeSchemas,
    instructions: &str,
    path: &Path,
) -> io::Result<()> {
    let content = render_tool_manifest(endpoints, schemas, instructions)?;
    write_if_changed(path, &content)
}

fn render_tool_manifest(
    endpoints: &[&Endpoint],
    schemas: &TypeSchemas,
    instructions: &str,
) -> io::Result<String> {
    let routes = endpoints
        .iter()
        .map(|endpoint| (super::route(endpoint), endpoint.operation_id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut names = BTreeSet::new();
    let mut operations = endpoints
        .iter()
        .map(|endpoint| operation_from_endpoint(endpoint, &routes, schemas, &mut names))
        .collect::<io::Result<Vec<_>>>()?;

    operations.sort_by(|a, b| a.tool.name.cmp(&b.tool.name));

    let manifest = Manifest {
        schema_version: MANIFEST_SCHEMA_VERSION,
        instructions,
        operations,
    };
    let mut content = to_string_pretty(&manifest)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    content.push('\n');
    Ok(content)
}

fn operation_from_endpoint<'a>(
    endpoint: &'a Endpoint,
    routes: &BTreeMap<String, &str>,
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
            description: tool_description(endpoint, routes),
            input_schema,
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

/// What a model reads to choose a tool: the domain first, since a tool list can mix servers, then
/// the operation's own description. A paragraph that is only a link to another API's docs stays in
/// the OpenAPI document and out of here.
fn tool_description(endpoint: &Endpoint, routes: &BTreeMap<String, &str>) -> Option<String> {
    let description = endpoint
        .description
        .as_deref()
        .map(without_link_paragraphs)
        .filter(|description| !description.is_empty())
        .or_else(|| endpoint.summary.clone())?;
    let description = description
        .split('`')
        .enumerate()
        .map(|(index, mention)| {
            if index % 2 == 0 {
                return mention.to_string();
            }
            routes.get(mention).copied().unwrap_or(mention).to_string()
        })
        .collect::<Vec<_>>()
        .join("`");
    Some(format!("Bitcoin: {description}"))
}

fn without_link_paragraphs(description: &str) -> String {
    description
        .split("\n\n")
        .filter(|paragraph| {
            let paragraph = paragraph.trim();
            !(paragraph.starts_with("*[") && paragraph.ends_with(")*"))
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Replaces a root `$ref` with the definition it names, so the root states its own type.
fn inline_root_reference(schema: &mut Value, schemas: &TypeSchemas) {
    let Some(root) = schema.as_object_mut() else {
        return;
    };
    let target = root
        .get("$ref")
        .and_then(Value::as_str)
        .and_then(|reference| reference.strip_prefix(COMPONENT_REF_PREFIX))
        .and_then(|name| schemas.get(&unescape_json_pointer(name)))
        .and_then(|target| target.as_object().cloned());
    let Some(target) = target else {
        return;
    };
    root.remove("$ref");
    for (key, value) in target {
        root.entry(key).or_insert(value);
    }
}

fn manifest_parameter(parameter: &Parameter, location: ParameterLocation) -> ManifestParameter {
    ManifestParameter {
        name: parameter.name.clone(),
        argument: parameter.arg.clone(),
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
        inline_root_reference(&mut schema, schemas);
        // A union also states its accepted primitive types for clients that inspect only `type`.
        if schema.get("type").is_none()
            && let Some(branches) = schema.get("anyOf").and_then(Value::as_array)
        {
            let types = branches
                .iter()
                .map(|branch| {
                    let mut branch = branch.clone();
                    inline_root_reference(&mut branch, schemas);
                    branch
                        .get("type")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                })
                .collect::<Option<BTreeSet<_>>>();
            if let Some(types) = types {
                let types = types
                    .into_iter()
                    .map(|kind| Value::String(kind.to_string()))
                    .collect();
                schema
                    .as_object_mut()
                    .expect("parameter schema object")
                    .insert("type".into(), Value::Array(types));
            }
        }
        if let Some(description) = &parameter.description
            && let Some(object) = schema.as_object_mut()
        {
            object
                .entry("description")
                .or_insert_with(|| Value::String(description.clone()));
        }
        // MCP clients refuse other names (Claude: `^[a-zA-Z0-9_.-]{1,64}$`), so `txId[]` is
        // offered as `txids`.
        let argument = &parameter.arg;
        if argument.is_empty()
            || argument.len() > 64
            || !argument
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("MCP argument name `{argument}` is not [a-zA-Z0-9_.-]{{1,64}}"),
            ));
        }
        properties.insert(argument.clone(), schema);
        if parameter.required {
            required.push(Value::String(argument.clone()));
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

    let schema = standalone_schema(root, schemas)?;
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
fn standalone_schema(root: Map<String, Value>, schemas: &TypeSchemas) -> io::Result<Value> {
    let mut root = Value::Object(root);
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

    let object = root.as_object_mut().expect("input schema object");
    if !definitions.is_empty() {
        object.insert(
            "$defs".to_string(),
            Value::Object(definitions.into_iter().collect()),
        );
    }
    normalize_input_schema(&mut root);
    Ok(root)
}

/// The `format`s JSON Schema 2020-12 defines. Clients' validators warn on any other (`uint32`,
/// `double`: schemars' Rust types), once per use; the type and bounds say what they mean.
const JSON_SCHEMA_FORMATS: [&str; 19] = [
    "date-time",
    "date",
    "time",
    "duration",
    "email",
    "idn-email",
    "hostname",
    "idn-hostname",
    "ipv4",
    "ipv6",
    "uri",
    "uri-reference",
    "iri",
    "iri-reference",
    "uuid",
    "uri-template",
    "json-pointer",
    "relative-json-pointer",
    "regex",
];

/// Keep standard formats and one example per schema. Values such as
/// `default`, `examples`, `enum` and `const` are data, not nested schemas.
fn normalize_input_schema(schema: &mut Value) {
    let Some(object) = schema.as_object_mut() else {
        return;
    };
    if let Some(examples) = object.get_mut("examples").and_then(Value::as_array_mut) {
        examples.truncate(1);
    }
    if object
        .get("format")
        .and_then(Value::as_str)
        .is_some_and(|format| !JSON_SCHEMA_FORMATS.contains(&format))
    {
        object.remove("format");
    }
    for (key, value) in object {
        match (key.as_str(), value) {
            ("properties" | "$defs", Value::Object(schemas)) => {
                schemas.values_mut().for_each(normalize_input_schema);
            }
            ("anyOf", Value::Array(schemas)) => {
                schemas.iter_mut().for_each(normalize_input_schema);
            }
            ("items" | "additionalProperties", schema) => normalize_input_schema(schema),
            _ => {}
        }
    }
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
