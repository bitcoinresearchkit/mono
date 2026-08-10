use brk_traversable::Traversable;
use brk_types::{FeeRate, Sats, StoredBool, StoredU64, TxIndex, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{
    ColumnId, ColumnarVec, EagerVec, LazyColumnVec, PcoVec, ReadOnlyColumnarVec, Rw, StorageMode,
    VecValue,
};

use crate::internal::{
    ColumnarPerBlockCumulativeRolling, LazyColumnPerBlockCumulativeRolling, PerTxDistribution,
};

const CPFP_ROLE_COUNT: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CpfpRoleId {
    Parent,
    Child,
}

const CPFP_ROLE_IDS: [CpfpRoleId; CPFP_ROLE_COUNT] = [CpfpRoleId::Parent, CpfpRoleId::Child];

impl ColumnId for CpfpRoleId {
    type Row<T>
        = [T; CPFP_ROLE_COUNT]
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &CPFP_ROLE_IDS;

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
        std::array::from_fn(|index| create(CPFP_ROLE_IDS[index]))
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
pub struct CpfpFlags<V> {
    pub is_cpfp_parent: V,
    pub is_cpfp_child: V,
}

/// Confirmed transactions participating in same-block CPFP clusters.
#[derive(Deref, DerefMut, Traversable)]
pub struct CountVecs<M: StorageMode = Rw> {
    pub cpfp_parent: LazyColumnPerBlockCumulativeRolling<StoredU64, CpfpRoleId>,
    pub cpfp_child: LazyColumnPerBlockCumulativeRolling<StoredU64, CpfpRoleId>,
    #[deref]
    #[deref_mut]
    #[traversable(hidden)]
    pub source: ColumnarPerBlockCumulativeRolling<StoredU64, CpfpRoleId, (), M>,
}

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    pub count: CountVecs<M>,
    pub input_value: M::Stored<EagerVec<PcoVec<TxIndex, Sats>>>,
    pub output_value: M::Stored<EagerVec<PcoVec<TxIndex, Sats>>>,
    pub fee: PerTxDistribution<Sats, M>,
    pub fee_rate: M::Stored<EagerVec<PcoVec<TxIndex, FeeRate>>>,
    pub effective_fee_rate: PerTxDistribution<FeeRate, M>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub cpfp_flags: CpfpFlags<
        LazyColumnVec<ReadOnlyColumnarVec<PcoVec<TxIndex, StoredBool>, CpfpRoleId>, CpfpRoleId>,
    >,
    #[traversable(hidden)]
    pub cpfp_flags_source:
        M::Stored<EagerVec<ColumnarVec<PcoVec<TxIndex, StoredBool>, CpfpRoleId>>>,
}

#[cfg(test)]
mod tests {
    use vecdb::ColumnId;

    use super::{CPFP_ROLE_IDS, CpfpRoleId};

    #[test]
    fn cpfp_role_columns_match_public_field_order() {
        assert_eq!(CpfpRoleId::ALL, CPFP_ROLE_IDS);
        assert_eq!(
            CpfpRoleId::from_fn(|role| role),
            [CpfpRoleId::Parent, CpfpRoleId::Child]
        );
    }
}
