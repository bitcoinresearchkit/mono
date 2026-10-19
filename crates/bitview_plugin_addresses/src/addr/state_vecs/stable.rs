use bitview_primitives::{ByAddrType, EmptyAddressData, FundedAddressData, TypeIndex};
use brk_types::{Height, OutputType};
use rustc_hash::FxHashMap;

/// What the address state's lock guards beside its vectors.
#[derive(Default)]
pub struct Stable {
    /// The length the state reads through (none while a rollback's restored values or a failed
    /// write are not).
    pub written: Option<Height>,
    /// The addresses the publication changed, read before the vectors: an update publishes them
    /// and writes the vectors after. Every other write under the lock clears it.
    pub changed: Option<Changed>,
}

/// Addresses an update changed, with their data and whether they are funded, as readers decode
/// them from the vectors: the protected tail's, at most ten blocks of them.
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

    pub fn get(
        &self,
        addr_type: OutputType,
        type_index: TypeIndex,
    ) -> Option<&(FundedAddressData, bool)> {
        self.0.get(addr_type)?.get(&type_index)
    }
}
