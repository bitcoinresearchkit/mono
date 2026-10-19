use super::{LevelId, LossPercentileId};

/// One of a mode's price bands: a floor per loss percentile, or a level of the cost basis above
/// the 95th-percentile floor.
#[derive(Clone, Copy)]
pub enum PriceBandId {
    Floor(LossPercentileId),
    AboveFloor(LevelId),
}

impl PriceBandId {
    pub fn suffix(self) -> String {
        match self {
            Self::Floor(id) => format!("floor_{}", id.name()),
            Self::AboveFloor(id) => format!("cost_basis_above_floor_{}", id.name()),
        }
    }
}
