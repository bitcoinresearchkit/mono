use bitview_cohort::ByAddrType;
use brk_types::{
    AddrState, EmptyAddrData, ExtendedEmptyAddrIndex, FundedAddrIndex, OutputType, TypeIndex,
};
use vecdb::likely;

use crate::addr::{AddrTypeToTypeIndexMap, AddrTypeToVec, SourcedAddrData};

pub struct EmptyAddrUpdates {
    pub primaries: AddrTypeToVec<(TypeIndex, AddrState)>,
    pub funded_deletes: Vec<FundedAddrIndex>,
    pub extended_updates: Vec<(ExtendedEmptyAddrIndex, EmptyAddrData)>,
    pub extended_deletes: Vec<ExtendedEmptyAddrIndex>,
    pub extended_pushes: Vec<(OutputType, TypeIndex, EmptyAddrData)>,
}

impl EmptyAddrUpdates {
    pub fn stage(
        empty: &mut AddrTypeToTypeIndexMap<SourcedAddrData<EmptyAddrData>>,
        primary_capacities: ByAddrType<usize>,
    ) -> Self {
        let mut updates = Self {
            primaries: AddrTypeToVec::with_capacities(primary_capacities),
            funded_deletes: Vec::new(),
            extended_updates: Vec::new(),
            extended_deletes: Vec::new(),
            extended_pushes: Vec::new(),
        };

        for (addr_type, entries) in empty.iter_mut() {
            for (type_index, source) in entries.drain() {
                updates.push(addr_type, type_index, source);
            }
        }
        updates
    }

    #[inline(always)]
    fn push(
        &mut self,
        addr_type: OutputType,
        type_index: TypeIndex,
        source: SourcedAddrData<EmptyAddrData>,
    ) {
        match source {
            SourcedAddrData::New(data) | SourcedAddrData::FromInlineEmpty(data) => {
                self.push_empty(addr_type, type_index, data);
            }
            SourcedAddrData::FromFunded(index, data) => {
                self.funded_deletes.push(index);
                self.push_empty(addr_type, type_index, data);
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
    fn push_empty(&mut self, addr_type: OutputType, type_index: TypeIndex, data: EmptyAddrData) {
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
