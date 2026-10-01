#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum OpReturnPolicyId {
    PreV30Standard,
    PreV30Nonstandard,
    Oversized,
    Multiple,
}

const OP_RETURN_POLICY_COUNT: usize = OpReturnPolicyId::Multiple as usize + 1;

const OP_RETURN_POLICY_IDS: [OpReturnPolicyId; OP_RETURN_POLICY_COUNT] = [
    OpReturnPolicyId::PreV30Standard,
    OpReturnPolicyId::PreV30Nonstandard,
    OpReturnPolicyId::Oversized,
    OpReturnPolicyId::Multiple,
];

impl OpReturnPolicyId {
    pub const ALL: &'static [Self] = &OP_RETURN_POLICY_IDS;

    #[inline]
    fn index(self) -> usize {
        self as usize
    }

    #[inline]
    pub fn get<T>(self, values: &[T; OP_RETURN_POLICY_COUNT]) -> &T {
        &values[self.index()]
    }

    #[inline]
    pub fn from_fn<T, F>(f: F) -> [T; OP_RETURN_POLICY_COUNT]
    where
        F: FnMut(Self) -> T,
    {
        OP_RETURN_POLICY_IDS.map(f)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "storage")]
    use super::*;

    #[cfg(feature = "storage")]
    #[test]
    fn iteration_order_matches_discriminants() {
        for (index, policy) in OP_RETURN_POLICY_IDS.into_iter().enumerate() {
            assert_eq!(policy as usize, index);
            assert_eq!(policy.index(), index);
        }
    }
}
