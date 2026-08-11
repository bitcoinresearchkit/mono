use std::{fmt, str};

use brk_traversable::Traversable;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::Formattable;

use super::{LevelId, Levels, Percentiles, PriceBandId};

#[derive(Debug, Clone, Copy, PartialEq, Traversable, Serialize, JsonSchema)]
pub struct PriceBands<T> {
    pub floor: Percentiles<T>,
    pub level: Levels<T>,
}

impl_named_row_formattable!(PriceBands { floor, level });

impl<T> PriceBands<T> {
    pub(super) fn from_fn(mut create: impl FnMut(PriceBandId) -> T) -> Self {
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

    pub(super) fn map<U>(self, mut map: impl FnMut(T) -> U) -> PriceBands<U> {
        PriceBands {
            floor: self.floor.map(&mut map),
            level: self.level.map(map),
        }
    }
}
