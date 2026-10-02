use serde_json::{from_value, to_value};

use super::*;

#[test]
fn parsing_and_serde_share_strict_calendar_validation() {
    for text in [
        "0000-01-01",
        "0001-01-01",
        "2000-02-29",
        "2024-02-29",
        "9999-12-31",
    ] {
        let date: Date = text.parse().unwrap();
        assert_eq!(date.to_string(), text);
        assert_eq!(from_value::<Date>(text.into()).unwrap(), date);
        assert_eq!(to_value(date).unwrap(), text);
    }
    for text in [
        "2026-02-29",
        "2026-02-30",
        "2026-02-31",
        "2023-02-29",
        "1900-02-29",
        "2026-04-31",
        "2026-00-01",
        "2026-13-01",
        "2026-01-00",
        "2026-01-32",
        "2026_01_01",
        "2026-+1-01",
        "+026-01-01",
        "123é01-01",
        "2026-1-01",
        "",
        "2026-01-01extra",
    ] {
        assert!(text.parse::<Date>().is_err(), "{text}");
        assert!(from_value::<Date>(text.into()).is_err(), "{text}");
    }
}
