use serde_json::{Value, json};

use crate::{
    Ty, TypeSchemas, extract_endpoints,
    generators::javascript::{api::generate_api_methods, client::generate_base_client, keep::keep_expr},
    parse_openapi_json,
};

/// Where the JavaScript client keeps keys as sent: untyped values (an OpenAPI document, series
/// values) and maps whole, everything else camelCased; a typed empty array keeps nothing, so a
/// union with it leaves its sibling's typed list converted.
#[test]
fn javascript_keeps_untyped_values_and_maps_as_sent() {
    let schemas: TypeSchemas = [
        (
            "SeriesData",
            json!({"type": "object", "properties": {
                "start": {"type": "integer"},
                "data": {"type": "array", "items": true},
            }}),
        ),
        ("Entry", json!({"type": "object", "properties": {"tx_id": {"type": "string"}}})),
        (
            "Info",
            json!({"type": "object", "properties": {
                "ancestors": {"type": "array", "items": {"$ref": "#/components/schemas/Entry"}},
            }}),
        ),
        (
            "Empty",
            json!({"type": "object", "properties": {"ancestors": {
                "type": "array", "items": {"$ref": "#/components/schemas/Entry"}, "maxItems": 0,
            }}}),
        ),
        (
            "Answer",
            json!({"anyOf": [
                {"$ref": "#/components/schemas/Info"},
                {"$ref": "#/components/schemas/Empty"},
            ]}),
        ),
        ("Rates", json!({"type": "object", "additionalProperties": {"type": "number"}})),
    ]
    .into_iter()
    .map(|(name, schema)| (name.to_owned(), schema))
    .collect();
    let keep = |schema: Value| keep_expr(&Ty::parse(&schema), &schemas);
    let named = |name: &str| keep(json!({"$ref": format!("#/components/schemas/{name}")}));
    assert_eq!(keep(json!(true)).as_deref(), Some("true"), "an untyped document");
    assert_eq!(named("SeriesData").as_deref(), Some(r#"{ "data": true }"#), "series values");
    assert_eq!(named("Rates").as_deref(), Some("true"), "a map");
    assert_eq!(named("Answer"), None, "typed lists, the empty one included");
}

/// The generated methods pass those rules on: a document's untyped body kept whole, series
/// responses (one or a bulk list) by the runtime's `_SERIES_KEEP`, which the base client emits
/// from `SeriesData`'s schema.
#[test]
fn javascript_methods_forward_their_keep_rules() {
    let ok = |schema: Value| json!({"200": {"description": "", "content": {"application/json": {"schema": schema}}}});
    let series = json!({"$ref": "#/components/schemas/SeriesData"});
    let spec = json!({
        "openapi": "3.1.0",
        "info": {"title": "test", "version": "1"},
        "paths": {
            "/openapi.json": {"get": {"operationId": "get_openapi", "responses": ok(json!(true))}},
            "/api/series/one": {"get": {"operationId": "get_series", "responses": ok(series.clone())}},
            "/api/series/bulk": {"get": {
                "operationId": "get_series_bulk",
                "responses": ok(json!({"type": "array", "items": series})),
            }},
        },
        "components": {"schemas": {"SeriesData": {"type": "object", "properties": {
            "start": {"type": "integer"},
            "data": {"type": "array", "items": true},
        }}}},
    });
    let (spec, schemas) = parse_openapi_json(&spec.to_string()).unwrap();
    let endpoints = extract_endpoints(&spec, &schemas).unwrap();
    let mut output = String::new();
    generate_base_client(&mut output, &schemas);
    let operations = crate::generators::operation::Operation::lower_all(&endpoints);
    generate_api_methods(&mut output, &operations, &schemas);
    let method = |name: &str| {
        let start = output.find(&format!("async {name}(")).expect(name);
        output[start..].split("\n  }\n").next().unwrap().to_owned()
    };
    assert!(output.contains(r#"const _SERIES_KEEP = { "data": true };"#));
    assert!(method("getOpenapi").contains("_readJson(true)"), "{}", method("getOpenapi"));
    for name in ["getSeries", "getSeriesBulk"] {
        assert!(method(name).contains("_readJson(_SERIES_KEEP)"), "{}", method(name));
    }
}
