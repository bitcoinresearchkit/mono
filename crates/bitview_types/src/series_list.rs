use std::{borrow::Cow, fmt, mem};

use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, de::Error as _};
use serde_json::Value;

use super::SeriesName;

/// Comma-separated list of series names
///
/// Deserialization permits at most 32 normalized names and 2,048 decoded input
/// string bytes. For arrays, the byte budget is shared by their string values.
#[derive(Clone, Debug, Deref, JsonSchema)]
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

        match value {
            Value::String(text) if text.len() <= MAX_STRING_SIZE => sanitize(text.split(','))
                .map(Self)
                .map_err(D::Error::custom),
            Value::Array(values)
                if values.len() <= MAX_VECS
                    && values
                        .iter()
                        .filter_map(Value::as_str)
                        .try_fold(MAX_STRING_SIZE, |remaining, text| {
                            remaining.checked_sub(text.len())
                        })
                        .is_some() =>
            {
                sanitize(values.into_iter().filter_map(|value| match value {
                    Value::String(text) => Some(text),
                    _ => None,
                }))
                .map(Self)
                .map_err(D::Error::custom)
            }
            Value::String(_) | Value::Array(_) => {
                Err(D::Error::custom("Given parameter is too long"))
            }
            _ => Err(D::Error::custom("Bad ids format")),
        }
    }
}

impl fmt::Display for SeriesList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (index, name) in self.0.iter().enumerate() {
            if index != 0 {
                f.write_str(",")?;
            }
            fmt::Display::fmt(name, f)?;
        }
        Ok(())
    }
}

fn sanitize<'a, S: Into<Cow<'a, str>>>(
    dirty: impl Iterator<Item = S>,
) -> Result<Vec<SeriesName>, &'static str> {
    let mut clean = Vec::new();
    for s in dirty {
        let s = s.into();
        if s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            if !s.is_empty() {
                if clean.len() == MAX_VECS {
                    return Err("At most 32 series may be requested");
                }
                clean.push(SeriesName::from(s.into_owned()));
            }
            continue;
        }
        let mut current = String::new();
        // A final separator flushes the last name through the same bound check.
        for c in s.to_lowercase().chars().chain([' ']) {
            match c {
                ' ' | ',' | '+' if !current.is_empty() => {
                    if clean.len() == MAX_VECS {
                        return Err("At most 32 series may be requested");
                    }
                    clean.push(SeriesName::from(mem::take(&mut current)));
                }
                '-' => current.push('_'),
                c if c.is_alphanumeric() || c == '_' => current.push(c),
                _ => {}
            }
        }
    }
    Ok(clean)
}
