use bitview_collections::Windows;
use bitview_primitives::{Count, PartsPerMillion32};
use bitview_transforms::Quotient;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::{LazyVec, ReadableCloneableVec};

use crate::{IndexSources, LazyFixedRatioCumulativeRolling, LazyPerBlockCumulativeRolling};

/// Total-count views used by their breakdowns, without a separate retained cache.
#[derive(Clone, Traversable)]
pub struct CountTotal {
    all: LazyPerBlockCumulativeRolling<Count>,
}

impl CountTotal {
    /// Derive views from the owner's total.
    pub fn from_source(
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, Count>,
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
        source: &impl ReadableCloneableVec<Height, Count>,
        transform: fn(Height, Count) -> Count,
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
        numerator: &impl ReadableCloneableVec<Height, Count>,
        windows: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> LazyFixedRatioCumulativeRolling<PartsPerMillion32> {
        LazyFixedRatioCumulativeRolling::from_cumulative_ratio::<
            Count,
            Count,
            Quotient<PartsPerMillion32>,
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
