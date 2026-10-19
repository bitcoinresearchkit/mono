use bitview_traversable::Traversable;
use brk_types::{Address, AddressBytes, Version};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    Formattable, IndexVec, LazyVec, ReadableBoxedVec, ReadableCloneableVec, VecIndex, VecValue,
};

#[derive(Clone, Traversable)]
pub struct AddressVecs<I, B>
where
    I: VecIndex + Formattable + Serialize + JsonSchema,
    B: VecValue,
{
    /// Zero-based type-specific address index assigned in first-seen canonical
    /// chain order.
    pub identity: IndexVec<I, I, ReadableBoxedVec<I, B>>,
    /// Textual identifier reconstructed from the indexed locking script: raw
    /// public-key hex for P2PK, otherwise the standard mainnet Bitcoin address.
    pub address: LazyVec<I, Address, B>,
}

impl<I, B> AddressVecs<I, B>
where
    I: VecIndex + Formattable + Serialize + JsonSchema,
    B: VecValue,
    AddressBytes: From<B>,
{
    pub fn new(prefix: &str, version: Version, bytes: &impl ReadableCloneableVec<I, B>) -> Self {
        Self {
            identity: IndexVec::new(
                &format!("{prefix}_address_index"),
                version,
                bytes.read_only_boxed_clone(),
                |index| index,
            ),
            address: LazyVec::new(
                &format!("{prefix}_address"),
                version,
                bytes.read_only_boxed_clone(),
                |_, bytes| Address::try_from(&AddressBytes::from(bytes)).unwrap(),
            ),
        }
    }
}
