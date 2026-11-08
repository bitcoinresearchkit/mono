use serde::de::{DeserializeOwned, IntoDeserializer, value::Error};

use crate::de_plain;

fn plain<T: DeserializeOwned>(text: &str) -> Option<T> {
    de_plain::<_, T>(IntoDeserializer::<Error>::into_deserializer(text)).ok()
}

/// A URL's number has one spelling, whatever the type it's read into.
#[test]
fn plain_numbers() {
    assert_eq!(plain::<u32>("0"), Some(0));
    assert_eq!(plain::<u32>("600"), Some(600));
    assert_eq!(plain::<Option<u32>>("600"), Some(Some(600)));
    for spelling in ["0600", "00", "+600", "-1", " 600", "600 ", "6e2", "0x258", "", "4294967296"] {
        assert_eq!(plain::<u32>(spelling), None, "`{spelling}`");
    }
    assert_eq!(plain::<Option<u32>>("null"), None);
}
