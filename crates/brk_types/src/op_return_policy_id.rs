use vecdb::{ColumnId, VecValue, Version};

pub const OP_RETURN_POLICY_COUNT: usize = OpReturnPolicyId::Multiple as usize + 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum OpReturnPolicyId {
    PreV30Standard,
    PreV30Nonstandard,
    Oversized,
    Multiple,
}

pub const OP_RETURN_POLICY_IDS: [OpReturnPolicyId; OP_RETURN_POLICY_COUNT] = [
    OpReturnPolicyId::PreV30Standard,
    OpReturnPolicyId::PreV30Nonstandard,
    OpReturnPolicyId::Oversized,
    OpReturnPolicyId::Multiple,
];

impl ColumnId for OpReturnPolicyId {
    type Row<T>
        = [T; OP_RETURN_POLICY_COUNT]
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &OP_RETURN_POLICY_IDS;

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
    fn from_fn<T, F>(f: F) -> Self::Row<T>
    where
        T: VecValue,
        F: FnMut(Self) -> T,
    {
        OP_RETURN_POLICY_IDS.map(f)
    }

    #[inline]
    fn map<T, U, F>(row: Self::Row<T>, f: F) -> Self::Row<U>
    where
        T: VecValue,
        U: VecValue,
        F: FnMut(T) -> U,
    {
        row.map(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_order_matches_discriminants() {
        for (index, policy) in OP_RETURN_POLICY_IDS.into_iter().enumerate() {
            assert_eq!(policy as usize, index);
            assert_eq!(policy.index(), index);
        }
    }
}
