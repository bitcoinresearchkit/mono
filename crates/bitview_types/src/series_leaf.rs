use std::{borrow::Cow, collections::BTreeSet};

use brk_types::Index;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Leaf node containing series metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct SeriesLeaf {
    /// The series name/identifier.
    pub name: String,
    /// The Rust type (e.g., "Sats", "StoredF64").
    pub kind: String,
    /// Available indexes for this series.
    pub indexes: BTreeSet<Index>,
    /// Human-readable metric definition, when documented.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Cow<'static, str>>,
}

impl SeriesLeaf {
    pub fn new(name: String, kind: String, indexes: BTreeSet<Index>) -> Self {
        Self {
            name,
            kind,
            indexes,
            description: None,
        }
    }

    /// Merge compatible metadata for another occurrence of the same series.
    pub fn merge(&mut self, other: &Self) -> Option<()> {
        match (&self.description, &other.description) {
            (Some(current), Some(incoming)) if current != incoming => return None,
            (None, Some(description)) => self.description = Some(description.clone()),
            _ => {}
        }
        self.indexes.extend(other.indexes.iter().copied());
        Some(())
    }
}
