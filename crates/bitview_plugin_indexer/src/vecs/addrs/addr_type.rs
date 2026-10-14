use bitview_primitives::AddrHash;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{AddressBytes, Height, Version};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, BytesVecValue, Database, Formattable, ImportableVec, PcoVec,
    PcoVecValue, ReadableVec, Rw, Stamp, StorageMode, VecIndex, WritableVec,
};

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

impl<I, B> AddrTypeVecs<I, B>
where
    I: VecIndex + PcoVecValue + Formattable + Serialize + JsonSchema,
    B: BytesVecValue + Formattable + Serialize + JsonSchema,
    AddressBytes: From<B>,
{
    /// Imports the `first_{name}_address_index` and `{name}_bytes` vectors.
    pub fn import(db: &Database, name: &str, version: Version) -> Result<Self> {
        let (first_index, bytes) = parallel_import! {
            first_index = PcoVec::import(db, &format!("first_{name}_address_index"), version),
            bytes = BytesVec::import(db, &format!("{name}_bytes"), version),
        };
        Ok(Self { first_index, bytes })
    }

    pub fn truncate(&mut self, height: Height, index: I, stamp: Stamp) -> Result<()> {
        self.first_index
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.bytes.truncate_if_needed_with_stamp(index, stamp)?;
        Ok(())
    }

    pub fn stored_vecs(&self) -> [&dyn AnyStoredVec; 2] {
        [&self.first_index, &self.bytes]
    }

    pub fn stored_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 2] {
        [&mut self.first_index, &mut self.bytes]
    }

    /// Calls `f` with the hash of every address first seen at or after `height` (for rollback).
    pub fn for_each_hash_from(&self, height: Height, mut f: impl FnMut(AddrHash)) {
        if let Some(first) = self.first_index.collect_one(height) {
            self.bytes
                .for_each_range_at(first.to_usize(), self.bytes.len(), |bytes| {
                    f(AddrHash::from(&AddressBytes::from(bytes)))
                });
        }
    }
}
