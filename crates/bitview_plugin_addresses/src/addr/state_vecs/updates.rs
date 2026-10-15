use bitview_primitives::{
    AddressState, ByAddrType, EmptyAddressData, ExtendedEmptyAddressIndex, FundedAddressData,
    FundedAddressIndex, TypeIndex,
};
use brk_types::OutputType;
use vecdb::likely;

use rustc_hash::FxHashMap;

use super::Changed;
use crate::addr::SourcedAddrData;

/// Cached addresses sorted into the writes each persistent vector needs.
#[derive(Default)]
pub struct AddrUpdates {
    pub primaries: ByAddrType<Vec<(TypeIndex, AddressState)>>,
    pub funded_updates: Vec<(FundedAddressIndex, FundedAddressData)>,
    pub funded_pushes: SidecarPushes<FundedAddressData>,
    pub funded_deletes: Vec<FundedAddressIndex>,
    pub extended_updates: Vec<(ExtendedEmptyAddressIndex, EmptyAddressData)>,
    pub extended_pushes: SidecarPushes<EmptyAddressData>,
    pub extended_deletes: Vec<ExtendedEmptyAddressIndex>,
}

/// Values a sidecar stores in new slots, apart from the addresses whose state will point to
/// them, as the sidecar and the primaries take them.
pub struct SidecarPushes<T> {
    pub addresses: Vec<(OutputType, TypeIndex)>,
    pub values: Vec<T>,
}

impl<T> Default for SidecarPushes<T> {
    fn default() -> Self {
        Self {
            addresses: Vec::new(),
            values: Vec::new(),
        }
    }
}

impl<T> SidecarPushes<T> {
    #[inline(always)]
    fn push(&mut self, addr_type: OutputType, type_index: TypeIndex, value: T) {
        self.addresses.push((addr_type, type_index));
        self.values.push(value);
    }
}

impl AddrUpdates {
    /// Records each address's final data in `changed` too, when given.
    pub fn stage(
        cache: &mut [ByAddrType<FxHashMap<TypeIndex, SourcedAddrData<FundedAddressData>>>],
        mut changed: Option<&mut Changed>,
    ) -> Self {
        let mut updates = Self::default();
        for (addr_type, entries) in cache.iter_mut().flat_map(|shard| shard.iter_mut()) {
            for (type_index, source) in entries.drain() {
                if source.is_funded() {
                    if let Some(changed) = changed.as_deref_mut() {
                        changed.funded(addr_type, type_index, &source);
                    }
                    updates.push_funded(addr_type, type_index, source);
                } else {
                    let source = SourcedAddrData::<EmptyAddressData>::from(source);
                    if let Some(changed) = changed.as_deref_mut() {
                        changed.empty(addr_type, type_index, &source);
                    }
                    updates.push_empty(addr_type, type_index, source);
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
        source: SourcedAddrData<FundedAddressData>,
    ) {
        match source {
            SourcedAddrData::New(data) | SourcedAddrData::FromInlineEmpty(data) => {
                self.funded_pushes.push(addr_type, type_index, data);
            }
            SourcedAddrData::FromFunded(index, data) => {
                self.funded_updates.push((index, data));
            }
            SourcedAddrData::FromExtendedEmpty(index, data) => {
                self.extended_deletes.push(index);
                self.funded_pushes.push(addr_type, type_index, data);
            }
        }
    }

    #[inline(always)]
    fn push_empty(
        &mut self,
        addr_type: OutputType,
        type_index: TypeIndex,
        source: SourcedAddrData<EmptyAddressData>,
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
                let state = AddressState::from_empty(&data);
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
        data: EmptyAddressData,
    ) {
        let state = AddressState::from_empty(&data);
        if likely(state.is_some()) {
            self.primaries
                .get_mut_unwrap(addr_type)
                .push((type_index, state.unwrap()));
        } else {
            self.extended_pushes.push(addr_type, type_index, data);
        }
    }
}
