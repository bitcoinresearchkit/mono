use bitview_primitives::OpReturnPolicyId;
use bitview_traversable::Traversable;

#[derive(Clone, Copy, Default, Traversable)]
pub struct Policy<T> {
    /// Restricted to transactions with exactly one OP_RETURN output containing
    /// at most 82 post-OP_RETURN bytes, the pre-v30 standard relay shape.
    pub pre_v30_standard: T,
    /// Restricted to transactions with an oversized OP_RETURN output or more
    /// than one OP_RETURN output, the pre-v30 nonstandard relay shape.
    pub pre_v30_nonstandard: T,
    /// Restricted to transactions with at least one OP_RETURN output containing
    /// more than 82 post-OP_RETURN bytes.
    pub oversized: T,
    /// Restricted to transactions containing more than one OP_RETURN output.
    pub multiple: T,
}

impl<T> Policy<T> {
    pub fn try_new<E>(
        mut create: impl FnMut(OpReturnPolicyId, &'static str) -> Result<T, E>,
    ) -> Result<Self, E> {
        Ok(Self {
            pre_v30_standard: create(OpReturnPolicyId::PreV30Standard, "pre_v30_standard")?,
            pre_v30_nonstandard: create(
                OpReturnPolicyId::PreV30Nonstandard,
                "pre_v30_nonstandard",
            )?,
            oversized: create(OpReturnPolicyId::Oversized, "oversized")?,
            multiple: create(OpReturnPolicyId::Multiple, "multiple")?,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [
            &self.pre_v30_standard,
            &self.pre_v30_nonstandard,
            &self.oversized,
            &self.multiple,
        ]
        .into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [
            &mut self.pre_v30_standard,
            &mut self.pre_v30_nonstandard,
            &mut self.oversized,
            &mut self.multiple,
        ]
        .into_iter()
    }

    pub fn get(&self, policy: OpReturnPolicyId) -> &T {
        match policy {
            OpReturnPolicyId::PreV30Standard => &self.pre_v30_standard,
            OpReturnPolicyId::PreV30Nonstandard => &self.pre_v30_nonstandard,
            OpReturnPolicyId::Oversized => &self.oversized,
            OpReturnPolicyId::Multiple => &self.multiple,
        }
    }
}
