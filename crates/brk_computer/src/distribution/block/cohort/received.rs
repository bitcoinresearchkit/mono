use brk_cohort::AmountBucket;
use brk_types::{Cents, Sats, TypeIndex};
use rustc_hash::FxHashMap;

use crate::distribution::{
    AddrStates,
    addr::{AddrMetricsState, AddrReceivePreState, AddrTypeToVec},
};

use super::super::cache::{AddrLookup, TrackingStatus};

/// Aggregated receive data for a single address within a block.
#[derive(Default)]
struct AggregatedReceive {
    total_value: Sats,
    output_count: u32,
}

pub(crate) fn process_received(
    received_data: AddrTypeToVec<(TypeIndex, Sats)>,
    cohorts: &mut AddrStates,
    lookup: &mut AddrLookup<'_>,
    price: Cents,
    state: &mut AddrMetricsState,
) {
    let max_type_len = received_data
        .iter()
        .map(|(_, v)| v.len())
        .max()
        .unwrap_or(0);
    let mut aggregated: FxHashMap<TypeIndex, AggregatedReceive> =
        FxHashMap::with_capacity_and_hasher(max_type_len, Default::default());

    for (output_type, vec) in received_data.unwrap().into_iter() {
        if vec.is_empty() {
            continue;
        }

        // Aggregate per address so each address is processed exactly once.
        for (type_index, value) in vec {
            let entry = aggregated.entry(type_index).or_default();
            entry.total_value += value;
            entry.output_count += 1;
        }

        for (type_index, recv) in aggregated.drain() {
            let (addr_data, status) = lookup.get_or_create_for_receive(output_type, type_index);
            let pre = AddrReceivePreState::capture(addr_data, output_type);

            if matches!(status, TrackingStatus::New | TrackingStatus::WasEmpty) {
                addr_data.receive_outputs(recv.total_value, price, recv.output_count);
                cohorts
                    .amount_range
                    .get_mut_by_bucket(AmountBucket::from(recv.total_value))
                    .add(addr_data);
            } else {
                let prev_balance = addr_data.balance();
                let new_balance = prev_balance + recv.total_value;
                let prev_bucket = AmountBucket::from(prev_balance);
                let new_bucket = AmountBucket::from(new_balance);

                if let Some((old_bucket, new_bucket)) = prev_bucket.transition_to(new_bucket) {
                    let cohort_state = cohorts.amount_range.get_mut_by_bucket(old_bucket);

                    if cohort_state.inner.supply.utxo_count < addr_data.utxo_count() as u64 {
                        panic!(
                            "process_received: cohort underflow detected!\n\
                            output_type={:?}, type_index={:?}\n\
                            prev_balance={}, new_balance={}, total_value={}\n\
                            Addr: {:?}",
                            output_type,
                            type_index,
                            prev_balance,
                            new_balance,
                            recv.total_value,
                            addr_data
                        );
                    }

                    cohort_state.subtract(addr_data);
                    addr_data.receive_outputs(recv.total_value, price, recv.output_count);
                    cohorts
                        .amount_range
                        .get_mut_by_bucket(new_bucket)
                        .add(addr_data);
                } else {
                    cohorts
                        .amount_range
                        .get_mut_by_bucket(new_bucket)
                        .receive_outputs(addr_data, recv.total_value, price, recv.output_count);
                }
            }

            state.on_receive_applied(output_type, status, addr_data, &pre, recv.output_count);
        }
    }
}
