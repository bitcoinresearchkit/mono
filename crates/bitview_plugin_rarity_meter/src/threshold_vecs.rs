use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{ColumnId, VecValue};

const THRESHOLD_COUNT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExtremeThresholdId {
    Pct0_1,
    Pct0_05,
    Pct0_025,
}

const EXTREME_THRESHOLD_IDS: [ExtremeThresholdId; THRESHOLD_COUNT] = [
    ExtremeThresholdId::Pct0_1,
    ExtremeThresholdId::Pct0_05,
    ExtremeThresholdId::Pct0_025,
];

impl ExtremeThresholdId {
    pub fn series<T>(mut create: impl FnMut(Self) -> T) -> ThresholdVecs<T> {
        ThresholdVecs {
            threshold_pct0_1: create(Self::Pct0_1),
            threshold_pct0_05: create(Self::Pct0_05),
            threshold_pct0_025: create(Self::Pct0_025),
        }
    }
}

impl ColumnId for ExtremeThresholdId {
    type Row<T>
        = [T; THRESHOLD_COUNT]
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &EXTREME_THRESHOLD_IDS;

    #[inline]
    fn index(self) -> usize {
        self as usize
    }

    #[inline]
    fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T {
        &row[self.index()]
    }

    #[inline]
    fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
        &mut row[self.index()]
    }

    #[inline]
    fn from_fn<T, F>(mut create: F) -> Self::Row<T>
    where
        T: VecValue,
        F: FnMut(Self) -> T,
    {
        std::array::from_fn(|index| create(EXTREME_THRESHOLD_IDS[index]))
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

#[derive(Clone, Traversable)]
pub struct ThresholdVecs<T> {
    /// Source-value boundary for a 0.1% historical tail share.
    pub threshold_pct0_1: T,
    /// Source-value boundary for a 0.05% historical tail share.
    pub threshold_pct0_05: T,
    /// Source-value boundary for a 0.025% historical tail share.
    pub threshold_pct0_025: T,
}

#[cfg(test)]
mod tests {
    use vecdb::ColumnId;

    use super::{EXTREME_THRESHOLD_IDS, ExtremeThresholdId};

    #[test]
    fn field_order_matches_columns() {
        assert_eq!(ExtremeThresholdId::ALL, EXTREME_THRESHOLD_IDS);
        let fields = ExtremeThresholdId::series(|threshold| threshold);
        assert_eq!(fields.threshold_pct0_1, ExtremeThresholdId::Pct0_1);
        assert_eq!(fields.threshold_pct0_05, ExtremeThresholdId::Pct0_05);
        assert_eq!(fields.threshold_pct0_025, ExtremeThresholdId::Pct0_025);
    }
}
