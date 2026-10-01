use bitview_collections::Windows;
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::ReadableCloneableVec;

use crate::{IndexSources, LazyRollingAvgsAmountFromHeight, LazyRollingSumsAmountFromHeight};

/// Rolling totals of cumulative sats and historical fiat flows.
#[derive(Clone, Traversable)]
pub struct RollingAmountTotals {
    pub sum: LazyRollingSumsAmountFromHeight,
    pub average: LazyRollingAvgsAmountFromHeight,
}

impl RollingAmountTotals {
    pub(crate) fn new(
        name: &str,
        version: Version,
        sats: &impl ReadableCloneableVec<Height, Sats>,
        cents: &impl ReadableCloneableVec<Height, Cents>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        Self {
            sum: LazyRollingSumsAmountFromHeight::new(
                &format!("{name}_sum"),
                version,
                sats,
                cents,
                window_starts,
                indexes,
            ),
            average: LazyRollingAvgsAmountFromHeight::new(
                &format!("{name}_average"),
                version,
                sats,
                cents,
                window_starts,
                indexes,
            ),
        }
    }
}
