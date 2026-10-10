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
    /// Creation-price percentiles of the mode-weighted supply at or above the
    /// mode's 95th-percentile floor. Unavailable when that floor or the
    /// conditional supply subset is unavailable.
    pub cost_basis_above_floor: Levels<T>,
}

impl_named_row_formattable!(PriceBands {
    floor,
    cost_basis_above_floor
});

impl<T> PriceBands<T> {
    pub fn try_from_fn<E>(mut create: impl FnMut(PriceBandId) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            floor: Percentiles {
                pct95: create(PriceBandId::FloorPct95)?,
                pct98: create(PriceBandId::FloorPct98)?,
                pct99: create(PriceBandId::FloorPct99)?,
                pct99_5: create(PriceBandId::FloorPct99_5)?,
                pct99_9: create(PriceBandId::FloorPct99_9)?,
            },
            cost_basis_above_floor: Levels::try_from_fn(|id| create(id.into()))?,
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.floor
            .iter_mut()
            .chain(self.cost_basis_above_floor.iter_mut())
    }
    pub fn from_fn(mut create: impl FnMut(PriceBandId) -> T) -> Self {
        Self {
            floor: Percentiles {
                pct95: create(PriceBandId::FloorPct95),
                pct98: create(PriceBandId::FloorPct98),
                pct99: create(PriceBandId::FloorPct99),
                pct99_5: create(PriceBandId::FloorPct99_5),
                pct99_9: create(PriceBandId::FloorPct99_9),
            },
            cost_basis_above_floor: Levels::from_fn(|id: LevelId| create(id.into())),
        }
    }
}
