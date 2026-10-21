use std::{collections::BTreeSet, sync::Arc};

use bitview_primitives::{Index, SeriesName};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct SeriesLeaf {
    #[schemars(with = "SeriesName")]
    pub name: String,
    /// The value type, by its schema's name (e.g., "Sats", "Ratio").
    pub kind: String,
    pub indexes: BTreeSet<Index>,
    /// Indexes whose values can be null: a missing value (e.g. a period without blocks) or an
    /// undefined one (e.g. NaN).
    pub nullable: BTreeSet<Index>,
    /// Human-readable metric definition, when documented.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Arc<str>>,
}

impl SeriesLeaf {
    pub fn new(name: String, kind: String, indexes: BTreeSet<Index>) -> Self {
        Self {
            name,
            kind,
            indexes,
            nullable: BTreeSet::new(),
            description: None,
        }
    }

    /// Two occurrences of a series agree unless both are described, differently.
    pub(crate) fn descriptions_agree(a: Option<&Arc<str>>, b: Option<&Arc<str>>) -> bool {
        a.zip(b).is_none_or(|(a, b)| a == b)
    }

    /// Merge compatible metadata for another occurrence of the same series.
    pub(crate) fn merge(&mut self, other: &Self) -> Option<()> {
        if !Self::descriptions_agree(self.description.as_ref(), other.description.as_ref()) {
            return None;
        }
        if self.description.is_none() {
            self.description.clone_from(&other.description);
        }
        self.indexes.extend(other.indexes.iter().copied());
        self.nullable.extend(other.nullable.iter().copied());
        Some(())
    }
}
