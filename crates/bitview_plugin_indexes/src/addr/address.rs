use bitview_traversable::Traversable;
use brk_types::Addr;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{Formattable, LazyVec, VecIndex, VecValue};

#[derive(Clone, Traversable)]
pub struct AddressVecs<I, B>
where
    I: VecIndex + Formattable + Serialize + JsonSchema,
    B: VecValue,
{
    /// Zero-based type-specific address index assigned in first-seen canonical
    /// chain order.
    pub identity: LazyVec<I, I, I, B>,
    /// Textual identifier reconstructed from the indexed locking script: raw
    /// public-key hex for P2PK, otherwise the standard mainnet Bitcoin address.
    pub addr: LazyVec<I, Addr, I, B>,
}
