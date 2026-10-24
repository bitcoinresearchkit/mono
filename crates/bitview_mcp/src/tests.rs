use serde_json::json;

use crate::manifest::Catalog;

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
    assert!(check("get_transaction_times", json!({ "txId[]": [txid] })).is_ok());
    assert!(check("get_transaction_times", json!({ "txId[]": [] })).is_err());
    assert!(
        check(
            "get_transaction_times",
            json!({ "txId[]": vec![txid; 101] })
        )
        .is_err()
    );
    assert!(
        check(
            "get_address_hash_prefix_matches",
            json!({ "address_type": "op_return", "prefix": "ab" })
        )
        .is_err()
    );
}
