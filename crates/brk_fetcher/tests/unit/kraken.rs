use serde_json::{json, to_value};

use super::*;

#[test]
fn parsing_preserves_order_duplicates_and_errors() {
    let response = json!({"result": {"XXBTZUSD": [
        [2, "2", "3", "1", "2"],
        null,
        [1, "1", "2", "0.5", "1"],
        [2, "4", "5", "3", "4"]
    ]}});
    let map = Kraken::parse_ohlc_response(&response).unwrap();
    assert_eq!(
        map.keys().copied().collect::<Vec<_>>(),
        [Timestamp::new(1), Timestamp::new(2)]
    );
    assert_eq!(
        to_value(&map[&Timestamp::new(2)]).unwrap(),
        json!([400, 500, 300, 400])
    );
    let daily = Kraken::parse_date_ohlc_response(&response).unwrap();
    assert_eq!(daily.len(), 1);
    assert_eq!(
        to_value(daily.values().next().unwrap()).unwrap(),
        json!([400, 500, 300, 400])
    );
    assert!(
        Kraken::parse_ohlc_response(&json!({"result": {"XXBTZUSD": []}}))
            .unwrap()
            .is_empty()
    );
    for bad in [json!(null), json!({}), json!({"result": {"XXBTZUSD": 1}})] {
        assert!(
            matches!(Kraken::parse_ohlc_response(&bad), Err(Error::Parse(message)) if message == "Invalid Kraken response format")
        );
    }
}
