use bitview_primitives::TypeIndex;
use rustc_hash::FxHashMap;

const RECEIVED: u8 = 1;
const SEEN_SENDING: u8 = 2;

/// One shard's receivers and senders within a block.
#[derive(Default)]
pub struct TransferAddressCache {
    addresses: FxHashMap<TypeIndex, u8>,
}

impl TransferAddressCache {
    pub fn prepare(&mut self, received: impl Iterator<Item = TypeIndex>) {
        self.addresses.clear();
        for type_index in received {
            self.addresses.insert(type_index, RECEIVED);
        }
    }

    #[inline]
    pub fn observe_send(&mut self, type_index: TypeIndex) -> (bool, bool) {
        let flags = self.addresses.entry(type_index).or_default();
        let is_first_encounter = *flags & SEEN_SENDING == 0;
        let also_received = *flags & RECEIVED != 0;
        *flags |= SEEN_SENDING;
        (is_first_encounter, also_received)
    }
}
