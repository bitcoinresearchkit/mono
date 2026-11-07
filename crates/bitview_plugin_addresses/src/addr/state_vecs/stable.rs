use bitview_primitives::{ByAddrType, TypeIndex};
use crate::addr::{AddrOrigin, EmptyAddressData, FundedAddressData, SourcedAddrData};
use brk_types::{Height, OutputType};
use rustc_hash::FxHashMap;

/// What the address state's lock guards beside its vectors.
#[derive(Default)]
pub struct Stable {
    /// The length the state reads through (none while a rollback's restored values or a failed
    /// write are not).
    pub written: Option<Height>,
    /// Addresses at their data in the publication, read before the vectors, whose rows the
    /// writes since rewrite: the ones an update changed, which it publishes before writing them,
    /// and the ones a catch-up's writes rewrite, held as published until its own publication.
    /// Every write under the lock clears it.
    pub changed: Option<Changed>,
}

/// Most addresses a catch-up holds at their published data, about 100 bytes each: a longer one
/// writes under the state's lock, and address reads answer that the state is updating until it
/// publishes.
pub const MAX_HELD: usize = 4_000_000;

/// Addresses with their data and whether they are funded, as readers decode them from the
/// vectors: an update's changes (the protected tail's, at most ten blocks of them), or what a
/// catch-up holds ([`MAX_HELD`] at most).
#[derive(Clone)]
pub struct Changed(ByAddrType<FxHashMap<TypeIndex, (FundedAddressData, bool)>>);

impl Changed {
    pub fn new() -> Self {
        Self(ByAddrType::from_fn(|_| FxHashMap::default()))
    }

    pub fn funded(
        &mut self,
        addr_type: OutputType,
        type_index: TypeIndex,
        data: &FundedAddressData,
    ) {
        self.0
            .get_mut_unwrap(addr_type)
            .insert(type_index, (data.clone(), true));
    }

    pub fn empty(&mut self, addr_type: OutputType, type_index: TypeIndex, data: &EmptyAddressData) {
        self.0
            .get_mut_unwrap(addr_type)
            .insert(type_index, (data.clone().into(), false));
    }

    /// Holds an address at its stored data, as a reader would decode it.
    pub fn hold(&mut self, addr_type: OutputType, type_index: TypeIndex, stored: SourcedAddrData) {
        let is_funded = matches!(stored.origin, AddrOrigin::Funded(_));
        self.0
            .get_mut_unwrap(addr_type)
            .insert(type_index, (stored.data, is_funded));
    }

    pub fn len(&self) -> usize {
        self.0.values().map(FxHashMap::len).sum()
    }

    pub fn get(
        &self,
        addr_type: OutputType,
        type_index: TypeIndex,
    ) -> Option<&(FundedAddressData, bool)> {
        self.0.get(addr_type)?.get(&type_index)
    }
}
