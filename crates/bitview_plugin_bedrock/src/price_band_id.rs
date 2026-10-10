use super::{LEVEL_COUNT, LevelId, PriceBands};

const PERCENTILE_COUNT: usize = 5;
const PRICE_BAND_COUNT: usize = PERCENTILE_COUNT + LEVEL_COUNT;
const PRICE_BAND_IDS: [PriceBandId; PRICE_BAND_COUNT] = [
    PriceBandId::FloorPct95,
    PriceBandId::FloorPct98,
    PriceBandId::FloorPct99,
    PriceBandId::FloorPct99_5,
    PriceBandId::FloorPct99_9,
    PriceBandId::LevelPct10,
    PriceBandId::LevelPct20,
    PriceBandId::LevelPct30,
    PriceBandId::LevelPct40,
    PriceBandId::LevelMedian,
    PriceBandId::LevelPct60,
    PriceBandId::LevelPct70,
    PriceBandId::LevelPct80,
    PriceBandId::LevelPct90,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriceBandId {
    FloorPct95,
    FloorPct98,
    FloorPct99,
    FloorPct99_5,
    FloorPct99_9,
    LevelPct10,
    LevelPct20,
    LevelPct30,
    LevelPct40,
    LevelMedian,
    LevelPct60,
    LevelPct70,
    LevelPct80,
    LevelPct90,
}

impl PriceBandId {
    pub const ALL: &'static [Self] = &PRICE_BAND_IDS;

    pub const fn suffix(self) -> &'static str {
        match self {
            Self::FloorPct95 => "floor_pct95",
            Self::FloorPct98 => "floor_pct98",
            Self::FloorPct99 => "floor_pct99",
            Self::FloorPct99_5 => "floor_pct99_5",
            Self::FloorPct99_9 => "floor_pct99_9",
            Self::LevelPct10 => "cost_basis_above_floor_pct10",
            Self::LevelPct20 => "cost_basis_above_floor_pct20",
            Self::LevelPct30 => "cost_basis_above_floor_pct30",
            Self::LevelPct40 => "cost_basis_above_floor_pct40",
            Self::LevelMedian => "cost_basis_above_floor_median",
            Self::LevelPct60 => "cost_basis_above_floor_pct60",
            Self::LevelPct70 => "cost_basis_above_floor_pct70",
            Self::LevelPct80 => "cost_basis_above_floor_pct80",
            Self::LevelPct90 => "cost_basis_above_floor_pct90",
        }
    }

    pub fn select<T>(self, values: &PriceBands<T>) -> &T {
        match self {
            Self::FloorPct95 => &values.floor.pct95,
            Self::FloorPct98 => &values.floor.pct98,
            Self::FloorPct99 => &values.floor.pct99,
            Self::FloorPct99_5 => &values.floor.pct99_5,
            Self::FloorPct99_9 => &values.floor.pct99_9,
            Self::LevelPct10 => &values.cost_basis_above_floor.pct10,
            Self::LevelPct20 => &values.cost_basis_above_floor.pct20,
            Self::LevelPct30 => &values.cost_basis_above_floor.pct30,
            Self::LevelPct40 => &values.cost_basis_above_floor.pct40,
            Self::LevelMedian => &values.cost_basis_above_floor.median,
            Self::LevelPct60 => &values.cost_basis_above_floor.pct60,
            Self::LevelPct70 => &values.cost_basis_above_floor.pct70,
            Self::LevelPct80 => &values.cost_basis_above_floor.pct80,
            Self::LevelPct90 => &values.cost_basis_above_floor.pct90,
        }
    }

    pub fn select_mut<T>(self, values: &mut PriceBands<T>) -> &mut T {
        match self {
            Self::FloorPct95 => &mut values.floor.pct95,
            Self::FloorPct98 => &mut values.floor.pct98,
            Self::FloorPct99 => &mut values.floor.pct99,
            Self::FloorPct99_5 => &mut values.floor.pct99_5,
            Self::FloorPct99_9 => &mut values.floor.pct99_9,
            Self::LevelPct10 => &mut values.cost_basis_above_floor.pct10,
            Self::LevelPct20 => &mut values.cost_basis_above_floor.pct20,
            Self::LevelPct30 => &mut values.cost_basis_above_floor.pct30,
            Self::LevelPct40 => &mut values.cost_basis_above_floor.pct40,
            Self::LevelMedian => &mut values.cost_basis_above_floor.median,
            Self::LevelPct60 => &mut values.cost_basis_above_floor.pct60,
            Self::LevelPct70 => &mut values.cost_basis_above_floor.pct70,
            Self::LevelPct80 => &mut values.cost_basis_above_floor.pct80,
            Self::LevelPct90 => &mut values.cost_basis_above_floor.pct90,
        }
    }
}

impl From<LevelId> for PriceBandId {
    fn from(value: LevelId) -> Self {
        match value {
            LevelId::Pct10 => Self::LevelPct10,
            LevelId::Pct20 => Self::LevelPct20,
            LevelId::Pct30 => Self::LevelPct30,
            LevelId::Pct40 => Self::LevelPct40,
            LevelId::Median => Self::LevelMedian,
            LevelId::Pct60 => Self::LevelPct60,
            LevelId::Pct70 => Self::LevelPct70,
            LevelId::Pct80 => Self::LevelPct80,
            LevelId::Pct90 => Self::LevelPct90,
        }
    }
}
