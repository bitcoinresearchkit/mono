use std::mem;

use serde_json::{Map, Value, to_vec};

/// Compacts an OpenAPI spec JSON for LLM consumption: drops deprecated endpoints, `*Param`
/// schemas, examples, formats, constraints and secondary descriptions, and reduces schemas,
/// responses and parameters to their types.
pub(super) fn compact_json(mut spec: Value) -> Vec<u8> {
    if let Some(Value::Object(paths)) = spec.get_mut("paths") {
        paths.retain(|_, v| {
            if let Value::Object(path_obj) = v
                && let Some(Value::Object(get_obj)) = path_obj.get("get")
            {
                return get_obj.get("deprecated") != Some(&Value::Bool(true));
            }
            true
        });
    }

    if let Some(Value::Object(info)) = spec.get_mut("info") {
        info.remove("contact");
        info.remove("license");
    }

    if let Some(Value::Object(components)) = spec.get_mut("components")
        && let Some(Value::Object(schemas)) = components.get_mut("schemas")
    {
        schemas.retain(|name, _| !name.ends_with("Param"));
    }

    compact_value(&mut spec);
    to_vec(&spec).unwrap()
}

fn compact_value(value: &mut Value) {
    match value {
        Value::Object(obj) => {
            // The success response as `"returns": "Type"`; error responses are dropped.
            if let Some(Value::Object(responses)) = obj.remove("responses")
                && let Some(returns) = extract_return_type(&responses)
            {
                obj.insert("returns".to_string(), Value::String(returns));
            }

            // Per-endpoint tag arrays only, not the top-level tag definitions.
            if let Some(Value::Array(_)) = obj.get("tags") {
                obj.remove("tags");
            }
            // Complete here: the recursion below skips parameters.
            if let Some(Value::Array(params)) = obj.get_mut("parameters") {
                for param in params {
                    simplify_parameter(param);
                }
            }

            for key in ["style", "summary", "operationId", "example", "examples"] {
                obj.remove(key);
            }
            if let Some(Value::String(ref_path)) = obj.remove("$ref") {
                let type_name = ref_path.split('/').next_back().unwrap_or("any");
                obj.insert("type".to_string(), Value::String(type_name.to_string()));
            }

            if let Some(Value::Array(all_of)) = obj.remove("allOf")
                && all_of.len() == 1
                && let Some(Value::Object(inner)) = all_of.into_iter().next()
            {
                for (k, v) in inner {
                    obj.insert(k, v);
                }
            }

            if let Some(Value::Array(any_of)) = obj.remove("anyOf") {
                let types = union_types(any_of);
                if !types.is_empty() {
                    obj.insert("type".to_string(), Value::Array(types));
                }
            }

            // After the allOf flattening, which may bring them back.
            for key in ["format", "minimum", "maximum"] {
                obj.remove(key);
            }

            // Schema `required` arrays; parameters keep their boolean.
            if let Some(Value::Array(_)) = obj.get("required") {
                obj.remove("required");
            }

            if let Some(value) = obj.get_mut("type")
                && let Value::Array(arr) = value
                && arr.len() == 1
            {
                *value = arr.pop().unwrap();
            }

            if obj.contains_key("properties")
                && obj.get("type").and_then(Value::as_str) == Some("object")
            {
                obj.remove("type");
            }

            // Large enums become their plain type.
            if let Some(Value::Array(enum_values)) = obj.get("enum")
                && enum_values.len() > 40
            {
                obj.remove("enum");
            }

            if let Some(Value::String(desc)) = obj.get_mut("description") {
                *desc = trim_description(desc);
            }

            if let Some(Value::Object(props)) = obj.get_mut("properties") {
                simplify_properties(props);
            }

            for (key, v) in obj.iter_mut() {
                if key != "parameters" {
                    compact_value(v);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                compact_value(item);
            }
        }
        _ => {}
    }
}

/// Trim description to first paragraph and strip mempool.space endpoint links.
fn trim_description(desc: &str) -> String {
    let desc = if let Some(idx) = desc.find("*[Mempool.space docs]") {
        desc[..idx].trim()
    } else {
        desc
    };

    if let Some(idx) = desc.find("\n\n") {
        desc[..idx].trim().to_string()
    } else {
        desc.trim().to_string()
    }
}

/// The successful response: its JSON type, or its type and media type when it is not JSON.
fn extract_return_type(responses: &Map<String, Value>) -> Option<String> {
    let content = responses.get("200")?.get("content")?.as_object()?;
    if let Some(schema) = content
        .get("application/json")
        .and_then(|json| json.get("schema"))
    {
        return Some(schema_to_type_string(schema));
    }
    let (media_type, body) = content.iter().next()?;
    let media_type = media_type.split(';').next().unwrap_or(media_type);
    Some(match body.get("schema") {
        Some(schema) => format!("{} ({media_type})", schema_to_type_string(schema)),
        None => media_type.to_owned(),
    })
}

fn schema_to_type_string(schema: &Value) -> String {
    if let Some(Value::String(ref_path)) = schema.get("$ref") {
        return ref_path.split('/').next_back().unwrap_or("any").to_string();
    }
    if let Some(Value::String(t)) = schema.get("type") {
        if t == "array"
            && let Some(items) = schema.get("items")
        {
            return format!("array[{}]", schema_to_type_string(items));
        }
        return t.clone();
    }
    "any".to_string()
}

/// A parameter as its name, location, requirement and what a valid request needs from its
/// schema: type, enum, default, bounds and array items.
fn simplify_parameter(param: &mut Value) {
    let Value::Object(obj) = param else {
        return;
    };
    for key in ["description", "style", "example", "examples"] {
        obj.remove(key);
    }
    let Some(Value::Object(mut schema)) = obj.remove("schema") else {
        return;
    };
    for key in ["enum", "default", "minimum", "maximum"] {
        if let Some(value) = schema.remove(key)
            && !value.is_null()
        {
            obj.insert(key.to_owned(), value);
        }
    }
    if let Some(items) = schema.remove("items") {
        obj.insert("items".to_owned(), extract_type_from_schema(items));
    }
    obj.insert(
        "type".to_owned(),
        extract_type_from_schema(Value::Object(schema)),
    );
}

fn simple_type(schema: Value) -> Option<Value> {
    let Value::Object(mut obj) = schema else {
        return None;
    };
    if let Some(Value::String(path)) = obj.remove("$ref") {
        return Some(Value::String(
            path.rsplit('/').next().unwrap_or("any").to_owned(),
        ));
    }
    // A described reference is wrapped as `allOf: [{ $ref }]`.
    if let Some(Value::Array(mut all_of)) = obj.remove("allOf")
        && all_of.len() == 1
    {
        return simple_type(all_of.pop().unwrap());
    }
    obj.remove("type")
}

fn union_types(schemas: Vec<Value>) -> Vec<Value> {
    schemas.into_iter().filter_map(simple_type).collect()
}

fn extract_type_from_schema(mut schema: Value) -> Value {
    if let Value::Object(obj) = &mut schema
        && let Some(Value::Array(any_of)) = obj.remove("anyOf")
    {
        let mut types = union_types(any_of);
        return if types.len() == 1 {
            types.pop().unwrap()
        } else {
            Value::Array(types)
        };
    }
    simple_type(schema).unwrap_or_else(|| Value::String("any".to_owned()))
}

fn simplify_properties(props: &mut Map<String, Value>) {
    for value in props.values_mut() {
        if let Value::Object(obj) = value {
            *value = simplify_property_value(mem::take(obj));
        }
    }
}

fn simplify_property_value(mut obj: Map<String, Value>) -> Value {
    // Remove validation constraints, format, and examples
    for key in &[
        "default",
        "minItems",
        "maxItems",
        "uniqueItems",
        "minimum",
        "maximum",
        "format",
        "examples",
        "example",
        "description",
    ] {
        obj.remove(*key);
    }

    // Remove "items": true (means any type, not useful)
    if obj.get("items") == Some(&Value::Bool(true)) {
        obj.remove("items");
    }

    // Handle $ref - convert to type (runs before recursion would)
    if let Some(Value::String(ref_path)) = obj.remove("$ref") {
        let type_name = ref_path.split('/').next_back().unwrap_or("any");
        return Value::String(type_name.to_string());
    }

    // Handle single-item allOf - flatten and extract type
    if let Some(Value::Array(all_of)) = obj.remove("allOf")
        && all_of.len() == 1
        && let Some(Value::Object(inner)) = all_of.into_iter().next()
        && let Some(t) = simple_type(Value::Object(inner))
    {
        return t;
    }

    // Handle anyOf - flatten to type array (runs before recursion would)
    if let Some(Value::Array(any_of)) = obj.remove("anyOf") {
        let types = union_types(any_of);
        return Value::Array(types);
    }

    // If only "type" remains, return just the type value
    if obj.len() == 1
        && let Some(t) = obj.remove("type")
    {
        return t;
    }

    // Handle array with items
    if obj.get("type").and_then(Value::as_str) == Some("array")
        && let Some(items) = obj.get("items")
        && let Value::Object(items_obj) = items
        && items_obj.len() == 1
    {
        // Items can have either "type" or "$ref"
        if let Some(Value::String(item_type)) = items_obj.get("type") {
            return Value::String(format!("array[{}]", item_type));
        }
        if let Some(Value::String(ref_path)) = items_obj.get("$ref") {
            let type_name = ref_path.split('/').next_back().unwrap_or("any");
            return Value::String(format!("array[{}]", type_name));
        }
    }

    Value::Object(obj)
}
