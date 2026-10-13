use bitview_plugin_indexer::SafeLengths;
use bitview_primitives::Lengths;
use vecdb::{AnyExportableVec, BoundedVec, ReadBounds};

use crate::{Error, Query, Result, vecs::SeriesEntry};

/// Selected series and their published read protection.
///
/// Column readers borrow this view, so they cannot outlive its read protection.
/// The underlying unbounded vectors and bounds are deliberately private.
pub struct SeriesRead {
    vecs: Vec<&'static dyn AnyExportableVec>,
    bounds: ReadBounds,
    safe: Lengths,
    is_mutable: bool,
    _pin: SafeLengths,
}

impl SeriesRead {
    pub(super) fn new(query: &Query, entries: Vec<SeriesEntry<'static>>) -> Result<Self> {
        // Every plugin's rows below the published lengths stay put while the pin is held: updates
        // append past them, and only a rollback (which waits for the pin) rewrites them.
        let pin = query.pin_safe_lengths()?;
        let safe = pin.lengths();
        let bounds = query.read_bounds(safe);
        let is_mutable = entries.iter().any(|entry| entry.is_mutable());
        let vecs = entries
            .into_iter()
            .map(SeriesEntry::vec)
            .collect::<Vec<_>>();
        for vec in &vecs {
            bounds
                .bind(*vec)
                .ok_or(Error::Internal("Missing published series bound"))?;
        }
        Ok(Self {
            vecs,
            bounds,
            safe,
            is_mutable,
            _pin: pin,
        })
    }

    pub(super) fn safe_lengths(&self) -> Lengths {
        self.safe
    }

    pub(super) fn is_mutable(&self) -> bool {
        self.is_mutable
    }

    pub(super) fn bind<'a>(&'a self, vec: &'a dyn AnyExportableVec) -> Result<BoundedVec<'a>> {
        self.bounds
            .bind(vec)
            .ok_or(Error::Internal("Missing published series bound"))
    }

    pub(crate) fn columns(&self) -> impl ExactSizeIterator<Item = BoundedVec<'_>> + '_ {
        self.vecs.iter().map(|vec| {
            self.bounds
                .bind(*vec)
                .expect("selected series bounds were checked at construction")
        })
    }
}
