#[cfg(feature = "storage")]
use std::fmt::Result;

use std::fmt;

use derive_more::Deref;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::U8x20;

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable};

#[derive(Debug, Clone, Deref, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Bytes))]
pub struct P2WPKHBytes(U8x20);

impl From<&[u8]> for P2WPKHBytes {
    #[inline]
    fn from(value: &[u8]) -> Self {
        Self(U8x20::from(value))
    }
}

impl From<U8x20> for P2WPKHBytes {
    #[inline]
    fn from(value: U8x20) -> Self {
        Self(value)
    }
}

impl fmt::Display for P2WPKHBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[cfg(feature = "storage")]
impl Formattable for P2WPKHBytes {
    fn write_to(&self, buf: &mut Vec<u8>) {
        use std::io::Write;
        write!(buf, "{self}").unwrap();
    }

    fn fmt_csv(&self, f: &mut String) -> Result {
        let start = f.len();
        self.fmt_into(f);
        if f.as_bytes()[start..].contains(&b',') {
            f.insert(start, '"');
            f.push('"');
        }
        Ok(())
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}
