use bitview_cohort::ByAddrType;
use bitview_primitives::TypeIndex;
use brk_types::OutputType;
use rustc_hash::FxHashMap;

const RECEIVED: u8 = 1;
const SEEN_SENDING: u8 = 2;

#[derive(Default)]
pub struct TransferAddressCache {
    addresses: ByAddrType<FxHashMap<TypeIndex, u8>>,
}

impl TransferAddressCache {
    pub fn prepare(&mut self, received: impl Iterator<Item = (OutputType, TypeIndex)>) {
        self.addresses.values_mut().for_each(FxHashMap::clear);

        for (output_type, type_index) in received {
            self.addresses
                .get_mut_unwrap(output_type)
                .insert(type_index, RECEIVED);
        }
    }

    #[inline]
    pub fn observe_send(&mut self, output_type: OutputType, type_index: TypeIndex) -> (bool, bool) {
        let flags = self
            .addresses
            .get_mut_unwrap(output_type)
            .entry(type_index)
            .or_default();
        let is_first_encounter = *flags & SEEN_SENDING == 0;
        let also_received = *flags & RECEIVED != 0;
        *flags |= SEEN_SENDING;
        (is_first_encounter, also_received)
    }
}
