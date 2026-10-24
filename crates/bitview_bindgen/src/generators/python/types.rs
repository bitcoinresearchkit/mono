use std::collections::BTreeSet;
use std::fmt::Write;

use serde_json::Value;

use crate::{
    Ty, TypeSchemas, collect_schema_refs, escape_python_keyword,
    generators::{MANUAL_GENERIC_TYPES, write_description},
    is_rust_concrete_generic, rust_array_element_type,
    topological::topological,
};

pub(crate) fn generate_type_definitions(output: &mut String, schemas: &TypeSchemas) {
    // Type aliases come first to avoid forward reference issues.
    let (type_aliases, typed_dicts): (Vec<_>, Vec<_>) = topological_sort_schemas(schemas)
        .into_iter()
        .filter(|name| !MANUAL_GENERIC_TYPES.contains(&name.as_str()))
        .filter(|name| rust_array_element_type(name).is_none())
        .filter(|name| !is_rust_concrete_generic(name))
        .partition(|name| schemas[name].get("properties").is_none());

    // Quote references to TypedDicts since they're defined after
    let typed_dict_set: BTreeSet<_> = typed_dicts.iter().cloned().collect();
    for name in type_aliases {
        let schema = &schemas[&name];
        let type_desc = schema.get("description").and_then(|d| d.as_str());
        let quote = |reference: &str| reference == name || typed_dict_set.contains(reference);
        let py_type = python_type(&Ty::parse(schema), &quote);
        if let Some(desc) = type_desc {
            write_description(output, desc, "# ", "#");
        }
        writeln!(output, "{} = {}", name, py_type).unwrap();
    }

    for name in typed_dicts {
        let schema = &schemas[&name];
        let type_desc = schema.get("description").and_then(|d| d.as_str());
        let props = schema
            .get("properties")
            .and_then(|p| p.as_object())
            .unwrap();

        // A property outside `required` can be absent from the JSON: required keys go in a base
        // class, optional ones in a `total=False` class.
        let required: BTreeSet<&str> = schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|r| r.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        let (required_props, optional_props): (Vec<_>, Vec<_>) = props
            .iter()
            .partition(|(prop_name, _)| required.contains(prop_name.as_str()));
        let quote = |reference: &str| reference == name;
        let field = |output: &mut String, (prop_name, prop_schema): (&String, &Value)| {
            let prop_type = python_type(&Ty::parse(prop_schema), &quote);
            writeln!(
                output,
                "    {}: {}",
                escape_python_keyword(prop_name),
                prop_type
            )
            .unwrap();
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
                    format!("{key:?}: {}", python_type(&Ty::parse(prop_schema), &quote))
                })
                .collect();
            let total = if required_props.is_empty() {
                ", total=False"
            } else {
                ""
            };
            writeln!(
                output,
                "{name} = TypedDict({name:?}, {{{}}}{total})\n",
                entries.join(", ")
            )
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

        let field_docs: Vec<(String, Option<&str>)> = props
            .iter()
            .map(|(prop_name, prop_schema)| {
                let safe_name = escape_python_keyword(prop_name);
                let desc = prop_schema.get("description").and_then(|d| d.as_str());
                (safe_name, desc)
            })
            .collect();
        let has_field_docs = field_docs.iter().any(|(_, d)| d.is_some());

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
                        writeln!(
                            output,
                            "        {}: {}",
                            field_name,
                            d.split_whitespace().collect::<Vec<_>>().join(" ")
                        )
                        .unwrap();
                    }
                }
            }
            writeln!(output, "    \"\"\"").unwrap();
        }

        let own = if optional_props.is_empty() {
            &required_props
        } else {
            &optional_props
        };
        for &prop in own {
            field(output, prop);
        }
        writeln!(output).unwrap();
    }
    writeln!(output).unwrap();
}

/// Schema names, each after the schemas it references (no forward references); names on or above
/// a reference cycle follow in name order. Self-references are quoted at emit time instead.
fn topological_sort_schemas(schemas: &TypeSchemas) -> Vec<String> {
    let names: Vec<&String> = schemas.keys().collect();
    let mut edges = Vec::new();
    for (dependent, schema) in schemas.values().enumerate() {
        let mut refs = BTreeSet::new();
        collect_schema_refs(schema, &mut refs);
        edges.extend(
            refs.into_iter()
                .filter_map(|name| names.binary_search_by(|key| key.as_str().cmp(name)).ok())
                .filter(|&dependency| dependency != dependent)
                .map(|dependency| (dependency, dependent)),
        );
    }
    let (order, stuck) = topological(names.len(), edges);
    order
        .into_iter()
        .chain(stuck)
        .map(|i| names[i].clone())
        .collect()
}

/// A type in Python syntax. `quote` names the references to quote (self-references, and
/// classes an alias would otherwise reference before their definition).
pub(crate) fn python_type(ty: &Ty, quote: &impl Fn(&str) -> bool) -> String {
    match ty {
        Ty::Any => "Any".to_owned(),
        Ty::Null => "None".to_owned(),
        Ty::Boolean => "bool".to_owned(),
        Ty::Integer(_) => "int".to_owned(),
        Ty::Number => "float".to_owned(),
        Ty::String => "str".to_owned(),
        Ty::Object => "dict".to_owned(),
        Ty::Map(value) => format!("dict[str, {}]", python_type(value, quote)),
        Ty::List(item) => format!("List[{}]", python_type(item, quote)),
        // JSON arrays decode to lists: a tuple is a list of its positions' types.
        Ty::Tuple(items) => {
            let mut types: Vec<String> = Vec::new();
            for ty in items.iter().map(|ty| python_type(ty, quote)) {
                if !types.contains(&ty) {
                    types.push(ty);
                }
            }
            match types.as_slice() {
                [ty] => format!("List[{ty}]"),
                _ => format!("List[Union[{}]]", types.join(", ")),
            }
        }
        Ty::Ref(name) if quote(name) => format!("\"{name}\""),
        Ty::Ref(name) => name.clone(),
        Ty::Literals(values) => {
            let values: Vec<String> = values.iter().map(|value| format!("\"{value}\"")).collect();
            format!("Literal[{}]", values.join(", "))
        }
        Ty::Nullable(ty) => format!("Optional[{}]", python_type(ty, quote)),
        Ty::Union(variants) => {
            let variants: Vec<String> = variants.iter().map(|ty| python_type(ty, quote)).collect();
            format!("Union[{}]", variants.join(", "))
        }
    }
}
