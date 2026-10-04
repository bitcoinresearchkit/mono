//! Python type definitions generation.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use serde_json::Value;

use crate::{
    TypeSchemas, escape_python_keyword,
    generators::{MANUAL_GENERIC_TYPES, write_description},
    get_union_variants, is_rust_concrete_generic, ref_to_type_name, rust_array_element_type,
};

/// Generate type definitions from schemas.
pub(crate) fn generate_type_definitions(output: &mut String, schemas: &TypeSchemas) {
    if schemas.is_empty() {
        return;
    }

    writeln!(output, "# Type definitions\n").unwrap();

    let sorted_names = topological_sort_schemas(schemas);

    // Partition into simple type aliases and TypedDict classes
    // Generate type aliases first to avoid forward reference issues
    let (type_aliases, typed_dicts): (Vec<_>, Vec<_>) = sorted_names
        .into_iter()
        .filter(|name| !MANUAL_GENERIC_TYPES.contains(&name.as_str()))
        .filter(|name| rust_array_element_type(name).is_none())
        .filter(|name| !is_rust_concrete_generic(name))
        .filter(|name| schemas.contains_key(name))
        .partition(|name| {
            schemas
                .get(name)
                .map(|s| s.get("properties").is_none())
                .unwrap_or(false)
        });

    // Generate simple type aliases first
    // Quote references to TypedDicts since they're defined after
    let typed_dict_set: BTreeSet<_> = typed_dicts.iter().cloned().collect();
    for name in type_aliases {
        let schema = &schemas[&name];
        let type_desc = schema.get("description").and_then(|d| d.as_str());
        let py_type = schema_to_python_type(schema, Some(&name), Some(&typed_dict_set));
        if let Some(desc) = type_desc {
            write_description(output, desc, "# ", "#");
        }
        writeln!(output, "{} = {}", name, py_type).unwrap();
    }

    // Then generate TypedDict classes
    for name in typed_dicts {
        let schema = &schemas[&name];
        let type_desc = schema.get("description").and_then(|d| d.as_str());
        let props = schema
            .get("properties")
            .and_then(|p| p.as_object())
            .unwrap();

        // A property outside `required` can be absent from the JSON: required keys go in a base
        // class, optional ones in a `total=False` class (correct at runtime and for every checker).
        let required: BTreeSet<&str> = schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|r| r.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        let (required_props, optional_props): (Vec<_>, Vec<_>) =
            props.iter().partition(|(prop_name, _)| required.contains(prop_name.as_str()));
        let field = |output: &mut String, (prop_name, prop_schema): (&String, &Value)| {
            let prop_type = schema_to_python_type(prop_schema, Some(&name), None);
            writeln!(output, "    {}: {}", escape_python_keyword(prop_name), prop_type).unwrap();
        };

        // Keys that aren't Python identifiers (`24h`, `txId[]`) need the functional form, which
        // can't mix required and optional keys on Python 3.9.
        let identifier = |key: &str| {
            escape_python_keyword(key) == key
                && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        };
        if !props.keys().all(|key| identifier(key)) {
            assert!(
                required_props.is_empty() || optional_props.is_empty(),
                "{name} mixes optional keys with keys that aren't Python identifiers"
            );
            if let Some(desc) = type_desc {
                write_description(output, desc, "# ", "#");
            }
            let entries: Vec<String> = props
                .iter()
                .map(|(key, prop_schema)| {
                    format!("{key:?}: {}", schema_to_python_type(prop_schema, Some(&name), None))
                })
                .collect();
            let total = if required_props.is_empty() { ", total=False" } else { "" };
            writeln!(output, "{name} = TypedDict({name:?}, {{{}}}{total})\n", entries.join(", "))
                .unwrap();
            continue;
        }

        let header = match (required_props.is_empty(), optional_props.is_empty()) {
            (_, true) => format!("class {name}(TypedDict):"),
            (true, false) => format!("class {name}(TypedDict, total=False):"),
            (false, false) => {
                writeln!(output, "class _{name}Required(TypedDict):").unwrap();
                for &prop in &required_props {
                    field(output, prop);
                }
                writeln!(output).unwrap();
                format!("class {name}(_{name}Required, total=False):")
            }
        };
        writeln!(output, "{header}").unwrap();

        // Collect field descriptions for Attributes section
        let field_docs: Vec<(String, Option<&str>)> = props
            .iter()
            .map(|(prop_name, prop_schema)| {
                let safe_name = escape_python_keyword(prop_name);
                let desc = prop_schema.get("description").and_then(|d| d.as_str());
                (safe_name, desc)
            })
            .collect();
        let has_field_docs = field_docs.iter().any(|(_, d)| d.is_some());

        // Generate docstring if we have type description or field descriptions
        if type_desc.is_some() || has_field_docs {
            writeln!(output, "    \"\"\"").unwrap();
            if let Some(desc) = type_desc {
                for line in desc.lines() {
                    writeln!(output, "    {}", line).unwrap();
                }
            }
            if has_field_docs {
                if type_desc.is_some() {
                    writeln!(output).unwrap();
                }
                writeln!(output, "    Attributes:").unwrap();
                for (field_name, desc) in &field_docs {
                    if let Some(d) = desc {
                        writeln!(output, "        {}: {}", field_name, d.split_whitespace().collect::<Vec<_>>().join(" ")).unwrap();
                    }
                }
            }
            writeln!(output, "    \"\"\"").unwrap();
        }

        let own = if optional_props.is_empty() { &required_props } else { &optional_props };
        for &prop in own {
            field(output, prop);
        }
        writeln!(output).unwrap();
    }
    writeln!(output).unwrap();
}

/// Topologically sort schema names so dependencies come before dependents (avoids forward references).
/// Types that reference other types (via $ref) must be defined after their dependencies.
fn topological_sort_schemas(schemas: &TypeSchemas) -> Vec<String> {
    // Build dependency graph
    let mut deps: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (name, schema) in schemas.iter() {
        let mut type_deps = BTreeSet::new();
        collect_schema_refs(schema, &mut type_deps);
        // Only keep deps that are in our schemas, and drop self-references
        // (handled at emit time by quoting via current_type)
        type_deps.retain(|d| schemas.contains_key(*d) && *d != name);
        deps.insert(name, type_deps);
    }

    // Kahn's algorithm for topological sort
    let mut in_degree: BTreeMap<&str, usize> = BTreeMap::new();
    for name in schemas.keys() {
        in_degree.insert(name, 0);
    }
    for type_deps in deps.values() {
        for dep in type_deps {
            *in_degree.entry(dep).or_insert(0) += 1;
        }
    }

    // Start with types that have no dependents (are not referenced by others)
    let mut queue: BTreeSet<&str> = in_degree
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(name, _)| *name)
        .collect();

    let mut result = Vec::new();
    while let Some(name) = queue.pop_last() {
        result.push(name);
        if let Some(type_deps) = deps.get(name) {
            for dep in type_deps {
                if let Some(count) = in_degree.get_mut(dep) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        queue.insert(dep);
                    }
                }
            }
        }
    }

    // Reverse so dependencies come first
    result.reverse();

    // Add any types that weren't processed (e.g., due to circular refs or other edge cases)
    let result_set: BTreeSet<_> = result.iter().copied().collect();
    result.extend(
        schemas
            .keys()
            .map(String::as_str)
            .filter(|k| !result_set.contains(k)),
    );

    result.into_iter().map(str::to_owned).collect()
}

/// Collect all type references ($ref) from a schema
fn collect_schema_refs<'a>(schema: &'a Value, refs: &mut BTreeSet<&'a str>) {
    match schema {
        Value::Object(map) => {
            if let Some(ref_path) = map.get("$ref").and_then(|r| r.as_str())
                && let Some(type_name) = ref_to_type_name(ref_path)
            {
                refs.insert(type_name);
            }
            for value in map.values() {
                collect_schema_refs(value, refs);
            }
        }
        Value::Array(arr) => {
            for item in arr {
                collect_schema_refs(item, refs);
            }
        }
        _ => {}
    }
}

/// Convert a single JSON type string to Python type
fn json_type_to_python(ty: &str, schema: &Value, current_type: Option<&str>) -> String {
    match ty {
        "integer" => "int".to_string(),
        "number" => "float".to_string(),
        "boolean" => "bool".to_string(),
        "string" => "str".to_string(),
        "null" => "None".to_string(),
        "array" => {
            let item_type = schema
                .get("items")
                .map(|s| schema_to_python_type(s, current_type, None))
                .unwrap_or_else(|| "Any".to_string());
            format!("List[{}]", item_type)
        }
        "object" => {
            if let Some(add_props) = schema.get("additionalProperties") {
                let value_type = schema_to_python_type(add_props, current_type, None);
                return format!("dict[str, {}]", value_type);
            }
            "dict".to_string()
        }
        _ => "Any".to_string(),
    }
}

/// Convert JSON Schema to Python type.
///
/// - `current_type`: Used to detect and quote self-references for recursive types
/// - `quote_types`: Optional set of additional type names that should be quoted
fn schema_to_python_type(
    schema: &Value,
    current_type: Option<&str>,
    quote_types: Option<&BTreeSet<String>>,
) -> String {
    if let Some(all_of) = schema.get("allOf").and_then(|v| v.as_array()) {
        for item in all_of {
            let resolved = schema_to_python_type(item, current_type, quote_types);
            if resolved != "Any" {
                return resolved;
            }
        }
    }

    // Handle $ref
    if let Some(ref_path) = schema.get("$ref").and_then(|r| r.as_str()) {
        let type_name = ref_to_type_name(ref_path).unwrap_or("Any");
        if let Some(element) = rust_array_element_type(type_name) {
            return format!("List[{element}]");
        }
        // Quote self-references or types in quote_types set
        let should_quote =
            current_type == Some(type_name) || quote_types.is_some_and(|qt| qt.contains(type_name));
        if should_quote {
            return format!("\"{}\"", type_name);
        }
        return type_name.to_string();
    }

    // Handle enum (array of string values)
    if let Some(enum_values) = schema.get("enum").and_then(|e| e.as_array()) {
        let literals: Vec<String> = enum_values
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| format!("\"{}\"", s))
            .collect();
        if !literals.is_empty() {
            return format!("Literal[{}]", literals.join(", "));
        }
    }

    if let Some(ty) = schema.get("type") {
        if let Some(type_array) = ty.as_array() {
            let types: Vec<String> = type_array
                .iter()
                .filter_map(|t| t.as_str())
                .filter(|t| *t != "null")
                .map(|t| json_type_to_python(t, schema, current_type))
                .collect();
            let has_null = type_array.iter().any(|t| t.as_str() == Some("null"));

            if types.len() == 1 {
                let base_type = &types[0];
                return if has_null {
                    format!("Optional[{}]", base_type)
                } else {
                    base_type.clone()
                };
            } else if !types.is_empty() {
                let union = format!("Union[{}]", types.join(", "));
                return if has_null {
                    format!("Optional[{}]", union)
                } else {
                    union
                };
            }
        }

        if let Some(ty_str) = ty.as_str() {
            return json_type_to_python(ty_str, schema, current_type);
        }
    }

    if let Some(variants) = get_union_variants(schema) {
        let types: Vec<String> = variants
            .iter()
            .map(|v| schema_to_python_type(v, current_type, quote_types))
            .collect();
        let filtered: Vec<_> = types.iter().filter(|t| *t != "Any").collect();
        if !filtered.is_empty() {
            return format!(
                "Union[{}]",
                filtered
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        return format!("Union[{}]", types.join(", "));
    }

    // Check for format hint without type (common in OpenAPI)
    if let Some(format) = schema.get("format").and_then(|f| f.as_str()) {
        return match format {
            "int32" | "int64" => "int".to_string(),
            "float" | "double" => "float".to_string(),
            "date" | "date-time" => "str".to_string(),
            _ => "Any".to_string(),
        };
    }

    "Any".to_string()
}

/// Convert JS-style type to Python type (e.g., `Txid[]` -> `List[Txid]`, `integer` -> `int`).
pub(crate) fn js_type_to_python(js_type: &str) -> String {
    if let Some(inner) = js_type.strip_suffix("[]") {
        format!("List[{}]", js_type_to_python(inner))
    } else {
        match js_type {
            "integer" => "int".to_string(),
            "number" => "float".to_string(),
            "boolean" => "bool".to_string(),
            "string" => "str".to_string(),
            "null" => "None".to_string(),
            "Object" | "object" => "dict".to_string(),
            "*" => "Any".to_string(),
            _ => js_type.to_string(),
        }
    }
}
