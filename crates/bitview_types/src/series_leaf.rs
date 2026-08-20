use std::collections::BTreeSet;

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
}

impl SeriesLeaf {
    pub fn new(name: String, kind: String, indexes: BTreeSet<Index>) -> Self {
        Self {
            name,
            kind,
            indexes,
        }
    }

    /// Merge another leaf's indexes into this one (union).
    pub fn merge_indexes(&mut self, other: &Self) {
        self.indexes.extend(other.indexes.iter().copied());
    }
}
