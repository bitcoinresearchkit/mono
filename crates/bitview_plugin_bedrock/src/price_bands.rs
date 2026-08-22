use std::{fmt, str};

use bitview_traversable::Traversable;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::Formattable;

use super::{LevelId, Levels, Percentiles, PriceBandId};

#[derive(Debug, Clone, Copy, PartialEq, Traversable, Serialize, JsonSchema)]
pub struct PriceBands<T> {
    /// Lowest creation price at which the share of mode-weighted supply with a
    /// higher creation price is no greater than the historical loss-share
    /// threshold identified below. If spot equaled this floor, that higher-cost
    /// supply would be in loss. Unavailable when the mode has no positive
    /// weighted supply or no historical threshold.
    pub floor: Percentiles<T>,
    /// Additional creation-price level within the mode-weighted supply at or
    /// above the mode's 95th-percentile floor. Unavailable when that floor or
    /// the conditional supply subset is unavailable.
    pub level: Levels<T>,
}

impl_named_row_formattable!(PriceBands { floor, level });

impl<T> PriceBands<T> {
    pub fn from_fn(mut create: impl FnMut(PriceBandId) -> T) -> Self {
        Self {
            floor: Percentiles {
                pct95: create(PriceBandId::FloorPct95),
                pct98: create(PriceBandId::FloorPct98),
                pct99: create(PriceBandId::FloorPct99),
                pct99_5: create(PriceBandId::FloorPct99_5),
                pct99_9: create(PriceBandId::FloorPct99_9),
            },
            level: Levels::from_fn(|id: LevelId| create(id.into())),
        }
    }

    pub fn map<U>(self, mut map: impl FnMut(T) -> U) -> PriceBands<U> {
        PriceBands {
            floor: self.floor.map(&mut map),
            level: self.level.map(map),
        }
    }
}
