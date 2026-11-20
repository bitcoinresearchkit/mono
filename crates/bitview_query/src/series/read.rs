use bitview_plugin_indexer::SafeLengths;
use bitview_primitives::BlockHashPrefix;
use vecdb::{AnyExportableVec, BoundedVec, ReadBounds};

use super::SeriesAnchor;
use crate::{Error, Query, Result, SeriesEntry};

/// Selected series and their published read protection.
///
/// Column readers borrow this view, so they cannot outlive its read protection.
/// The underlying unbounded vectors and bounds are deliberately private.
pub struct SeriesRead {
    entries: Vec<SeriesEntry<'static>>,
    bounds: ReadBounds,
    is_mutable: bool,
    pin: SafeLengths,
}

impl SeriesRead {
    pub(super) fn new(query: &Query, entries: Vec<SeriesEntry<'static>>) -> Result<Self> {
        // Every plugin's rows below the published lengths stay put while the pin is held: updates
        // append past them, and only a rollback (which waits for the pin) rewrites them.
        let pin = query.pin_safe_lengths()?;
        let mut bounds = pin.bounds().clone();
        // A bulk selection only covers rows shared by every requested series.
        for entry in &entries {
            if !entry.available {
                bounds.set(entry.index.name(), 0);
            }
        }
        let is_mutable = entries.iter().any(|entry| entry.is_mutable);
        debug_assert!(
            entries.iter().all(|entry| bounds.bind(entry.vec).is_some()),
            "every series index has a published bound"
        );
        Ok(Self {
            entries,
            bounds,
            is_mutable,
            pin: pin.into_inner(),
        })
    }

    pub(super) fn pin(&self) -> &SafeLengths {
        &self.pin
    }

    pub(super) fn is_mutable(&self) -> bool {
        self.is_mutable
    }

    /// Tied to the published tip: changed by every block.
    pub(super) fn tip_anchor(&self) -> SeriesAnchor {
        SeriesAnchor::Tip(BlockHashPrefix::from(&self.pin.tip()))
    }

    pub(super) fn bind<'a>(&'a self, vec: &'a dyn AnyExportableVec) -> Result<BoundedVec<'a>> {
        self.bounds
            .bind(vec)
            .ok_or(Error::Internal("Missing published series bound"))
    }

    pub(crate) fn columns(&self) -> impl ExactSizeIterator<Item = BoundedVec<'_>> + '_ {
        self.entries.iter().map(|entry| {
            self.bounds
                .bind(entry.vec)
                .expect("selected series bounds were checked at construction")
                .with_null_values(entry.null_values)
        })
    }
}
