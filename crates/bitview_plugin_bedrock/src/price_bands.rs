use bitview_traversable::Traversable;

use super::{Levels, Percentiles, PriceBandId};

#[derive(Clone, Traversable)]
pub struct PriceBands<T> {
    /// Lowest creation price at which the share of mode-weighted supply with a
    /// higher creation price is no greater than the historical loss-share
    /// threshold identified below, creation prices rounded to four significant
    /// digits of cents (exact under $100, then steps of at most 0.1%). If spot equaled this floor, that higher-cost
    /// supply would be in loss. Unavailable when no mode-weighted supply has a
    /// positive creation price, or without a historical threshold.
    pub floor: Percentiles<T>,
    /// Creation-price percentiles of the mode-weighted supply at or above the
    /// mode's 95th-percentile floor, on the same rounded creation prices. Unavailable when that floor or the
    /// conditional supply subset is unavailable.
    pub cost_basis_above_p95_floor: Levels<T>,
}

impl<T> PriceBands<T> {
    pub fn try_from_fn<E>(mut create: impl FnMut(PriceBandId) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            floor: Percentiles::try_from_fn(|id| create(PriceBandId::Floor(id)))?,
            cost_basis_above_p95_floor: Levels::try_from_fn(|id| {
                create(PriceBandId::AboveFloor(id))
            })?,
        })
    }

    pub fn from_fn(mut create: impl FnMut(PriceBandId) -> T) -> Self {
        Self {
            floor: Percentiles::from_fn(|id| create(PriceBandId::Floor(id))),
            cost_basis_above_p95_floor: Levels::from_fn(|id| create(PriceBandId::AboveFloor(id))),
        }
    }

    pub fn select(&self, id: PriceBandId) -> &T {
        match id {
            PriceBandId::Floor(id) => self.floor.select(id),
            PriceBandId::AboveFloor(id) => self.cost_basis_above_p95_floor.select(id),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.floor
            .iter()
            .chain(self.cost_basis_above_p95_floor.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.floor
            .iter_mut()
            .chain(self.cost_basis_above_p95_floor.iter_mut())
    }
}
