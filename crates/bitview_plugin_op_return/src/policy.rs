use bitview_traversable::Traversable;
use brk_types::OpReturnPolicyId;

#[derive(Clone, Copy, Default, Traversable)]
pub struct Policy<T> {
    pub pre_v30_standard: T,
    pub pre_v30_nonstandard: T,
    pub oversized: T,
    pub multiple: T,
}

impl<T> Policy<T> {
    pub fn new(mut create: impl FnMut(OpReturnPolicyId, &'static str) -> T) -> Self {
        Self {
            pre_v30_standard: create(OpReturnPolicyId::PreV30Standard, "pre_v30_standard"),
            pre_v30_nonstandard: create(OpReturnPolicyId::PreV30Nonstandard, "pre_v30_nonstandard"),
            oversized: create(OpReturnPolicyId::Oversized, "oversized"),
            multiple: create(OpReturnPolicyId::Multiple, "multiple"),
        }
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
