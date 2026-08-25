use std::{collections::hash_map::Entry, mem};

use brk_types::{EmptyAddrData, FundedAddrData, OutputType, TypeIndex};
use rustc_hash::FxHashMap;

use crate::{
    addr::{AddrReceiveStatus, SourcedAddrData},
    block::TxIndexes,
};

use super::AddrLookup;

/// Cached address data selected for one output type.
pub struct AddrTypeLookup<'a> {
    funded: &'a mut FxHashMap<TypeIndex, SourcedAddrData<FundedAddrData>>,
    empty: &'a mut FxHashMap<TypeIndex, SourcedAddrData<EmptyAddrData>>,
}

impl AddrLookup<'_> {
    #[inline]
    pub fn select(&mut self, output_type: OutputType) -> AddrTypeLookup<'_> {
        AddrTypeLookup {
            funded: self.funded.get_mut_unwrap(output_type),
            empty: self.empty.get_mut_unwrap(output_type),
        }
    }
}

impl AddrTypeLookup<'_> {
    pub fn update_tx_counts(
        &mut self,
        mut first: FxHashMap<TypeIndex, TxIndexes>,
        mut second: FxHashMap<TypeIndex, TxIndexes>,
    ) {
        if first.len() < second.len() {
            mem::swap(&mut first, &mut second);
        }

        for (type_index, tx_indexes) in second {
            let tx_count = match first.remove(&type_index) {
                Some(other) => tx_indexes.union_len(&other),
                None => tx_indexes.len(),
            };
            self.add_tx_count(type_index, tx_count);
        }
        for (type_index, tx_indexes) in first {
            self.add_tx_count(type_index, tx_indexes.len());
        }
    }

    fn add_tx_count(&mut self, type_index: TypeIndex, tx_count: u32) {
        if let Some(addr_data) = self.funded.get_mut(&type_index) {
            addr_data.tx_count += tx_count;
        } else if let Some(addr_data) = self.empty.get_mut(&type_index) {
            addr_data.tx_count += tx_count;
        }
    }

    pub fn get_or_create_for_receive(
        &mut self,
        type_index: TypeIndex,
    ) -> (&mut SourcedAddrData<FundedAddrData>, AddrReceiveStatus) {
        match self.funded.entry(type_index) {
            Entry::Occupied(entry) => {
                let status = match entry.get() {
                    SourcedAddrData::New(data) => {
                        if data.funded_txo_count == 0 {
                            AddrReceiveStatus::New
                        } else {
                            AddrReceiveStatus::Tracked
                        }
                    }
                    SourcedAddrData::FromFunded(..) => AddrReceiveStatus::Tracked,
                    SourcedAddrData::FromInlineEmpty(data)
                    | SourcedAddrData::FromExtendedEmpty(_, data) => {
                        if data.utxo_count() == 0 {
                            AddrReceiveStatus::WasEmpty
                        } else {
                            AddrReceiveStatus::Tracked
                        }
                    }
                };
                (entry.into_mut(), status)
            }
            Entry::Vacant(entry) => {
                if let Some(empty_data) = self.empty.remove(&type_index) {
                    return (entry.insert(empty_data.into()), AddrReceiveStatus::WasEmpty);
                }
                (
                    entry.insert(SourcedAddrData::New(FundedAddrData::default())),
                    AddrReceiveStatus::New,
                )
            }
        }
    }

    #[inline]
    pub fn get_for_send(&mut self, type_index: TypeIndex) -> &mut SourcedAddrData<FundedAddrData> {
        self.funded
            .get_mut(&type_index)
            .expect("Addr must exist for send")
    }

    #[inline]
    pub fn move_to_empty(&mut self, type_index: TypeIndex) {
        let data = self.funded.remove(&type_index).unwrap();
        self.empty.insert(type_index, data.into());
    }
}
