use std::collections::hash_map::Entry;

use brk_types::{EmptyAddrData, FundedAddrData, OutputType, TypeIndex};
use rustc_hash::FxHashMap;

use crate::{
    addr::{AddrReceiveStatus, SourcedAddrData},
    block::{Received, TxIndexes},
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

#[cfg(test)]
mod tests {
    use brk_types::{EmptyAddrData, FundedAddrData, OutputType, Sats, TxIndex, TypeIndex};

    use crate::{
        addr::{AddrTypeToTypeIndexMap, SourcedAddrData},
        block::{AddrCache, Received, TxIndexes},
    };

    #[test]
    fn transaction_union_updates_funded_empty_and_input_only_addresses() {
        let ty = OutputType::P2WPKH;
        let (funded, empty, input_only) = (TypeIndex::new(1), TypeIndex::new(2), TypeIndex::new(3));
        let mut cache = AddrCache::default();
        let lookup = cache.as_lookup();
        lookup
            .funded
            .insert_for_type(ty, funded, SourcedAddrData::New(FundedAddrData::default()));
        lookup.empty.insert_for_type(
            ty,
            empty,
            SourcedAddrData::FromInlineEmpty(EmptyAddrData::default()),
        );
        lookup.funded.insert_for_type(
            ty,
            input_only,
            SourcedAddrData::New(FundedAddrData::default()),
        );
        let mut outputs = AddrTypeToTypeIndexMap::default();
        let mut received = Received::new(Sats::_1, TxIndex::new(10));
        received.add(Sats::_1, TxIndex::new(11));
        outputs.insert_for_type(ty, funded, received);
        outputs.insert_for_type(ty, empty, Received::new(Sats::_1, TxIndex::new(10)));
        let mut inputs = AddrTypeToTypeIndexMap::default();
        let mut sent = TxIndexes::new(TxIndex::new(11));
        sent.push(TxIndex::new(12));
        inputs.insert_for_type(ty, funded, sent);
        inputs.insert_for_type(ty, input_only, TxIndexes::new(TxIndex::new(12)));
        cache.update_tx_counts(&outputs, inputs);
        let lookup = cache.as_lookup();
        assert_eq!(lookup.funded.get_unwrap(ty)[&funded].tx_count, 3);
        assert_eq!(lookup.empty.get_unwrap(ty)[&empty].tx_count, 1);
        assert_eq!(lookup.funded.get_unwrap(ty)[&input_only].tx_count, 1);
        assert_eq!(outputs.get_unwrap(ty)[&funded].total_value, Sats::new(2));
        assert_eq!(outputs.get_unwrap(ty)[&funded].tx_indexes.len(), 2);
    }
}
