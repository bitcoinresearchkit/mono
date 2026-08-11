use std::ops::AddAssign;

use brk_cohort::{ByTerm, TermId, UTXOAggregate};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, BytesVecValue, ColumnarVec, Database, ImportableVec, Rw,
    StorageMode, WritableVec,
};

#[derive(Traversable)]
pub struct AdditiveUTXORawVec<T, M: StorageMode = Rw>
where
    T: BytesVecValue,
{
    pub matrix: M::Stored<ColumnarVec<BytesVec<Height, T>, TermId>>,
}

impl<T> AdditiveUTXORawVec<T>
where
    T: BytesVecValue + AddAssign + Copy,
{
    pub fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            matrix: ImportableVec::forced_import(db, &format!("{name}_by_term"), version)?,
        })
    }

    #[inline(always)]
    pub fn push(&mut self, row: &UTXOAggregate<T>) {
        self.matrix.push(ByTerm {
            short: row.sth,
            long: row.lth,
        });
    }

    pub fn len(&self) -> usize {
        self.matrix.len()
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        &mut self.matrix
    }
}
