use brk_cohort::ByAddrType;
use brk_types::{OutputType, Sats, TypeIndex};
use rustc_hash::FxHashSet;

use crate::distribution::addr::AddrTypeToVec;

#[derive(Default)]
pub struct TransferAddressCache {
    received: ByAddrType<FxHashSet<TypeIndex>>,
    seen_senders: ByAddrType<FxHashSet<TypeIndex>>,
}

impl TransferAddressCache {
    pub fn prepare(&mut self, received_data: &AddrTypeToVec<(TypeIndex, Sats)>) {
        self.received.values_mut().for_each(FxHashSet::clear);
        self.seen_senders.values_mut().for_each(FxHashSet::clear);

        for (output_type, received) in received_data.iter() {
            self.received
                .get_mut_unwrap(output_type)
                .extend(received.iter().map(|(type_index, _)| *type_index));
        }
    }

    pub fn sets_for(
        &mut self,
        output_type: OutputType,
    ) -> (Option<&FxHashSet<TypeIndex>>, &mut FxHashSet<TypeIndex>) {
        (
            self.received.get(output_type),
            self.seen_senders.get_mut_unwrap(output_type),
        )
    }
}
