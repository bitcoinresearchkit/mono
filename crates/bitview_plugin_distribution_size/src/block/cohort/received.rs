use bitview_cohort::AmountRangeId;
use brk_types::Cents;

use crate::{
    addr::{AddrMetricsState, AddrReceivePreState, AddrReceiveStatus, AddrTypeToTypeIndexMap},
    block::Received,
    state::AddrStates,
};

use super::super::cache::AddrLookup;

pub fn process_received(
    received: AddrTypeToTypeIndexMap<Received>,
    cohorts: &mut AddrStates,
    lookup: &mut AddrLookup<'_>,
    price: Cents,
    state: &mut AddrMetricsState,
) {
    for (output_type, received) in received.into_iter() {
        if received.is_empty() {
            continue;
        }

        let mut lookup = lookup.select(output_type);
        let mut metrics = state.select(output_type);
        for (type_index, recv) in received {
            let (addr_data, status) = lookup.get_or_create_for_receive(type_index);
            let pre = AddrReceivePreState::capture(addr_data, output_type);

            if matches!(status, AddrReceiveStatus::New | AddrReceiveStatus::WasEmpty) {
                addr_data.receive_outputs(recv.total_value, price, recv.output_count);
                cohorts
                    .amount_range
                    .get_mut(recv.total_value)
                    .add(addr_data);
            } else {
                let prev_balance = addr_data.balance();
                let new_balance = prev_balance + recv.total_value;
                let prev_bucket = AmountRangeId::from(prev_balance);
                let new_bucket = AmountRangeId::from(new_balance);

                if prev_bucket != new_bucket {
                    let cohort_state = prev_bucket.select_mut(&mut cohorts.amount_range);

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
                    new_bucket
                        .select_mut(&mut cohorts.amount_range)
                        .add(addr_data);
                } else {
                    new_bucket
                        .select_mut(&mut cohorts.amount_range)
                        .receive_outputs(addr_data, recv.total_value, price, recv.output_count);
                }
            }

            metrics.on_receive_applied(status, addr_data, &pre, recv.output_count);
        }
    }
}
