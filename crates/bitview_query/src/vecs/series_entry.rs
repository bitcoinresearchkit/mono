use bitview_primitives::Index;
use vecdb::AnyExportableVec;

/// A queryable vector and its index.
#[derive(Clone, Copy)]
pub struct SeriesEntry<'a> {
    vec: &'a dyn AnyExportableVec,
    index: Index,
    is_mutable: bool,
}

pub enum SeriesEntryLookup<'a> {
    Found(SeriesEntry<'a>),
    Unsupported(Vec<Index>),
    Missing,
}

impl<'a> SeriesEntry<'a> {
    pub(crate) fn new(index: Index, vec: &'a dyn AnyExportableVec, is_mutable: bool) -> Self {
        Self {
            vec,
            index,
            is_mutable,
        }
    }

    pub(crate) fn index(self) -> Index {
        self.index
    }

    pub(crate) fn vec(self) -> &'a dyn AnyExportableVec {
        self.vec
    }

    /// Whether existing values may change, preventing an immutable cache prefix.
    pub(crate) fn is_mutable(self) -> bool {
        self.is_mutable
    }
}
