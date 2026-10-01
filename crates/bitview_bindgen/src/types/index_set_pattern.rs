use std::collections::BTreeSet;

use brk_types::Index;

/// A pattern of indexes that appear together on multiple series.
#[derive(Debug, Clone)]
pub struct IndexSetPattern {
    /// Pattern name (e.g., "DateHeightIndexes")
    pub(crate) name: String,
    /// The set of indexes
    pub(crate) indexes: BTreeSet<Index>,
}
