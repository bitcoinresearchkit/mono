//! Python type definitions generation.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use serde_json::Value;

use crate::{
    TypeSchemas, escape_python_keyword, generators::MANUAL_GENERIC_TYPES, get_union_variants,
    is_rust_concrete_generic, ref_to_type_name, rust_array_element_type,
};

/// Generate type definitions from schemas.
pub fn generate_type_definitions(output: &mut String, schemas: &TypeSchemas) {
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
            for line in desc.lines() {
                writeln!(output, "# {}", line).unwrap();
            }
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

        writeln!(output, "class {}(TypedDict):", name).unwrap();

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
                        writeln!(output, "        {}: {}", field_name, d).unwrap();
                    }
                }
            }
            writeln!(output, "    \"\"\"").unwrap();
        }

        for (prop_name, prop_schema) in props {
            let prop_type = schema_to_python_type(prop_schema, Some(&name), None);
            let safe_name = escape_python_keyword(prop_name);
            writeln!(output, "    {}: {}", safe_name, prop_type).unwrap();
        }
        writeln!(output).unwrap();
    }
    writeln!(output).unwrap();
}

/// Topologically sort schema names so dependencies come before dependents (avoids forward references).
/// Types that reference other types (via $ref) must be defined after their dependencies.
fn topological_sort_schemas(schemas: &TypeSchemas) -> Vec<String> {
    // Build dependency graph
    let mut deps: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (name, schema) in schemas.iter() {
        let mut type_deps = BTreeSet::new();
        collect_schema_refs(schema, &mut type_deps);
        // Only keep deps that are in our schemas, and drop self-references
        // (handled at emit time by quoting via current_type)
        type_deps.retain(|d| schemas.contains_key(d) && d != name);
        deps.insert(name.clone(), type_deps);
    }

    // Kahn's algorithm for topological sort
    let mut in_degree: BTreeMap<String, usize> = BTreeMap::new();
    for name in schemas.keys() {
        in_degree.insert(name.clone(), 0);
    }
    for type_deps in deps.values() {
        for dep in type_deps {
            *in_degree.entry(dep.clone()).or_insert(0) += 1;
        }
    }

    // Start with types that have no dependents (are not referenced by others)
    let mut queue: Vec<String> = in_degree
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(name, _)| name.clone())
        .collect();
    queue.sort(); // Deterministic order

    let mut result = Vec::new();
    while let Some(name) = queue.pop() {
        result.push(name.clone());
        if let Some(type_deps) = deps.get(&name) {
            for dep in type_deps {
                if let Some(count) = in_degree.get_mut(dep) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        queue.push(dep.clone());
                        queue.sort(); // Keep sorted for determinism
                    }
                }
            }
        }
    }

    // Reverse so dependencies come first
    result.reverse();

    // Add any types that weren't processed (e.g., due to circular refs or other edge cases)
    let result_set: BTreeSet<_> = result.iter().cloned().collect();
    let mut missing: Vec<_> = schemas
        .keys()
        .filter(|k| !result_set.contains(*k))
        .cloned()
        .collect();
    missing.sort();
    result.extend(missing);

    result
}

/// Collect all type references ($ref) from a schema
fn collect_schema_refs(schema: &Value, refs: &mut BTreeSet<String>) {
    match schema {
        Value::Object(map) => {
            if let Some(ref_path) = map.get("$ref").and_then(|r| r.as_str())
                && let Some(type_name) = ref_to_type_name(ref_path)
            {
                refs.insert(type_name.to_string());
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
pub fn schema_to_python_type(
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

/// Convert JS-style type to Python type (e.g., "Txid[]" -> "List[Txid]", "integer" -> "int")
pub fn js_type_to_python(js_type: &str) -> String {
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
