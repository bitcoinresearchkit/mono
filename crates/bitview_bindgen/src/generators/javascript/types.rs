use std::fmt::Write;

use crate::{
    Ty, TypeSchemas,
    generators::{MANUAL_GENERIC_TYPES, write_description},
    is_rust_concrete_generic, rust_array_element_type,
};

pub(crate) fn generate_type_definitions(output: &mut String, schemas: &TypeSchemas) {
    for (name, schema) in schemas.iter() {
        if MANUAL_GENERIC_TYPES.contains(&name.as_str())
            || rust_array_element_type(name).is_some()
            || is_rust_concrete_generic(name)
        {
            continue;
        }

        let alias = js_type(&Ty::parse(schema));

        let type_desc = schema.get("description").and_then(|d| d.as_str());

        if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
            let required = |prop_name: &str| {
                schema
                    .get("required")
                    .and_then(|r| r.as_array())
                    .is_some_and(|arr| arr.iter().any(|v| v.as_str() == Some(prop_name)))
            };
            writeln!(output, "/**").unwrap();
            if let Some(desc) = type_desc {
                write_description(output, desc, " * ", " *");
                writeln!(output, " *").unwrap();
            }
            let names = props
                .keys()
                .map(|key| js_property_name(&key.replace(['[', ']'], "")));
            if names.clone().all(|name| is_js_identifier(&name)) {
                writeln!(output, " * @typedef {{Object}} {}", name).unwrap();
                for ((prop_name, prop_schema), safe_name) in props.iter().zip(names) {
                    let prop_type = js_type(&Ty::parse(prop_schema));
                    let optional = if required(prop_name) { "" } else { "=" };
                    let prop_desc = prop_schema
                        .get("description")
                        .and_then(|d| d.as_str())
                        .map(|d| format!(" - {}", d))
                        .unwrap_or_default();
                    writeln!(
                        output,
                        " * @property {{{}{}}} {}{}",
                        prop_type, optional, safe_name, prop_desc
                    )
                    .unwrap();
                }
            } else {
                // JSDoc @property takes identifiers only: quote keys like `24h` in a type literal.
                for (prop_name, prop_schema) in props {
                    if let Some(desc) = prop_schema.get("description").and_then(|d| d.as_str()) {
                        writeln!(output, " * - `{}`: {}", prop_name, desc).unwrap();
                    }
                }
                let fields = props
                    .iter()
                    .zip(names)
                    .map(|((prop_name, prop_schema), safe_name)| {
                        let optional = if required(prop_name) { "" } else { "?" };
                        let prop_type = js_type(&Ty::parse(prop_schema));
                        if is_js_identifier(&safe_name) {
                            format!("{safe_name}{optional}: {prop_type}")
                        } else {
                            format!("\"{safe_name}\"{optional}: {prop_type}")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                writeln!(output, " * @typedef {{{{ {} }}}} {}", fields, name).unwrap();
            }
            writeln!(output, " */").unwrap();
        } else if let Some(desc) = type_desc {
            writeln!(output, "/**").unwrap();
            write_description(output, desc, " * ", " *");
            writeln!(output, " *").unwrap();
            writeln!(output, " * @typedef {{{}}} {}", alias, name).unwrap();
            writeln!(output, " */").unwrap();
        } else {
            writeln!(output, "/** @typedef {{{}}} {} */", alias, name).unwrap();
        }
    }
    writeln!(output).unwrap();
}

/// The name the client gives a wire field: camelCased by the runtime (`tx_count` → `txCount`;
/// `avgFee_0` and `24h` stay as they are). Map keys are data and stay as sent.
fn js_property_name(key: &str) -> String {
    let mut name = String::with_capacity(key.len());
    let mut chars = key.chars().peekable();
    while let Some(c) = chars.next() {
        match chars.peek() {
            Some(next) if c == '_' && next.is_ascii_lowercase() => {
                name.push(next.to_ascii_uppercase());
                chars.next();
            }
            _ => name.push(c),
        }
    }
    name
}

fn is_js_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// A type in JSDoc syntax (which has no `integer`, only `number`).
pub(crate) fn js_type(ty: &Ty) -> String {
    match ty {
        Ty::Any => "*".to_owned(),
        Ty::Null => "null".to_owned(),
        Ty::Boolean => "boolean".to_owned(),
        Ty::Integer(_) | Ty::Number => "number".to_owned(),
        Ty::String => "string".to_owned(),
        Ty::Object => "Object".to_owned(),
        Ty::Map(value) => format!("{{ [key: string]: {} }}", js_type(value)),
        Ty::List(item) => format!("{}[]", js_type(item)),
        Ty::Tuple(items) => {
            let items: Vec<String> = items.iter().map(js_type).collect();
            format!("[{}]", items.join(", "))
        }
        Ty::Ref(name) => name.clone(),
        Ty::Literals(values) => {
            let values: Vec<String> = values.iter().map(|value| format!("\"{value}\"")).collect();
            format!("({})", values.join("|"))
        }
        Ty::Nullable(ty) => format!("?{}", js_type(ty)),
        Ty::Union(variants) => {
            let variants: Vec<String> = variants.iter().map(js_type).collect();
            format!("({})", variants.join("|"))
        }
    }
}
