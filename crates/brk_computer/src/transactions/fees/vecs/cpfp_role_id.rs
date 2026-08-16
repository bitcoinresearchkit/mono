use brk_types::Version;
use vecdb::{ColumnId, VecValue};

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
