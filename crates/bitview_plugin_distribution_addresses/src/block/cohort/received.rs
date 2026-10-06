use bitview_cohort::AmountRangeId;
use bitview_primitives::TypeIndex;
use brk_types::Cents;
use rustc_hash::FxHashMap;

use crate::{
    addr::{AddrReceivePreState, AddrReceiveStatus, AddrTypeMetricsState},
    block::Received,
    state::CohortLog,
};

use super::super::cache::AddrTypeLookup;

/// Apply one address type's received outputs for a block.
pub fn process_received(
    received: FxHashMap<TypeIndex, Received>,
    cohorts: &mut CohortLog,
    lookup: &mut AddrTypeLookup<'_>,
    price: Cents,
    metrics: &mut AddrTypeMetricsState<'_>,
) {
    let output_type = metrics.output_type();
    for (type_index, recv) in received {
        let (addr_data, status) = lookup.get_or_create_for_receive(type_index);
        let pre = AddrReceivePreState::capture(addr_data, output_type);

        if matches!(status, AddrReceiveStatus::New | AddrReceiveStatus::WasEmpty) {
            addr_data.receive_outputs(recv.total_value, price, recv.output_count);
            cohorts.add(AmountRangeId::from(recv.total_value), addr_data);
        } else {
            let prev_balance = addr_data.balance();
            let new_balance = prev_balance + recv.total_value;
            let prev_bucket = AmountRangeId::from(prev_balance);
            let new_bucket = AmountRangeId::from(new_balance);

            if prev_bucket != new_bucket {
                cohorts.subtract(prev_bucket, addr_data);
                addr_data.receive_outputs(recv.total_value, price, recv.output_count);
                cohorts.add(new_bucket, addr_data);
            } else {
                cohorts.receive_outputs(
                    new_bucket,
                    addr_data,
                    recv.total_value,
                    price,
                    recv.output_count,
                );
            }
        }

        metrics.on_receive_applied(status, addr_data, &pre, recv.output_count);
    }
}
