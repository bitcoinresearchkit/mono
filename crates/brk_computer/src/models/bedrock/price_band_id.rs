use brk_types::Version;
use vecdb::{ColumnId, VecValue};

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
    PriceBandId::LevelPct50,
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
    LevelPct50,
    LevelPct60,
    LevelPct70,
    LevelPct80,
    LevelPct90,
}

impl PriceBandId {
    pub(super) const fn suffix(self) -> &'static str {
        match self {
            Self::FloorPct95 => "floor_pct95",
            Self::FloorPct98 => "floor_pct98",
            Self::FloorPct99 => "floor_pct99",
            Self::FloorPct99_5 => "floor_pct99_5",
            Self::FloorPct99_9 => "floor_pct99_9",
            Self::LevelPct10 => "level_pct10",
            Self::LevelPct20 => "level_pct20",
            Self::LevelPct30 => "level_pct30",
            Self::LevelPct40 => "level_pct40",
            Self::LevelPct50 => "level_pct50",
            Self::LevelPct60 => "level_pct60",
            Self::LevelPct70 => "level_pct70",
            Self::LevelPct80 => "level_pct80",
            Self::LevelPct90 => "level_pct90",
        }
    }

    pub(super) fn select<T>(self, values: &PriceBands<T>) -> &T {
        match self {
            Self::FloorPct95 => &values.floor.pct95,
            Self::FloorPct98 => &values.floor.pct98,
            Self::FloorPct99 => &values.floor.pct99,
            Self::FloorPct99_5 => &values.floor.pct99_5,
            Self::FloorPct99_9 => &values.floor.pct99_9,
            Self::LevelPct10 => &values.level.pct10,
            Self::LevelPct20 => &values.level.pct20,
            Self::LevelPct30 => &values.level.pct30,
            Self::LevelPct40 => &values.level.pct40,
            Self::LevelPct50 => &values.level.pct50,
            Self::LevelPct60 => &values.level.pct60,
            Self::LevelPct70 => &values.level.pct70,
            Self::LevelPct80 => &values.level.pct80,
            Self::LevelPct90 => &values.level.pct90,
        }
    }

    pub(super) fn select_mut<T>(self, values: &mut PriceBands<T>) -> &mut T {
        match self {
            Self::FloorPct95 => &mut values.floor.pct95,
            Self::FloorPct98 => &mut values.floor.pct98,
            Self::FloorPct99 => &mut values.floor.pct99,
            Self::FloorPct99_5 => &mut values.floor.pct99_5,
            Self::FloorPct99_9 => &mut values.floor.pct99_9,
            Self::LevelPct10 => &mut values.level.pct10,
            Self::LevelPct20 => &mut values.level.pct20,
            Self::LevelPct30 => &mut values.level.pct30,
            Self::LevelPct40 => &mut values.level.pct40,
            Self::LevelPct50 => &mut values.level.pct50,
            Self::LevelPct60 => &mut values.level.pct60,
            Self::LevelPct70 => &mut values.level.pct70,
            Self::LevelPct80 => &mut values.level.pct80,
            Self::LevelPct90 => &mut values.level.pct90,
        }
    }

    pub(super) fn series<T>(create: impl FnMut(Self) -> T) -> PriceBands<T> {
        PriceBands::from_fn(create)
    }
}

impl From<LevelId> for PriceBandId {
    fn from(value: LevelId) -> Self {
        match value {
            LevelId::Pct10 => Self::LevelPct10,
            LevelId::Pct20 => Self::LevelPct20,
            LevelId::Pct30 => Self::LevelPct30,
            LevelId::Pct40 => Self::LevelPct40,
            LevelId::Pct50 => Self::LevelPct50,
            LevelId::Pct60 => Self::LevelPct60,
            LevelId::Pct70 => Self::LevelPct70,
            LevelId::Pct80 => Self::LevelPct80,
            LevelId::Pct90 => Self::LevelPct90,
        }
    }
}

impl ColumnId for PriceBandId {
    type Row<T>
        = PriceBands<T>
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &PRICE_BAND_IDS;

    #[inline]
    fn index(self) -> usize {
        self as usize
    }

    #[inline]
    fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T {
        self.select(row)
    }

    #[inline]
    fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
        self.select_mut(row)
    }

    #[inline]
    fn from_fn<T, F>(create: F) -> Self::Row<T>
    where
        T: VecValue,
        F: FnMut(Self) -> T,
    {
        PriceBands::from_fn(create)
    }

    #[inline]
    fn map<T, U, F>(row: Self::Row<T>, create: F) -> Self::Row<U>
    where
        T: VecValue,
        U: VecValue,
        F: FnMut(T) -> U,
    {
        row.map(create)
    }
}

#[cfg(test)]
mod tests {
    use vecdb::ColumnId;

    use super::{PRICE_BAND_IDS, PriceBandId};

    #[test]
    fn storage_order_matches_public_order() {
        assert_eq!(PriceBandId::ALL, PRICE_BAND_IDS);
        let row = PriceBandId::from_fn(|id| id);
        for &id in PriceBandId::ALL {
            assert_eq!(id.get(&row), &id);
        }
    }
}
