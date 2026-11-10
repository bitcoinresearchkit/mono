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
    generate_api_methods(&mut output, &endpoints, &schemas);
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


/// The status declaration, not a path or nullable JSON body, selects the optional decoder.
/// Execute generated methods against mock HTTP transports in both interpreted clients, then
/// check the Rust signatures and the MCP object-rooted contract from the same OpenAPI fixture.
#[test]
fn declared_no_content_is_nullable_across_clients() {
    use std::{fs, process::Command};
    use crate::generators::{python, rust, llm::manifest::generate_tool_manifest};

    let body = json!({"$ref": "#/components/schemas/Reply"});
    let responses = |optional: bool| {
        let mut responses = json!({"200": {"content": {"application/json": {"schema": body}}}});
        if optional { responses["204"] = json!({"description": "Absent; no body"}); }
        responses["503"] = json!({"description": "Unavailable"});
        responses
    };
    let document = json!({"openapi":"3.1.0", "info":{"title":"test","version":"1"},
        "paths": {
            "/optional": {
                "get":{"operationId":"get_optional","responses":responses(true)},
                "post":{"operationId":"post_optional","x-mcp-ignore":true,"responses":responses(true),
                    "requestBody":{"required":true,"content":{"text/plain":{"schema":{"type":"string"}}}}}
            },
            "/ordinary":{"get":{"operationId":"get_ordinary","responses":responses(false)}},
            "/nullable":{"get":{"operationId":"get_nullable","responses":{"200":{"content":{
                "application/json":{"schema":{"type":["string","null"]}}}}}}},
            "/query/{id}":{"get":{"operationId":"get_query","responses":responses(true),"parameters":[
                {"name":"id","in":"path","required":true,"schema":{"type":"string"}},
                {"name":"q","in":"query","schema":{"type":"string"}}
            ]}}
        },
        "components":{"schemas":{"Reply":{"type":"object","properties":{
            "tx_id":{"type":"string"}, "rates":{"type":"object","additionalProperties":{"type":"number"}}
        }}}}
    });
    let (spec, schemas) = parse_openapi_json(&document.to_string()).unwrap();
    let endpoints = extract_endpoints(&spec, &schemas).unwrap();
    let endpoint = |name: &str| endpoints.iter().find(|e| e.operation_id == name).unwrap();
    assert!(endpoint("get_optional").no_content);
    assert!(!endpoint("get_ordinary").no_content);
    assert!(!endpoint("get_nullable").no_content, "nullable JSON does not declare a 204");
    assert_eq!(endpoint("get_optional").response_ty(), Some(Ty::Nullable(Box::new(Ty::Ref("Reply".into())))));
    assert_eq!(endpoint("get_optional").json_response_schema, Some(json!({"anyOf":[body,{"type":"null"}]})));
    let nullable = Ty::Nullable(Box::new(Ty::String));
    assert_eq!(nullable.clone().nullable(), nullable, "never Option<Option<T>>");

    let mut rust = String::new();
    rust::api::generate_main_client(&mut rust, &endpoints);
    assert!(rust.contains("pub fn get_optional(&self) -> Result<Option<Reply>>"));
    assert!(rust.contains("self.base.get_optional_json(&format!(\"/optional\"))"));
    assert!(rust.contains("pub fn get_ordinary(&self) -> Result<Reply>"));
    assert!(rust.contains("self.base.get_json(&format!(\"/ordinary\"))"));

    let mut javascript = String::new();
    generate_base_client(&mut javascript, &schemas);
    javascript.push_str("\nclass FixtureClient extends BitviewClientBase {\n");
    generate_api_methods(&mut javascript, &endpoints, &schemas);
    javascript.push_str("}\n");
    assert!(javascript.contains("Promise<Frozen<?Reply>>"));
    assert!(javascript.contains("_readJson({ \"rates\": true }, true)"));
    javascript.push_str(include_str!("optional_response.js"));

    let mut python = String::from("from __future__ import annotations\nfrom typing import *\nfrom http.client import HTTPException, HTTPConnection, HTTPSConnection\nfrom urllib.parse import quote, urlencode, urlparse\nimport json\n");
    python.push_str(include_str!("generators/python/runtime.py").split("_MASK_64 =").next().unwrap());
    python.push_str("\nclass FixtureClient(BitviewClientBase):\n");
    python::api::generate_api_methods(&mut python, &endpoints);
    assert!(python.contains("def get_optional(self) -> Optional[Reply]:"));
    assert!(python.contains("return self.get_json('/ordinary')"));
    assert!(python.contains("return self.get_json(path, no_content=True)"));
    python.push_str(include_str!("optional_response.py"));

    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("manifest.json");
    generate_tool_manifest(&endpoints, &schemas, &manifest_path).unwrap();
    let manifest: Value = serde_json::from_str(&fs::read_to_string(manifest_path).unwrap()).unwrap();
    let operation = |name: &str| manifest["operations"].as_array().unwrap().iter()
        .find(|op| op["tool"]["name"] == name).unwrap();
    assert!(operation("get_optional")["tool"].get("outputSchema").is_none());
    assert_eq!(operation("get_ordinary")["tool"]["outputSchema"]["type"], "object");

    for (command, extension, source, marker) in [
        ("bun", "js", javascript, "FINISHED 19 JavaScript mock-HTTP runtime cases"),
        ("python3", "py", python, "FINISHED 18 Python mock-HTTP runtime cases"),
    ] {
        let path = dir.path().join(format!("client.{extension}"));
        fs::write(&path, source).unwrap();
        let output = Command::new(command).arg(path).output().unwrap();
        assert!(output.status.success(), "{command}: {}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        assert!(String::from_utf8_lossy(&output.stdout).contains(marker));
    }

    // A bodyless-only or other-format declaration is refused instead of misgenerated.
    for content in [json!({}), json!({"text/plain":{"schema":{"type":"string"}}}),
        json!({"application/json":{"schema":{}},"text/csv":{"schema":{"type":"string"}}})]
    {
        let mut invalid = document.clone();
        invalid["paths"]["/optional"]["get"]["responses"]["200"]["content"] = content;
        let (spec, schemas) = parse_openapi_json(&invalid.to_string()).unwrap();
        assert!(extract_endpoints(&spec, &schemas).is_err());
    }
}
