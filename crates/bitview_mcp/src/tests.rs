use serde_json::json;

use crate::manifest::{Catalog, Pattern};

#[test]
fn argument_bounds() {
    let catalog = Catalog::embedded();
    let check = |tool: &str, arguments| {
        catalog
            .operation(tool)
            .unwrap()
            .validate_arguments(&arguments)
    };
    let txid = "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16";

    assert!(check("list_series", json!({ "per_page": 1000 })).is_ok());
    assert!(check("list_series", json!({ "per_page": 0 })).is_err());
    assert!(check("list_series", json!({ "per_page": 1001 })).is_err());
    assert!(check("get_transaction_times", json!({ "txids": [txid] })).is_ok());
    assert!(check("get_transaction_times", json!({ "txids": [] })).is_err());
    assert!(check("get_transaction_times", json!({ "txids": ["XYZ"] })).is_err());
    // The `txids` argument goes out as mempool.space's `txId[]`.
    let url = crate::upstream::Upstream::new("http://localhost:3110".to_owned())
        .prepare(
            catalog.operation("get_transaction_times").unwrap(),
            &json!({ "txids": [txid] }),
        )
        .unwrap();
    assert!(
        url.ends_with(&format!("/api/v1/transaction-times?txId%5B%5D={txid}")),
        "{url}"
    );
    assert!(
        catalog
            .operation("get_block_tip_height")
            .unwrap()
            .text_is_number()
    );
    assert!(
        !catalog
            .operation("get_block_tip_hash")
            .unwrap()
            .text_is_number()
    );
    assert!(check("get_transaction_times", json!({ "txids": vec![txid; 101] })).is_err());
    assert!(
        check(
            "get_address_hash_prefix_matches",
            json!({ "address_type": "op_return", "prefix": "ab" })
        )
        .is_err()
    );
}

/// Every input `pattern` the manifest carries compiles, so the bridge checks each one.
#[test]
fn input_patterns() {
    fn patterns<'a>(schema: &'a serde_json::Value, found: &mut Vec<&'a str>) {
        match schema {
            serde_json::Value::Object(object) => {
                if let Some(pattern) = object.get("pattern").and_then(|p| p.as_str()) {
                    found.push(pattern);
                }
                object.values().for_each(|value| patterns(value, found));
            }
            serde_json::Value::Array(values) => {
                values.iter().for_each(|value| patterns(value, found))
            }
            _ => {}
        }
    }
    let catalog = Catalog::embedded();
    let mut found = Vec::new();
    let schemas: Vec<serde_json::Value> = catalog
        .tools()
        .iter()
        .map(|tool| serde_json::Value::Object((*tool.input_schema).clone()))
        .collect();
    schemas
        .iter()
        .for_each(|schema| patterns(schema, &mut found));
    assert!(!found.is_empty());
    for pattern in found {
        Pattern::compile(pattern).unwrap();
    }

    let hash = Pattern::compile("^[0-9a-f]{64}$").unwrap();
    assert!(hash.matches(&"ab".repeat(32)));
    assert!(!hash.matches(&"AB".repeat(32)) && !hash.matches(&"ab".repeat(31)));
    let date = Pattern::compile(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    assert!(date.matches("2024-01-01") && !date.matches("2024-1-01"));
    let name = Pattern::compile("^[a-z0-9_]+$").unwrap();
    assert!(name.matches("price_close") && !name.matches("") && !name.matches("Price"));
    let prefix = Pattern::compile("^[0-9a-f]{1,16}$").unwrap();
    assert!(prefix.matches("a") && !prefix.matches(&"a".repeat(17)));
    assert!(Pattern::compile("^([0-9a-f]{2})*$").is_err());
}
