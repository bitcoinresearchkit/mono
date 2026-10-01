use std::{
    collections::BTreeSet,
    hash::{Hash, Hasher},
};

use brk_types::Index;

/// A field in a structural pattern.
#[derive(Debug, Clone, PartialOrd, Ord)]
pub struct PatternField {
    /// Field name
    pub(crate) name: String,
    /// Rust type for leaves or pattern name for branches
    pub(crate) rust_type: String,
    /// JSON type from schema
    pub(crate) json_type: String,
    /// For leaves: the set of supported indexes. Empty for branches.
    pub(crate) indexes: BTreeSet<Index>,
    /// For branches referencing generic patterns: the concrete type parameter
    pub(crate) type_param: Option<String>,
}

impl PatternField {
    /// Returns true if this is a leaf field (has indexes).
    pub(crate) fn is_leaf(&self) -> bool {
        !self.indexes.is_empty()
    }

    /// Returns true if this is a branch field (no indexes).
    pub(crate) fn is_branch(&self) -> bool {
        self.indexes.is_empty()
    }
}

impl Hash for PatternField {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.rust_type.hash(state);
        self.json_type.hash(state);
        self.indexes.hash(state);
    }
}

impl PartialEq for PatternField {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.rust_type == other.rust_type
            && self.json_type == other.json_type
            && self.indexes == other.indexes
    }
}

impl Eq for PatternField {}
