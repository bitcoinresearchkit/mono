use std::{collections::BTreeSet, sync::Arc};

use bitview_primitives::Index;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Leaf node containing series metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct SeriesLeaf {
    /// The series name/identifier.
    pub name: String,
    /// The Rust type (e.g., "Sats", "Ratio").
    pub kind: String,
    /// Available indexes for this series.
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

    /// Merge compatible metadata for another occurrence of the same series.
    pub(crate) fn merge(&mut self, other: &Self) -> Option<()> {
        match (&self.description, &other.description) {
            (Some(current), Some(incoming)) if current != incoming => return None,
            (None, Some(description)) => self.description = Some(description.clone()),
            _ => {}
        }
        self.indexes.extend(other.indexes.iter().copied());
        self.nullable.extend(other.nullable.iter().copied());
        Some(())
    }
}
