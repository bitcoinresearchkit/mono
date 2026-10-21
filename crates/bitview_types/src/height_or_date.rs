use std::{borrow::Cow, fmt, str::FromStr};

use bitview_primitives::Date;
use brk_types::Height;
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, Visitor},
};

/// A confirmed point of the chain: a block height (`840000`) or a UTC calendar day (`YYYY-MM-DD`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightOrDate {
    Height(Height),
    Date(Date),
}

impl From<Height> for HeightOrDate {
    fn from(height: Height) -> Self {
        Self::Height(height)
    }
}

impl From<Date> for HeightOrDate {
    fn from(date: Date) -> Self {
        Self::Date(date)
    }
}

impl fmt::Display for HeightOrDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Height(height) => write!(f, "{height}"),
            Self::Date(date) => write!(f, "{date}"),
        }
    }
}

impl FromStr for HeightOrDate {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(date) = s.parse::<Date>() {
            Ok(Self::Date(date))
        } else if let Ok(height) = s.parse::<u32>() {
            Ok(Self::Height(Height::from(height)))
        } else {
            Err(format!(
                "expected a block height or YYYY-MM-DD date, got `{s}`"
            ))
        }
    }
}

impl Serialize for HeightOrDate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Height(height) => height.serialize(serializer),
            Self::Date(date) => date.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for HeightOrDate {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PointVisitor;

        impl Visitor<'_> for PointVisitor {
            type Value = HeightOrDate;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a block height or a YYYY-MM-DD date")
            }

            fn visit_u64<E: Error>(self, height: u64) -> Result<Self::Value, E> {
                u32::try_from(height)
                    .map(|height| HeightOrDate::Height(Height::from(height)))
                    .map_err(|_| E::custom(format!("block height out of range: {height}")))
            }

            fn visit_str<E: Error>(self, text: &str) -> Result<Self::Value, E> {
                text.parse().map_err(E::custom)
            }
        }

        // Path segments arrive as strings, JSON heights as numbers.
        deserializer.deserialize_any(PointVisitor)
    }
}

impl JsonSchema for HeightOrDate {
    fn schema_name() -> Cow<'static, str> {
        "HeightOrDate".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "A confirmed point of the chain: a block height (`840000`) or a UTC calendar day \
                (`YYYY-MM-DD`).",
            "anyOf": [generator.subschema_for::<Height>(), generator.subschema_for::<Date>()],
        })
    }
}
