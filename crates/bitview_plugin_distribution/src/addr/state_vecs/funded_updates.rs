use brk_types::{ExtendedEmptyAddrIndex, FundedAddrData, FundedAddrIndex, OutputType, TypeIndex};

use crate::addr::{AddrTypeToTypeIndexMap, SourcedAddrData};

pub struct FundedAddrUpdates {
    pub funded_updates: Vec<(FundedAddrIndex, FundedAddrData)>,
    pub funded_pushes: Vec<(OutputType, TypeIndex, FundedAddrData)>,
    pub extended_deletes: Vec<ExtendedEmptyAddrIndex>,
}

impl FundedAddrUpdates {
    pub fn stage(funded: &mut AddrTypeToTypeIndexMap<SourcedAddrData<FundedAddrData>>) -> Self {
        let mut updates = Self {
            funded_updates: Vec::new(),
            funded_pushes: Vec::new(),
            extended_deletes: Vec::new(),
        };

        for (addr_type, entries) in funded.iter_mut() {
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
}
