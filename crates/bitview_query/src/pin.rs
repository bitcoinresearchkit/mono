use std::ops::Deref;
#[cfg(feature = "series")]
use std::sync::Arc;

use bitview_plugin_indexer::SafeLengths;
#[cfg(feature = "series")]
use bitview_primitives::{DifficultyEpoch, HalvingEpoch, Index, Lengths};
#[cfg(feature = "full-api")]
use brk_types::Height;
#[cfg(feature = "series")]
use vecdb::{ReadBounds, ReadScope, ReadableVec};

use crate::{Error, Query, Result};

#[cfg(feature = "series")]
pub(crate) struct PublishedBounds {
    pub(crate) revision: u64,
    pub(crate) bounds: ReadBounds,
}

/// A pin that bounds this thread's reads, lazy calendar and count views included, by its
/// publication until it drops, so no read under it can see past it. Not `Send`: a value carried
/// to another thread keeps [`ReadPin::into_inner`]'s plain pin.
pub(crate) struct ReadPin {
    pin: SafeLengths,
    #[cfg(feature = "series")]
    bounds: ReadBounds,
    #[cfg(feature = "series")]
    _scope: ReadScope,
}

impl ReadPin {
    #[cfg(feature = "series")]
    pub(crate) fn bounds(&self) -> &ReadBounds {
        &self.bounds
    }

    /// The pin alone, without bounding this thread's reads.
    #[cfg(feature = "series")]
    pub(crate) fn into_inner(self) -> SafeLengths {
        self.pin
    }
}

impl Deref for ReadPin {
    type Target = SafeLengths;

    fn deref(&self) -> &SafeLengths {
        &self.pin
    }
}

/// `StateUpdating` for a read targeting `index` at or past `bound` (its kind's exclusive bound in
/// `pin`) while the pin is lowered: a rollback or rebuild is rewriting the canonical rows there,
/// which are hidden, not missing. Otherwise `Ok`, and the caller treats it like any row past the
/// tip (missing, or clamped); reads wholly inside the retained prefix proceed either way.
#[cfg(feature = "series")]
pub(crate) fn check_bound<T: PartialOrd>(pin: &SafeLengths, index: T, bound: T) -> Result<()> {
    if index >= bound && pin.is_lowered() {
        return Err(Error::updating());
    }
    Ok(())
}

impl Query {
    /// Pins the published state for one read: every plugin's rows below its lengths stay put
    /// (only a rollback waits for the pin; pinning never waits), and this thread's reads stay
    /// below them while the pin lives. Fails past the deadline, and while a rebuild lowered the
    /// publication to genesis: the chain is updating, not empty.
    pub(crate) fn pin_safe_lengths(&self) -> Result<ReadPin> {
        self.check_deadline()?;
        let pin = self.indexer().pin();
        if pin.is_lowered() && pin.lengths().height.is_zero() {
            return Err(Error::updating());
        }
        Ok(self.read_pin(pin))
    }

    /// Bounds this thread's reads by `pin` while the returned pin lives: how a value carried from
    /// an earlier read resumes reading under its pin.
    pub(crate) fn read_pin(&self, pin: SafeLengths) -> ReadPin {
        #[cfg(feature = "series")]
        let bounds = self.publication_bounds(&pin);
        ReadPin {
            #[cfg(feature = "series")]
            _scope: bounds.enter(),
            #[cfg(feature = "series")]
            bounds,
            pin,
        }
    }

    /// The read bounds of `pin`'s publication: cached for the newest one, built otherwise.
    #[cfg(feature = "series")]
    fn publication_bounds(&self, pin: &SafeLengths) -> ReadBounds {
        let cached = self.0.bounds.load();
        if let Some(cached) = cached.as_deref()
            && cached.revision == pin.revision()
        {
            return cached.bounds.clone();
        }
        drop(cached);
        let bounds = self.read_bounds(pin.lengths());
        let published = Arc::new(PublishedBounds {
            revision: pin.revision(),
            bounds: bounds.clone(),
        });
        // An older pin never replaces a newer publication's bounds.
        self.0.bounds.rcu(|cached| match cached {
            Some(cached) if cached.revision >= published.revision => Some(Arc::clone(cached)),
            _ => Some(Arc::clone(&published)),
        });
        bounds
    }

    /// The published tip height a pin reads at; `StateUpdating` while nothing is published.
    #[cfg(feature = "full-api")]
    pub(crate) fn tip_height(pin: &SafeLengths) -> Result<Height> {
        pin.lengths().last_height().ok_or_else(|| Error::updating())
    }

    #[cfg(feature = "series")]
    fn index_read_bounds(safe: Lengths) -> ReadBounds {
        let mut bounds = ReadBounds::new();
        for index in Index::all() {
            if let Some(len) = safe.get(index) {
                bounds.set(index.name(), len);
            }
        }

        let tip = safe.last_height();
        bounds.set(
            Index::DifficultyEpoch.name(),
            tip.map(|height| usize::from(DifficultyEpoch::from(height)) + 1)
                .unwrap_or(0),
        );
        bounds.set(
            Index::HalvingEpoch.name(),
            tip.map(|height| usize::from(HalvingEpoch::from(height)) + 1)
                .unwrap_or(0),
        );

        bounds
    }

    #[cfg(feature = "series")]
    fn read_bounds(&self, safe: Lengths) -> ReadBounds {
        let mut bounds = Self::index_read_bounds(safe);
        let timestamp = safe.last_height().and_then(|height| {
            self.plugins()
                .mappings
                .timestamp
                .monotonic
                .collect_one(height)
        });
        for index in Index::all().into_iter().filter(Index::is_date_based) {
            let len = timestamp
                .and_then(|timestamp| index.timestamp_to_index(timestamp))
                .map(|last| last + 1)
                .unwrap_or(0);
            bounds.set(index.name(), len);
        }

        bounds
    }
}
