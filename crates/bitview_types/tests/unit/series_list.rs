use serde_json::{from_value, json};

use super::*;

#[test]
fn normalized_count_is_bounded_for_strings_and_arrays() {
    for separator in [",", " ", "+"] {
        for count in [MAX_VECS, MAX_VECS + 1] {
            let names = vec!["price_close"; count].join(separator);
            for value in [Value::String(names.clone()), json!([names])] {
                let parsed = from_value::<SeriesList>(value);
                assert_eq!(
                    parsed.is_ok(),
                    count == MAX_VECS,
                    "count={count} separator={separator}"
                );
                if let Ok(parsed) = parsed {
                    assert_eq!(parsed.len(), count);
                }
            }
        }
    }
}

#[test]
fn decoded_string_bytes_are_bounded_before_normalization() {
    for prefix in [
        "a".repeat(MAX_STRING_SIZE),
        "é".repeat(MAX_STRING_SIZE / 2),
        "!".repeat(MAX_STRING_SIZE),
    ] {
        for extra in ["", "a"] {
            let text = format!("{prefix}{extra}");
            for value in [
                Value::String(text.clone()),
                json!([text]),
                json!([prefix, extra]),
            ] {
                assert_eq!(from_value::<SeriesList>(value).is_ok(), extra.is_empty());
            }
        }
    }
}
