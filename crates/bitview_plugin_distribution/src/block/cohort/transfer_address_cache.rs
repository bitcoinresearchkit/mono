use bitview_cohort::ByAddrType;
use brk_types::{OutputType, Sats, TypeIndex};
use rustc_hash::FxHashMap;

use crate::addr::AddrTypeToVec;

const RECEIVED: u8 = 1;
const SEEN_SENDING: u8 = 2;

#[derive(Default)]
pub struct TransferAddressCache {
    addresses: ByAddrType<FxHashMap<TypeIndex, u8>>,
}

impl TransferAddressCache {
    pub fn prepare(&mut self, received_data: &AddrTypeToVec<(TypeIndex, Sats)>) {
        self.addresses.values_mut().for_each(FxHashMap::clear);

        for (output_type, received) in received_data.iter() {
            let addresses = self.addresses.get_mut_unwrap(output_type);
            for (type_index, _) in received {
                addresses.insert(*type_index, RECEIVED);
            }
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

#[cfg(test)]
mod tests {
    use brk_types::{OutputType, Sats, TypeIndex};

    use super::TransferAddressCache;
    use crate::addr::AddrTypeToVec;

    #[test]
    fn tracks_received_and_sending_per_address_type() {
        let received_index = TypeIndex::from(1_u32);
        let send_only_index = TypeIndex::from(2_u32);
        let mut received = AddrTypeToVec::default();
        received
            .get_mut_unwrap(OutputType::P2PKH)
            .extend([(received_index, Sats::_1), (received_index, Sats::_1)]);

        let mut cache = TransferAddressCache::default();
        cache.prepare(&received);

        assert_eq!(
            cache.observe_send(OutputType::P2PKH, received_index),
            (true, true)
        );
        assert_eq!(
            cache.observe_send(OutputType::P2PKH, received_index),
            (false, true)
        );
        assert_eq!(
            cache.observe_send(OutputType::P2PKH, send_only_index),
            (true, false)
        );
        assert_eq!(
            cache.observe_send(OutputType::P2PKH, send_only_index),
            (false, false)
        );
        assert_eq!(
            cache.observe_send(OutputType::P2SH, received_index),
            (true, false)
        );

        cache.prepare(&AddrTypeToVec::default());
        assert_eq!(
            cache.observe_send(OutputType::P2PKH, received_index),
            (true, false)
        );
    }
}
