use brk_types::Version;
use vecdb::{ColumnId, VecValue};

use super::Percentiles;

const PERCENTILE_COUNT: usize = 5;
const LOSS_PERCENTILE_IDS: [LossPercentileId; PERCENTILE_COUNT] = [
    LossPercentileId::Pct95,
    LossPercentileId::Pct98,
    LossPercentileId::Pct99,
    LossPercentileId::Pct99_5,
    LossPercentileId::Pct99_9,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LossPercentileId {
    Pct95,
    Pct98,
    Pct99,
    Pct99_5,
    Pct99_9,
}

impl LossPercentileId {
    pub const ALL: [Self; PERCENTILE_COUNT] = LOSS_PERCENTILE_IDS;

    pub const fn suffix(self) -> &'static str {
        match self {
            Self::Pct95 => "pct95",
            Self::Pct98 => "pct98",
            Self::Pct99 => "pct99",
            Self::Pct99_5 => "pct99_5",
            Self::Pct99_9 => "pct99_9",
        }
    }

    pub fn select<T>(self, values: &Percentiles<T>) -> &T {
        match self {
            Self::Pct95 => &values.pct95,
            Self::Pct98 => &values.pct98,
            Self::Pct99 => &values.pct99,
            Self::Pct99_5 => &values.pct99_5,
            Self::Pct99_9 => &values.pct99_9,
        }
    }

    pub fn select_mut<T>(self, values: &mut Percentiles<T>) -> &mut T {
        match self {
            Self::Pct95 => &mut values.pct95,
            Self::Pct98 => &mut values.pct98,
            Self::Pct99 => &mut values.pct99,
            Self::Pct99_5 => &mut values.pct99_5,
            Self::Pct99_9 => &mut values.pct99_9,
        }
    }

    pub fn series<T>(create: impl FnMut(Self) -> T) -> Percentiles<T> {
        Percentiles::from_fn(create)
    }
}

impl ColumnId for LossPercentileId {
    type Row<T>
        = Percentiles<T>
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &LOSS_PERCENTILE_IDS;

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
        Percentiles::from_fn(create)
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

    use super::{LOSS_PERCENTILE_IDS, LossPercentileId};

    #[test]
    fn storage_order_matches_public_order() {
        assert_eq!(LossPercentileId::ALL, LOSS_PERCENTILE_IDS);
        let row = LossPercentileId::from_fn(|id| id);
        for id in LossPercentileId::ALL {
            assert_eq!(id.get(&row), &id);
        }
    }
}
