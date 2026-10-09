use bitview_primitives::{FundedAddrData, TypeIndex};
use rustc_hash::FxHashMap;

use crate::{
    addr::{AddrReceiveStatus, SourcedAddrData},
    block::{Received, TxIndexes},
};

/// Cached address data of one output type.
pub struct AddrTypeLookup<'a> {
    addrs: &'a mut FxHashMap<TypeIndex, SourcedAddrData<FundedAddrData>>,
}

impl<'a> AddrTypeLookup<'a> {
    pub fn new(addrs: &'a mut FxHashMap<TypeIndex, SourcedAddrData<FundedAddrData>>) -> Self {
        Self { addrs }
    }

    pub fn update_tx_counts(
        &mut self,
        outputs: &FxHashMap<TypeIndex, Received>,
        mut inputs: FxHashMap<TypeIndex, TxIndexes>,
    ) {
        for (&type_index, received) in outputs {
            let tx_count = match inputs.remove(&type_index) {
                Some(sent) => received.tx_indexes.union_len(&sent),
                None => received.tx_indexes.len(),
            };
            self.add_tx_count(type_index, tx_count);
        }
        for (type_index, tx_indexes) in inputs {
            self.add_tx_count(type_index, tx_indexes.len());
        }
    }

    fn add_tx_count(&mut self, type_index: TypeIndex, tx_count: u32) {
        self.addrs
            .entry(type_index)
            .or_insert_with(|| SourcedAddrData::New(FundedAddrData::default()))
            .tx_count += tx_count;
    }

    pub fn get_or_create_for_receive(
        &mut self,
        type_index: TypeIndex,
    ) -> (&mut SourcedAddrData<FundedAddrData>, AddrReceiveStatus) {
        let addr_data = self
            .addrs
            .entry(type_index)
            .or_insert_with(|| SourcedAddrData::New(FundedAddrData::default()));
        let status = if addr_data.funded_txo_count == 0 {
            AddrReceiveStatus::New
        } else if addr_data.is_funded() {
            AddrReceiveStatus::Tracked
        } else {
            AddrReceiveStatus::WasEmpty
        };
        (addr_data, status)
    }

    #[inline]
    pub fn get_for_send(&mut self, type_index: TypeIndex) -> &mut SourcedAddrData<FundedAddrData> {
        self.addrs
            .get_mut(&type_index)
            .expect("Addr must exist for send")
    }
}
