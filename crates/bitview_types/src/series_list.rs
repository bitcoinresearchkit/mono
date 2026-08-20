use std::{fmt, mem};

use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, de::Error as _};
use serde_json::Value;

use super::SeriesName;

/// Comma-separated list of series names
#[derive(Debug, Deref, JsonSchema)]
#[schemars(
    with = "String",
    example = &"date,price_close",
    example = &"price_close",
    example = &"price_close,market_cap",
    example = &"realized_price,market_cap,mvrv"
)]
pub struct SeriesList(Vec<SeriesName>);

const MAX_VECS: usize = 32;
const MAX_STRING_SIZE: usize = 64 * MAX_VECS;

impl From<SeriesName> for SeriesList {
    #[inline]
    fn from(series: SeriesName) -> Self {
        Self(vec![series])
    }
}

impl From<String> for SeriesList {
    #[inline]
    fn from(value: String) -> Self {
        Self::from(SeriesName::from(value))
    }
}

impl<'a> From<Vec<&'a str>> for SeriesList {
    #[inline]
    fn from(value: Vec<&'a str>) -> Self {
        Self(value.into_iter().map(SeriesName::from).collect())
    }
}

impl<'de> Deserialize<'de> for SeriesList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        if let Some(str) = value.as_str() {
            if str.len() <= MAX_STRING_SIZE {
                Ok(Self(
                    sanitize(str.split(",").map(|s| s.to_string()))
                        .into_iter()
                        .map(SeriesName::from)
                        .collect(),
                ))
            } else {
                Err(D::Error::custom("Given parameter is too long"))
            }
        } else if let Some(vec) = value.as_array() {
            if vec.len() <= MAX_VECS {
                Ok(Self(
                    sanitize(vec.iter().filter_map(|s| s.as_str().map(String::from)))
                        .into_iter()
                        .map(SeriesName::from)
                        .collect(),
                ))
            } else {
                Err(D::Error::custom("Given parameter is too long"))
            }
        } else {
            Err(D::Error::custom("Bad ids format"))
        }
    }
}

impl fmt::Display for SeriesList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self
            .0
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{s}")
    }
}

fn sanitize(dirty: impl Iterator<Item = String>) -> Vec<String> {
    let mut clean = Vec::new();
    dirty.for_each(|s| {
        let mut current = String::new();
        for c in s.to_lowercase().chars() {
            match c {
                ' ' | ',' | '+' if !current.is_empty() => {
                    clean.push(mem::take(&mut current));
                }
                '-' => current.push('_'),
                c if c.is_alphanumeric() || c == '_' => current.push(c),
                _ => {}
            }
        }
        if !current.is_empty() {
            clean.push(current);
        }
    });
    clean
}
