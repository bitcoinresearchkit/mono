use bitview_traversable::Traversable;
use brk_types::Height;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{BytesVec, BytesVecValue, Formattable, PcoVec, PcoVecValue, Rw, StorageMode, VecIndex};

#[derive(Traversable)]
pub struct AddrTypeVecs<
    I: VecIndex + PcoVecValue + Formattable + Serialize + JsonSchema,
    B: BytesVecValue + Formattable + Serialize + JsonSchema,
    M: StorageMode = Rw,
> {
    /// Zero-based type-specific address index at the start of the indexed block,
    /// equal to the number of distinct addresses of this type first seen in
    /// preceding blocks.
    pub first_index: M::Stored<PcoVec<Height, I>>,
    /// Raw locking-script payload identifying the address, with script opcodes
    /// and push-length bytes removed.
    pub bytes: M::Stored<BytesVec<I, B>>,
}
