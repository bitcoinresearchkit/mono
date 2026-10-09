use std::{fmt, ops::Deref, path::Path};

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, de::Error};

/// URPD cohort identifier. Use `GET /api/urpd` to list available cohorts.
///
/// Names are non-empty ASCII `[a-z0-9_]+`. Availability is determined by
/// supported age filters and the published UTXO set.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, JsonSchema)]
#[schemars(extend("pattern" = "^[a-z0-9_]+$"))]
pub struct Cohort(String);

impl Cohort {
    /// Returns `Some(Cohort)` iff `s` is non-empty ASCII `[a-z0-9_]+`.
    pub fn new(s: impl Into<String>) -> Option<Self> {
        let s = s.into();
        if s.is_empty()
            || !s
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return None;
        }
        Some(Self(s))
    }
}

impl fmt::Display for Cohort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Deref for Cohort {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Cohort {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<Path> for Cohort {
    fn as_ref(&self) -> &Path {
        Path::new(&self.0)
    }
}

impl<'de> Deserialize<'de> for Cohort {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Self::new(s).ok_or_else(|| Error::custom("invalid cohort: expected non-empty [a-z0-9_]+"))
    }
}
