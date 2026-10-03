use std::{borrow::Cow, fmt};

use bitview_primitives::Date;
use brk_types::Timestamp;
use jiff::Timestamp as JiffTimestamp;
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer, de::Error};

/// A range boundary: integer index, date, or timestamp.
#[derive(Debug, Clone, Copy)]
pub enum RangeIndex {
    Int(i64),
    Date(Date),
    Timestamp(Timestamp),
}

impl JsonSchema for RangeIndex {
    fn schema_name() -> Cow<'static, str> {
        "RangeIndex".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
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
            return Err(Error::custom("empty range index"));
        }
        if let Ok(i) = s.parse::<i64>() {
            return Ok(Self::Int(i));
        }
        if let Ok(date) = s.parse::<Date>() {
            return Ok(Self::Date(date));
        }
        if let Ok(ts) = s.parse::<JiffTimestamp>() {
            let secs = ts.as_second();
            if secs < 0 || secs > u32::MAX as i64 {
                return Err(Error::custom(format!("timestamp out of range: {s}")));
            }
            return Ok(Self::Timestamp(Timestamp::new(secs as u32)));
        }
        Err(Error::custom(format!(
            "expected integer, YYYY-MM-DD, or ISO 8601 timestamp: {s}"
        )))
    }
}
