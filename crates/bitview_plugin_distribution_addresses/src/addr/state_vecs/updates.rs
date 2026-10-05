use bitview_primitives::{
    AddrState, EmptyAddrData, ExtendedEmptyAddrIndex, FundedAddrData, FundedAddrIndex, TypeIndex,
};
use brk_types::OutputType;
use vecdb::likely;

use crate::addr::{AddrTypeToTypeIndexMap, AddrTypeToVec, SourcedAddrData};

/// Cached addresses sorted into the writes each persistent vector needs.
pub struct AddrUpdates {
    pub primaries: AddrTypeToVec<(TypeIndex, AddrState)>,
    pub funded_updates: Vec<(FundedAddrIndex, FundedAddrData)>,
    pub funded_pushes: Vec<(OutputType, TypeIndex, FundedAddrData)>,
    pub funded_deletes: Vec<FundedAddrIndex>,
    pub extended_updates: Vec<(ExtendedEmptyAddrIndex, EmptyAddrData)>,
    pub extended_pushes: Vec<(OutputType, TypeIndex, EmptyAddrData)>,
    pub extended_deletes: Vec<ExtendedEmptyAddrIndex>,
}

impl AddrUpdates {
    pub fn stage(cache: &mut AddrTypeToTypeIndexMap<SourcedAddrData<FundedAddrData>>) -> Self {
        let mut updates = Self {
            primaries: AddrTypeToVec::with_capacities(cache.lengths()),
            funded_updates: Vec::new(),
            funded_pushes: Vec::new(),
            funded_deletes: Vec::new(),
            extended_updates: Vec::new(),
            extended_pushes: Vec::new(),
            extended_deletes: Vec::new(),
        };

        for (addr_type, entries) in cache.iter_mut() {
            for (type_index, source) in entries.drain() {
                if source.is_funded() {
                    updates.push_funded(addr_type, type_index, source);
                } else {
                    updates.push_empty(addr_type, type_index, source.into());
                }
            }
        }
        updates
    }

    #[inline(always)]
    fn push_funded(
        &mut self,
        addr_type: OutputType,
        type_index: TypeIndex,
        source: SourcedAddrData<FundedAddrData>,
    ) {
        match source {
            SourcedAddrData::New(data) | SourcedAddrData::FromInlineEmpty(data) => {
                self.funded_pushes.push((addr_type, type_index, data));
            }
            SourcedAddrData::FromFunded(index, data) => {
                self.funded_updates.push((index, data));
            }
            SourcedAddrData::FromExtendedEmpty(index, data) => {
                self.extended_deletes.push(index);
                self.funded_pushes.push((addr_type, type_index, data));
            }
        }
    }

    #[inline(always)]
    fn push_empty(
        &mut self,
        addr_type: OutputType,
        type_index: TypeIndex,
        source: SourcedAddrData<EmptyAddrData>,
    ) {
        match source {
            SourcedAddrData::New(data) | SourcedAddrData::FromInlineEmpty(data) => {
                self.push_empty_state(addr_type, type_index, data);
            }
            SourcedAddrData::FromFunded(index, data) => {
                self.funded_deletes.push(index);
                self.push_empty_state(addr_type, type_index, data);
            }
            SourcedAddrData::FromExtendedEmpty(index, data) => {
                let state = AddrState::from_empty(&data);
                if likely(state.is_some()) {
                    self.extended_deletes.push(index);
                    self.primaries
                        .get_mut_unwrap(addr_type)
                        .push((type_index, state.unwrap()));
                } else {
                    self.extended_updates.push((index, data));
                }
            }
        }
    }

    #[inline(always)]
    fn push_empty_state(
        &mut self,
        addr_type: OutputType,
        type_index: TypeIndex,
        data: EmptyAddrData,
    ) {
        let state = AddrState::from_empty(&data);
        if likely(state.is_some()) {
            self.primaries
                .get_mut_unwrap(addr_type)
                .push((type_index, state.unwrap()));
        } else {
            self.extended_pushes.push((addr_type, type_index, data));
        }
    }
}
