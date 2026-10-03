use bitview_plugin::{Plugin, PublicationReadGuard};
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
    _guard: SeriesGuard,
}

enum SeriesGuard {
    Publication { _guard: PublicationReadGuard },
    Prefix { _guard: SafeLengths },
}

impl SeriesRead {
    pub(super) fn new(query: &Query, entries: Vec<SeriesEntry<'static>>) -> Result<Self> {
        // The indexer's exported facts only append above the published prefix;
        // rollback is excluded by SafeLengths. Plugin vectors can recompute
        // existing rows, independently of their HTTP cache mutability flag.
        let indexer_only = entries
            .iter()
            .all(|entry| entry.plugin().id() == query.indexer().id());
        let (guard, safe, bounds) = if indexer_only {
            let pin = query.pin_safe_lengths()?;
            let safe = pin.lengths();
            (
                SeriesGuard::Prefix { _guard: pin },
                safe,
                Query::index_read_bounds(safe),
            )
        } else {
            let guard = query.read_publication()?;
            let safe = query.safe_lengths();
            (
                SeriesGuard::Publication { _guard: guard },
                safe,
                query.read_bounds(safe),
            )
        };
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
            _guard: guard,
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
