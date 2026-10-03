use bitview_collections::Windows;
use bitview_primitives::{PartsPerMillion32, StoredU64};
use bitview_transforms::RatioU64;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::{LazyVec, ReadableCloneableVec};

use crate::{IndexSources, LazyPerBlockCumulativeRolling, LazyPercentCumulativeRolling};

/// Total-count views used by their breakdowns, without a separate retained cache.
#[derive(Clone, Traversable)]
pub struct CountTotal {
    all: LazyPerBlockCumulativeRolling<StoredU64>,
}

impl CountTotal {
    /// Derive views from the owner's total.
    pub fn from_source(
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, StoredU64>,
        indexes: &IndexSources,
        windows: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Self {
        Self::from_transformed_source(name, version, source, |_, value| value, indexes, windows)
    }

    /// Derive views from an adjusted total.
    /// The caller selects the adjustment (for example, excluding coinbase transactions).
    pub fn from_transformed_source(
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, StoredU64>,
        transform: fn(Height, StoredU64) -> StoredU64,
        indexes: &IndexSources,
        windows: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Self {
        let source = LazyVec::init(
            &format!("{name}_cumulative_source"),
            version,
            source.read_only_boxed_clone(),
            transform,
        );
        Self {
            all: LazyPerBlockCumulativeRolling::from_cumulative_source(
                name, version, &source, windows, indexes,
            ),
        }
    }

    pub fn lazy_share(
        &self,
        name: &str,
        version: Version,
        numerator: &impl ReadableCloneableVec<Height, StoredU64>,
        windows: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> LazyPercentCumulativeRolling<PartsPerMillion32> {
        LazyPercentCumulativeRolling::from_cumulative_ratio::<
            StoredU64,
            StoredU64,
            RatioU64<PartsPerMillion32>,
        >(
            name,
            version,
            numerator,
            &self.all.cumulative.height,
            windows,
            indexes,
        )
    }
}
