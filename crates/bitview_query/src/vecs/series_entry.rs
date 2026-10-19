use bitview_primitives::Index;
use vecdb::AnyExportableVec;

/// A queryable vector and its index.
#[derive(Clone, Copy)]
pub struct SeriesEntry<'a> {
    pub(crate) vec: &'a dyn AnyExportableVec,
    pub(crate) index: Index,
    /// Whether existing values may change, preventing an immutable cache prefix.
    pub(crate) is_mutable: bool,
}

pub enum SeriesEntryLookup<'a> {
    Found(SeriesEntry<'a>),
    Unsupported(Vec<Index>),
    Missing,
}
