use std::fmt;

use schemars::{JsonSchema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer};

use crate::{Date, Timestamp};

/// A range boundary: integer index, date, or timestamp.
#[derive(Debug, Clone, Copy)]
pub enum RangeIndex {
    Int(i64),
    Date(Date),
    Timestamp(Timestamp),
}

impl JsonSchema for RangeIndex {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "RangeIndex".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "description": "A positional index, YYYY-MM-DD date, or ISO 8601 timestamp.",
            "anyOf": [
                {
                    "type": "integer",
                    "format": "int64",
                    "examples": [0, -366]
                },
                {
                    "type": "string",
                    "format": "date",
                    "pattern": "^\\d{4}-\\d{2}-\\d{2}$",
                    "examples": ["2025-08-15"]
                },
                {
                    "type": "string",
                    "format": "date-time",
                    "examples": ["2025-08-15T00:00:00Z"]
                }
            ]
        })
    }
}

impl From<i64> for RangeIndex {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

impl From<Date> for RangeIndex {
    fn from(d: Date) -> Self {
        Self::Date(d)
    }
}

impl From<Timestamp> for RangeIndex {
    fn from(t: Timestamp) -> Self {
        Self::Timestamp(t)
    }
}

impl fmt::Display for RangeIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(i) => write!(f, "{i}"),
            Self::Date(d) => write!(f, "{d}"),
            Self::Timestamp(t) => write!(f, "{t}"),
        }
    }
}

impl<'de> Deserialize<'de> for RangeIndex {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let s = s.trim().trim_matches('"');
        if s.is_empty() {
            return Err(serde::de::Error::custom("empty range index"));
        }
        if let Ok(i) = s.parse::<i64>() {
            return Ok(Self::Int(i));
        }
        if let Some(date) = parse_date(s) {
            return Ok(Self::Date(date));
        }
        if let Ok(ts) = s.parse::<jiff::Timestamp>() {
            let secs = ts.as_second();
            if secs < 0 || secs > u32::MAX as i64 {
                return Err(serde::de::Error::custom(format!(
                    "timestamp out of range: {s}"
                )));
            }
            return Ok(Self::Timestamp(Timestamp::new(secs as u32)));
        }
        Err(serde::de::Error::custom(format!(
            "expected integer, YYYY-MM-DD, or ISO 8601 timestamp: {s}"
        )))
    }
}

fn parse_date(s: &str) -> Option<Date> {
    if s.len() != 10 {
        return None;
    }
    let b = s.as_bytes();
    if b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let year = s[0..4].parse().ok()?;
    let month = s[5..7].parse().ok()?;
    let day = s[8..10].parse().ok()?;
    Some(Date::new(year, month, day))
}

#[cfg(test)]
mod tests {
    use super::RangeIndex;

    #[test]
    fn schema_matches_accepted_wire_forms() {
        let schema = serde_json::to_value(schemars::schema_for!(RangeIndex))
            .expect("RangeIndex schema should serialize");
        let variants = schema["anyOf"]
            .as_array()
            .expect("RangeIndex schema should contain variants");

        assert_eq!(variants[0]["type"], "integer");
        assert_eq!(variants[1]["type"], "string");
        assert_eq!(variants[1]["format"], "date");
        assert_eq!(variants[2]["type"], "string");
        assert_eq!(variants[2]["format"], "date-time");
    }
}
