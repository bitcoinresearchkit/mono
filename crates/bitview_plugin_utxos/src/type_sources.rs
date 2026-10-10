use crate::Vecs;
use bitview_cohort::{AddrTypeId, ByAddrType};
use brk_types::{Height, Sats};
use vecdb::{ReadableBoxedVec, ReadableCloneableVec, StorageMode};
impl<M: StorageMode> Vecs<M> {
    pub fn type_supply(&self) -> ByAddrType<ReadableBoxedVec<Height, Sats>> {
        AddrTypeId::series(|id, _| {
            self.cohorts
                .types
                .get(id.output_type())
                .supply
                .total
                .value
                .sats
                .height
                .read_only_boxed_clone()
        })
    }
}
