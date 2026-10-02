use std::{borrow::Cow, str};

use byteview::ByteView;
use derive_more::Deref;
use serde::Serialize;
#[cfg(feature = "storage")]
use vecdb::PrintableIndex;

#[derive(Default, Debug, Deref, Clone, Serialize)]
pub struct StoredString(String);

impl From<String> for StoredString {
    #[inline]
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<Cow<'_, str>> for StoredString {
    #[inline]
    fn from(value: Cow<'_, str>) -> Self {
        Self(value.to_string())
    }
}

impl From<ByteView> for StoredString {
    #[inline]
    fn from(value: ByteView) -> Self {
        let bytes = &*value;
        Self(String::from_utf8_lossy(bytes).into_owned())
    }
}

impl From<StoredString> for ByteView {
    #[inline]
    fn from(value: StoredString) -> Self {
        Self::from(&value)
    }
}

impl From<&StoredString> for ByteView {
    #[inline]
    fn from(value: &StoredString) -> Self {
        Self::new(value.as_bytes())
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for StoredString {
    fn to_string() -> &'static str {
        "string"
    }
    fn to_possible_strings() -> &'static [&'static str] {
        &["string"]
    }
}
