use bitview_collections::Windows;
use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::ReadableCloneableVec;

use crate::{IndexSources, LazyRollingAvgsFromHeight, LazyRollingSumsFromHeight};

/// Sums and averages projected from the same cumulative source and windows.
#[derive(Clone, Traversable)]
pub struct RollingTotals<T: NumericValue + JsonSchema + Quantity> {
    pub sum: LazyRollingSumsFromHeight<T>,
    average: LazyRollingAvgsFromHeight<T>,
}

impl<T: NumericValue + JsonSchema + Quantity> RollingTotals<T> {
    pub(crate) fn new(
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, T>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        Self {
            sum: LazyRollingSumsFromHeight::new(
                &format!("{name}_sum"),
                version,
                source,
                window_starts,
                indexes,
            ),
            average: LazyRollingAvgsFromHeight::new(
                &format!("{name}_average"),
                version,
                source,
                window_starts,
                indexes,
            ),
        }
    }
}
