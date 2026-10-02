use serde_json::{from_str, to_vec};

use super::{Value, reserialize_json};

#[test]
fn reused_json_buffer_preserves_value_bytes_and_parse_errors() {
    for raw in [
        "1231007105",
        "1e20",
        "340282366920938463463374607431768211455",
        "null",
        "true",
        "[1e20, null]",
        r#"{"b":"\u0061","a":-0}"#,
    ] {
        let value: Value = from_str(raw).unwrap();
        assert_eq!(
            reserialize_json(raw.as_bytes().to_vec()).unwrap(),
            to_vec(&value).unwrap()
        );
    }
    for raw in [b"".as_slice(), b"NaN", b"inf", b"{", b"\xff"] {
        assert!(reserialize_json(raw.to_vec()).is_err());
    }
}
