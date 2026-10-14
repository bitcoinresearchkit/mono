use super::{LevelId, LossPercentileId, PriceBands};

/// One of a mode's price bands: a floor per loss percentile, or a level of the cost basis above
/// the 95th-percentile floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceBandId {
    Floor(LossPercentileId),
    AboveFloor(LevelId),
}

impl PriceBandId {
    pub fn suffix(self) -> String {
        match self {
            Self::Floor(id) => format!("floor_{}", id.suffix()),
            Self::AboveFloor(id) => format!("cost_basis_above_floor_{}", id.suffix()),
        }
    }

    pub fn select<T>(self, values: &PriceBands<T>) -> &T {
        match self {
            Self::Floor(id) => id.select(&values.floor),
            Self::AboveFloor(id) => id.select(&values.cost_basis_above_floor),
        }
    }
}
