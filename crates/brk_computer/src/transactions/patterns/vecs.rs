use brk_traversable::Traversable;
use brk_types::{StoredBool, TxIndex, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{
    ColumnId, ColumnarVec, EagerVec, LazyColumnVec, PcoVec, ReadOnlyColumnarVec, Rw, StorageMode,
    VecValue,
};

use super::{CountVecs, Flags};

const PATTERN_COUNT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PatternId {
    Coinjoin,
    Consolidation,
    BatchPayout,
}

const PATTERN_IDS: [PatternId; PATTERN_COUNT] = [
    PatternId::Coinjoin,
    PatternId::Consolidation,
    PatternId::BatchPayout,
];

impl ColumnId for PatternId {
    type Row<T>
        = [T; PATTERN_COUNT]
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &PATTERN_IDS;

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
        std::array::from_fn(|index| create(PATTERN_IDS[index]))
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
    pub count: CountVecs<M>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub flags: Flags<
        LazyColumnVec<ReadOnlyColumnarVec<PcoVec<TxIndex, StoredBool>, PatternId>, PatternId>,
    >,
    #[traversable(hidden)]
    pub flags_source: M::Stored<EagerVec<ColumnarVec<PcoVec<TxIndex, StoredBool>, PatternId>>>,
}

#[cfg(test)]
mod tests {
    use vecdb::ColumnId;

    use super::{PATTERN_IDS, PatternId};

    #[test]
    fn pattern_columns_match_public_field_order() {
        assert_eq!(PatternId::ALL, PATTERN_IDS);
        assert_eq!(
            PatternId::from_fn(|pattern| pattern),
            [
                PatternId::Coinjoin,
                PatternId::Consolidation,
                PatternId::BatchPayout,
            ]
        );
    }
}
