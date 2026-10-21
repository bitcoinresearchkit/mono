use std::{borrow::Cow, fmt, str::FromStr};

use bitview_cohort::{AgeAggregateId, AgeRangeId};
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

/// A URPD cohort, by its series' prefix: an age aggregate (`all`, `sth`, `lth`, `age_under_4m`, ...)
/// or an age band (`age_under_1h`, `age_1h_to_1d`, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UrpdCohort {
    Aggregate(AgeAggregateId),
    Range(AgeRangeId),
}

impl UrpdCohort {
    /// Every cohort: the aggregates, then the age bands from the youngest.
    pub fn all() -> impl Iterator<Item = Self> {
        (AgeAggregateId::ALL.iter().copied().map(Self::Aggregate))
            .chain(AgeRangeId::ALL.iter().copied().map(Self::Range))
    }

    /// The cohort's name, its series' prefix (`sth`, `age_1h_to_1d`).
    pub fn name(self) -> String {
        match self {
            Self::Aggregate(id) => id.full_name(),
            Self::Range(id) => id.full_name(),
        }
    }

    /// The age bands the cohort spans.
    pub fn age_range_ids(self) -> &'static [AgeRangeId] {
        match self {
            Self::Aggregate(id) => id.age_range_ids(),
            Self::Range(id) => &AgeRangeId::ALL[id.index()..id.index() + 1],
        }
    }
}

impl fmt::Display for UrpdCohort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

impl FromStr for UrpdCohort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AgeAggregateId::from_full_name(s)
            .map(Self::Aggregate)
            .or_else(|| AgeRangeId::from_full_name(s).map(Self::Range))
            .ok_or_else(|| format!("unknown URPD cohort `{s}` (`GET /api/urpd` lists them)"))
    }
}

impl Serialize for UrpdCohort {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.name())
    }
}

impl<'de> Deserialize<'de> for UrpdCohort {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(Error::custom)
    }
}

impl JsonSchema for UrpdCohort {
    fn schema_name() -> Cow<'static, str> {
        "UrpdCohort".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        let names: Vec<String> = Self::all().map(Self::name).collect();
        json_schema!({
            "description": "A URPD cohort, by its series' prefix: an age aggregate (`all`, `sth`, `lth`, \
                `age_under_4m`, ...) or an age band (`age_under_1h`, `age_1h_to_1d`, ...).",
            "type": "string",
            "enum": names,
        })
    }
}
