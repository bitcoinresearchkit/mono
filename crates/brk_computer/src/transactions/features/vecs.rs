use brk_traversable::Traversable;
use brk_types::{StoredU64, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{ColumnId, Rw, StorageMode, VecValue};

use crate::internal::{ColumnarPerBlockCumulativeRolling, LazyColumnPerBlockCumulativeRolling};

const FEATURE_COUNT: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FeatureId {
    Inscription,
    Annex,
    SighashAll,
    SighashNone,
    SighashSingle,
    SighashDefault,
    SighashAnyoneCanPay,
    DustOutput,
}

const FEATURE_IDS: [FeatureId; FEATURE_COUNT] = [
    FeatureId::Inscription,
    FeatureId::Annex,
    FeatureId::SighashAll,
    FeatureId::SighashNone,
    FeatureId::SighashSingle,
    FeatureId::SighashDefault,
    FeatureId::SighashAnyoneCanPay,
    FeatureId::DustOutput,
];

impl ColumnId for FeatureId {
    type Row<T>
        = [T; FEATURE_COUNT]
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &FEATURE_IDS;

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
        std::array::from_fn(|index| create(FEATURE_IDS[index]))
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

/// Transaction counts by detected feature.
///
/// Each metric counts transactions containing the feature at least once, not
/// individual occurrences. A transaction can contribute to multiple metrics.
#[derive(Deref, DerefMut, Traversable)]
pub struct CountVecs<M: StorageMode = Rw> {
    pub inscription: LazyColumnPerBlockCumulativeRolling<StoredU64, FeatureId>,
    pub annex: LazyColumnPerBlockCumulativeRolling<StoredU64, FeatureId>,
    /// Transactions containing at least one `SIGHASH_ALL` signature.
    pub sighash_all: LazyColumnPerBlockCumulativeRolling<StoredU64, FeatureId>,
    /// Transactions containing at least one `SIGHASH_NONE` signature.
    pub sighash_none: LazyColumnPerBlockCumulativeRolling<StoredU64, FeatureId>,
    /// Transactions containing at least one `SIGHASH_SINGLE` signature.
    pub sighash_single: LazyColumnPerBlockCumulativeRolling<StoredU64, FeatureId>,
    /// Transactions containing at least one Taproot `SIGHASH_DEFAULT` signature.
    pub sighash_default: LazyColumnPerBlockCumulativeRolling<StoredU64, FeatureId>,
    /// Transactions containing at least one `SIGHASH_ANYONECANPAY` signature.
    ///
    /// This modifier is counted independently from ALL, NONE, and SINGLE.
    pub sighash_anyone_can_pay: LazyColumnPerBlockCumulativeRolling<StoredU64, FeatureId>,
    pub dust_output: LazyColumnPerBlockCumulativeRolling<StoredU64, FeatureId>,
    #[deref]
    #[deref_mut]
    #[traversable(hidden)]
    pub source: ColumnarPerBlockCumulativeRolling<StoredU64, FeatureId, (), M>,
}

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    pub count: CountVecs<M>,
}

#[cfg(test)]
mod tests {
    use vecdb::ColumnId;

    use super::{FEATURE_IDS, FeatureId};

    #[test]
    fn feature_columns_match_public_field_order() {
        assert_eq!(FeatureId::ALL, FEATURE_IDS);
        assert_eq!(
            FeatureId::from_fn(|feature| feature),
            [
                FeatureId::Inscription,
                FeatureId::Annex,
                FeatureId::SighashAll,
                FeatureId::SighashNone,
                FeatureId::SighashSingle,
                FeatureId::SighashDefault,
                FeatureId::SighashAnyoneCanPay,
                FeatureId::DustOutput,
            ]
        );
    }
}
