use schemars::Schema;
use serde_json::Value;

/// A response struct's schema transform, for one that serializes every field, `None` as null:
/// every key is required, and an `Option` one stays nullable. Schemars' deserialize contract,
/// which the OpenAPI document uses, would leave an `Option` field optional.
pub(crate) fn every_key_required(schema: &mut Schema) {
    let object = schema.ensure_object();
    let names = object
        .get("properties")
        .and_then(Value::as_object)
        .expect("every_key_required is for a struct's schema")
        .keys()
        .cloned()
        .map(Value::String)
        .collect::<Vec<_>>();
    let Value::Array(required) = object
        .entry("required")
        .or_insert_with(|| Value::Array(Vec::new()))
    else {
        unreachable!("a schema's required keys are an array");
    };
    for name in names {
        if !required.contains(&name) {
            required.push(name);
        }
    }
}
