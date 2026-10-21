use std::mem;

use serde_json::{Map, Value, to_vec};

/// Compacts an OpenAPI spec JSON for LLM consumption: drops paths whose GET is deprecated,
/// `*Param` schemas, examples, formats and secondary descriptions, reduces schemas and responses
/// to their types, and parameters to their type, enum, default and bounds. Walks the document's
/// known sections; every schema goes through [`compact_schema`].
pub(super) fn compact_json(mut spec: Value) -> Vec<u8> {
    if let Some(Value::Object(info)) = spec.get_mut("info") {
        info.shift_remove("contact");
        info.shift_remove("license");
        trim_description(info);
    }

    if let Some(Value::Array(tags)) = spec.get_mut("tags") {
        tags.iter_mut()
            .filter_map(Value::as_object_mut)
            .for_each(trim_description);
    }

    if let Some(Value::Object(paths)) = spec.get_mut("paths") {
        paths.retain(|_, path| path.pointer("/get/deprecated") != Some(&Value::Bool(true)));
        paths
            .values_mut()
            .filter_map(Value::as_object_mut)
            .flat_map(|path| path.values_mut())
            .filter_map(Value::as_object_mut)
            .for_each(compact_operation);
    }

    if let Some(Value::Object(schemas)) = spec.pointer_mut("/components/schemas") {
        schemas.retain(|name, _| !name.ends_with("Param"));
        for schema in schemas.values_mut() {
            *schema = compact_component(mem::take(schema));
        }
    }

    to_vec(&spec).unwrap()
}

/// An operation as its trimmed description, parameters and `"returns": "Type"`; error
/// responses are dropped.
fn compact_operation(operation: &mut Map<String, Value>) {
    for key in ["tags", "summary", "operationId"] {
        operation.shift_remove(key);
    }
    trim_description(operation);
    if let Some(Value::Array(params)) = operation.get_mut("parameters") {
        params
            .iter_mut()
            .filter_map(Value::as_object_mut)
            .for_each(compact_parameter);
    }
    if let Some(Value::Object(responses)) = operation.shift_remove("responses")
        && let Some(returns) = return_type(responses)
    {
        operation.insert("returns".to_owned(), Value::String(returns));
    }
}

/// A parameter as its name, location, requirement and what a valid request needs from its
/// schema: type, enum, default and bounds.
fn compact_parameter(param: &mut Map<String, Value>) {
    for key in ["description", "style", "example", "examples"] {
        param.shift_remove(key);
    }
    let Some(Value::Object(mut schema)) = param.shift_remove("schema") else {
        return;
    };
    for key in ["enum", "default", "minimum", "maximum"] {
        if let Some(value) = schema.shift_remove(key)
            && !value.is_null()
        {
            param.insert(key.to_owned(), value);
        }
    }
    param.insert("type".to_owned(), compact_schema(Value::Object(schema)));
}

/// The successful response's type, followed by its media type when it is not JSON.
fn return_type(mut responses: Map<String, Value>) -> Option<String> {
    let mut success = responses.shift_remove("200")?;
    let content = success.get_mut("content")?.as_object_mut()?;
    if let Some(schema) = content
        .get_mut("application/json")
        .and_then(|json| json.get_mut("schema"))
    {
        return Some(type_name(schema.take()));
    }
    let (media_type, body) = content.iter_mut().next()?;
    let media_type = media_type.split(';').next().unwrap();
    Some(match body.get_mut("schema") {
        Some(schema) => format!("{} ({media_type})", type_name(schema.take())),
        None => media_type.to_owned(),
    })
}

/// A component schema: its trimmed description, then its compact form (a bare type as `type`).
fn compact_component(schema: Value) -> Value {
    let mut component = Map::new();
    if let Some(description) = schema.get("description").and_then(Value::as_str) {
        component.insert(
            "description".to_owned(),
            Value::String(trimmed(description)),
        );
    }
    match compact_schema(schema) {
        Value::Object(fields) => component.extend(fields),
        compact => {
            component.insert("type".to_owned(), compact);
        }
    }
    Value::Object(component)
}

/// A schema as the type a reader needs: a referenced schema's name, a JSON type, `array[T]`, a
/// tuple `tuple[T1, T2]`, a union `[T, ...]`, or, when there is more to say, an object of its `type`, `items`, small
/// `enum`, `properties` and map values (`additionalProperties`), each compacted the same way.
/// Everything else (descriptions, examples, formats, patterns, bounds, `required`) is dropped.
fn compact_schema(schema: Value) -> Value {
    let mut schema = match schema {
        Value::Object(schema) => schema,
        Value::Bool(true) => return Value::String("any".to_owned()),
        other => return other,
    };
    if let Some(Value::String(path)) = schema.get("$ref") {
        return Value::String(path.rsplit('/').next().unwrap().to_owned());
    }
    // A described reference is wrapped as `allOf: [{ $ref }]`.
    if let Some(Value::Array(all_of)) = schema.get_mut("allOf")
        && all_of.len() == 1
    {
        return compact_schema(all_of.pop().unwrap());
    }
    if let Some(Value::Array(members)) = schema
        .shift_remove("anyOf")
        .or_else(|| schema.shift_remove("oneOf"))
    {
        return union(members.into_iter().map(compact_schema));
    }

    let mut compact = Map::new();
    let items = schema.shift_remove("items").map(compact_schema);
    // A tuple of named item types reads as `tuple[T1, T2, ...]`.
    let tuple = match schema.shift_remove("prefixItems") {
        Some(Value::Array(members)) => {
            let members: Vec<Value> = members.into_iter().map(compact_schema).collect();
            members.iter().all(Value::is_string).then(|| {
                let names: Vec<&str> = members.iter().filter_map(Value::as_str).collect();
                Value::String(format!("tuple[{}]", names.join(", ")))
            })
        }
        _ => None,
    };
    if let Some(mut ty) = schema.shift_remove("type") {
        if let Some(tuple) = tuple
            && ty == "array"
        {
            ty = tuple;
        }
        // An array of a named item type reads as `array[T]`.
        if let Some(Value::String(item)) = &items {
            let array_of = Value::String(format!("array[{item}]"));
            match &mut ty {
                Value::Array(types) => types
                    .iter_mut()
                    .filter(|ty| **ty == "array")
                    .for_each(|ty| *ty = array_of.clone()),
                ty if *ty == "array" => *ty = array_of,
                _ => {}
            }
        }
        compact.insert("type".to_owned(), single(ty));
    }
    if let Some(items @ (Value::Object(_) | Value::Array(_))) = items {
        compact.insert("items".to_owned(), items);
    }
    if let Some(Value::Array(values)) = schema.shift_remove("enum")
        && values.len() <= 40
    {
        compact.insert("enum".to_owned(), Value::Array(values));
    }
    if let Some(Value::Object(properties)) = schema.shift_remove("properties") {
        let properties = properties
            .into_iter()
            .map(|(name, property)| (name, compact_schema(property)))
            .collect();
        compact.insert("properties".to_owned(), Value::Object(properties));
        // Properties imply an object.
        if compact.get("type").is_some_and(|ty| ty == "object") {
            compact.shift_remove("type");
        }
    }
    if let Some(values @ Value::Object(_)) = schema.shift_remove("additionalProperties") {
        compact.insert("additionalProperties".to_owned(), compact_schema(values));
    }

    // A lone type stands for the whole schema.
    match compact.len() {
        0 => Value::String("any".to_owned()),
        1 if compact.contains_key("type") => compact.shift_remove("type").unwrap(),
        _ => Value::Object(compact),
    }
}

/// The distinct member types of a union, in order; a lone one stands alone.
fn union(members: impl Iterator<Item = Value>) -> Value {
    let mut types = Vec::new();
    for member in members {
        let member = match member {
            Value::Array(nested) => nested,
            member => vec![member],
        };
        for ty in member {
            if !types.contains(&ty) {
                types.push(ty);
            }
        }
    }
    single(Value::Array(types))
}

/// A one-element type list as its element.
fn single(ty: Value) -> Value {
    match ty {
        Value::Array(mut types) if types.len() == 1 => types.pop().unwrap(),
        ty => ty,
    }
}

/// A schema's compact form as text.
fn type_name(schema: Value) -> String {
    match compact_schema(schema) {
        Value::String(name) => name,
        compact => compact.to_string(),
    }
}

/// A description reduced to nothing (only the mempool.space docs link) is dropped.
fn trim_description(object: &mut Map<String, Value>) {
    let Some(Value::String(description)) = object.get_mut("description") else {
        return;
    };
    *description = trimmed(description);
    if description.is_empty() {
        object.shift_remove("description");
    }
}

/// A description's first paragraph, without the mempool.space docs link.
fn trimmed(description: &str) -> String {
    let description = description
        .find("*[Mempool.space docs]")
        .map_or(description, |idx| description[..idx].trim());
    description.split("\n\n").next().unwrap().trim().to_owned()
}
