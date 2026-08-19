use bitview_traversable::Traversable;
use brk_types::{StoredU64, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{ColumnId, Rw, StorageMode, VecValue};

use bitview_compute::{ColumnarPerBlockCumulativeRolling, LazyColumnPerBlockCumulativeRolling};

const VERSION_COUNT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VersionId {
    V1,
    V2,
    V3,
    Other,
}

const VERSION_IDS: [VersionId; VERSION_COUNT] = [
    VersionId::V1,
    VersionId::V2,
    VersionId::V3,
    VersionId::Other,
];

impl ColumnId for VersionId {
    type Row<T>
        = [T; VERSION_COUNT]
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &VERSION_IDS;

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
        std::array::from_fn(|index| create(VERSION_IDS[index]))
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

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Transactions whose version is exactly 1.
    pub v1: LazyColumnPerBlockCumulativeRolling<StoredU64, VersionId>,
    /// Transactions whose version is exactly 2.
    pub v2: LazyColumnPerBlockCumulativeRolling<StoredU64, VersionId>,
    /// Transactions whose version is exactly 3.
    pub v3: LazyColumnPerBlockCumulativeRolling<StoredU64, VersionId>,
    /// Transactions whose version is not 1, 2, or 3. This category combines
    /// every other value; use individual raw transaction data to inspect the
    /// original version.
    pub other: LazyColumnPerBlockCumulativeRolling<StoredU64, VersionId>,
    #[deref]
    #[deref_mut]
    #[traversable(hidden)]
    pub source: ColumnarPerBlockCumulativeRolling<StoredU64, VersionId, (), M>,
}

#[cfg(test)]
mod tests {
    use vecdb::ColumnId;

    use super::{VERSION_IDS, VersionId};

    #[test]
    fn version_columns_match_public_field_order() {
        assert_eq!(VersionId::ALL, VERSION_IDS);
        assert_eq!(
            VersionId::from_fn(|version| version),
            [
                VersionId::V1,
                VersionId::V2,
                VersionId::V3,
                VersionId::Other
            ]
        );
    }
}
