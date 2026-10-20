use std::{collections::btree_map::Entry, io, path::PathBuf};

use bitview_catalog::TreeNode;
use serde_json::Value;

/// Where each output goes. Rust, CLI, JavaScript, Python and the MCP tool manifest
/// (`llm_manifest`) take a file path; each `llm` call adds a root directory for `llms.txt` and
/// `llms-full.txt`. Missing parent directories are created.
#[derive(Debug, Clone, Default)]
pub struct ClientOutputPaths {
    rust: Option<PathBuf>,
    cli: Option<PathBuf>,
    javascript: Option<PathBuf>,
    python: Option<PathBuf>,
    llm: Vec<PathBuf>,
    llm_manifest: Option<PathBuf>,
}

impl ClientOutputPaths {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rust(mut self, path: impl Into<PathBuf>) -> Self {
        self.rust = Some(path.into());
        self
    }

    pub fn cli(mut self, path: impl Into<PathBuf>) -> Self {
        self.cli = Some(path.into());
        self
    }

    pub fn javascript(mut self, path: impl Into<PathBuf>) -> Self {
        self.javascript = Some(path.into());
        self
    }

    pub fn python(mut self, path: impl Into<PathBuf>) -> Self {
        self.python = Some(path.into());
        self
    }

    pub fn llm(mut self, root: impl Into<PathBuf>) -> Self {
        self.llm.push(root.into());
        self
    }

    pub fn llm_manifest(mut self, path: impl Into<PathBuf>) -> Self {
        self.llm_manifest = Some(path.into());
        self
    }
}

mod client_paths;
mod constants;
mod generators;
mod model;
mod openapi;
mod topological;
mod types;

pub use client_paths::*;
pub(crate) use generators::*;
pub(crate) use openapi::*;
pub use types::*;

use constants::*;

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Names the clients and their languages define, which series-tree shapes must not take (schema
/// type names and type-parameter letters are reserved as well).
const RUNTIME_TYPE_NAMES: &[&str] = &[
    // Client runtimes.
    "AnyDateSeriesData",
    "AnyDateSeriesEndpoint",
    "AnySeriesData",
    "AnySeriesEndpoint",
    "AnySeriesPattern",
    "BitviewClient",
    "BitviewClientBase",
    "BitviewClientOptions",
    "BitviewError",
    "ClientFetchOptions",
    "DateIndex",
    "DateRangeBuilder",
    "DateSeriesData",
    "DateSeriesDataExtras",
    "DateSeriesEndpoint",
    "DateSeriesFetchArg",
    "DateSkippedBuilder",
    "DateThenable",
    "LazyNode",
    "Node",
    "RangeBuilder",
    "SeriesData",
    "SeriesDataBase",
    "SeriesEndpoint",
    "SeriesFetchArg",
    "SeriesPattern",
    "SeriesValue",
    "SkippedBuilder",
    "Thenable",
    // JavaScript globals and the TypeScript utility types the client's JSDoc uses.
    "AbortSignal",
    "Array",
    "ArrayBuffer",
    "BigInt",
    "Boolean",
    "Date",
    "Error",
    "Iterable",
    "Map",
    "Number",
    "Object",
    "Partial",
    "Promise",
    "PromiseLike",
    "Readonly",
    "Record",
    "RequestInit",
    "Response",
    "Set",
    "Symbol",
    "URL",
    "URLSearchParams",
    "Uint8Array",
    // Python keywords and imports.
    "Any",
    "Callable",
    "Dict",
    "False",
    "Generic",
    "HTTPConnection",
    "HTTPException",
    "HTTPSConnection",
    "Iterator",
    "List",
    "Literal",
    "None",
    "Optional",
    "Protocol",
    "T",
    "True",
    "Tuple",
    "TypeVar",
    "TypedDict",
    "Union",
    // Rust keywords, prelude and imports.
    "Arc",
    "Box",
    "Err",
    "FnMut",
    "Ok",
    "OnceLock",
    "Option",
    "Result",
    "Self",
    "Send",
    "Some",
    "String",
    "Sync",
    "Vec",
];

/// Generate every output that `output_paths` gives a location, from a series catalog and the
/// OpenAPI JSON.
///
/// ```ignore
/// let paths = ClientOutputPaths::new()
///     .rust("crates/bitview_client/src/generated.rs")
///     .llm("showcases");
/// generate_clients(&catalog, &openapi_json, &paths)?;
/// ```
pub fn generate_clients(
    catalog: &TreeNode,
    openapi_json: &str,
    output_paths: &ClientOutputPaths,
) -> io::Result<()> {
    let (spec, mut schemas) = parse_openapi_json(openapi_json)?;
    let endpoints = extract_endpoints(&spec, &schemas)?;
    collect_leaf_type_schemas(catalog, &mut schemas);
    let schema_values: Vec<_> = schemas.values().cloned().collect();
    for schema in &schema_values {
        collect_schema_definitions(schema, &mut schemas);
    }

    // Every client renders the series tree from one model, with the same shape names.
    let accessors = detect_index_patterns(catalog);
    let model = model::Model::build(catalog);
    let reserved = schemas
        .keys()
        .cloned()
        .chain(RUNTIME_TYPE_NAMES.iter().map(|&name| name.to_owned()))
        .chain(accessors.iter().map(|pattern| pattern.name.clone()))
        .collect();
    let shape_names = model.shape_names(&reserved);
    let value_types = value_types(catalog);

    if let Some(rust_path) = &output_paths.rust {
        generate_rust_client(
            &model,
            &shape_names,
            &accessors,
            &value_types,
            &endpoints,
            rust_path,
        )?;
    }

    if let Some(cli_path) = &output_paths.cli {
        generate_cli(&endpoints, cli_path)?;
    }

    if let Some(js_path) = &output_paths.javascript {
        generate_javascript_client(
            &model,
            &shape_names,
            &accessors,
            &value_types.undefined,
            &endpoints,
            &schemas,
            js_path,
        )?;
    }

    if let Some(python_path) = &output_paths.python {
        generate_python_client(
            &model,
            &shape_names,
            &accessors,
            &value_types.undefined,
            &endpoints,
            &schemas,
            python_path,
        )?;
    }

    generate_llm_clients(
        catalog,
        &spec,
        &endpoints,
        &schemas,
        &output_paths.llm,
        output_paths.llm_manifest.as_deref(),
    )?;

    Ok(())
}

/// Add each leaf's value-type schema and the definitions it references (such as `Bitcoin` in
/// `Close<Bitcoin>`); OpenAPI schemas take precedence.
fn collect_leaf_type_schemas(node: &TreeNode, schemas: &mut TypeSchemas) {
    node.for_each_leaf(&mut |leaf| {
        collect_schema_definitions(&leaf.schema, schemas);

        let type_name = inner_type(&leaf.leaf.kind).to_owned();
        if let Entry::Vacant(e) = schemas.entry(type_name) {
            let schema = unwrap_allof(&leaf.schema);
            let has_type = schema.get("type").is_some();
            let has_properties = schema.get("properties").is_some();
            let has_enum = schema.get("enum").is_some() || schema.get("oneOf").is_some();
            let is_ref = schema.get("$ref").is_some();

            if has_type || has_properties || has_enum || is_ref {
                e.insert(schema.clone());
            }
        }
    });
}

/// Add the referenced schemas a schemars schema carries in `definitions` (draft-07) or `$defs`
/// (2019-09+), keeping existing entries.
fn collect_schema_definitions(schema: &Value, schemas: &mut TypeSchemas) {
    for key in ["definitions", "$defs"] {
        if let Some(defs) = schema.get(key).and_then(Value::as_object) {
            for (name, def_schema) in defs {
                schemas
                    .entry(name.clone())
                    .or_insert_with(|| def_schema.clone());
            }
        }
    }
}
