use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A JSON Schema type: what a series' values are on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum JsonType {
    Null,
    Boolean,
    Integer,
    Number,
    String,
    Array,
    Object,
}

impl JsonType {
    const ALL: [Self; 7] = [
        Self::Null,
        Self::Boolean,
        Self::Integer,
        Self::Number,
        Self::String,
        Self::Array,
        Self::Object,
    ];

    /// The schema `type` keyword's value.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::Number => "number",
            Self::String => "string",
            Self::Array => "array",
            Self::Object => "object",
        }
    }

    /// The type a schema's `type` keyword names, if it is one.
    pub fn parse(keyword: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == keyword)
    }
}

impl fmt::Display for JsonType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
