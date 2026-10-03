use brk_error::Result;

use bitview_cohort::{AmountRangeId, ByAddrType};
use brk_types::{Cents, Sats, TypeIndex};

use crate::{
    addr::{AddrMetricsState, AddrSendPreState},
    state::AddrStates,
};

use super::{super::cache::AddrLookup, transfer_address_cache::TransferAddressCache};

pub fn process_typed_sent(
    typed: ByAddrType<Vec<(TypeIndex, Sats, Cents)>>,
    cohorts: &mut AddrStates,
    lookup: &mut AddrLookup<'_>,
    current_price: Cents,
    state: &mut AddrMetricsState,
    addresses: &mut TransferAddressCache,
) -> Result<()> {
    for (output_type, spends) in typed.into_iter() {
        let mut lookup = lookup.select(output_type);
        let mut metrics = state.select(output_type);
        for group in spends.chunk_by(|a, b| a.0 == b.0) {
            let type_index = group[0].0;
            let (mut is_first_encounter, also_received) =
                addresses.observe_send(output_type, type_index);
            let addr_data = lookup.get_for_send(type_index);
            let mut emptied = false;
            for &(_, value, prev_price) in group {
                debug_assert!(!emptied);
                let pre = AddrSendPreState::capture(addr_data, output_type);
                let prev_balance = addr_data.balance();
                let will_be_empty = addr_data.has_1_utxos();
                let prev_bucket = AmountRangeId::from(prev_balance);
                let cohort_state = prev_bucket.select_mut(&mut cohorts.amount_range);
                cohort_state.send(addr_data, value, current_price, prev_price)?;
                let new_bucket = AmountRangeId::from(addr_data.balance());
                let crossing_boundary = prev_bucket != new_bucket;
                metrics.on_send_applied(
                    addr_data,
                    &pre,
                    is_first_encounter,
                    also_received,
                    will_be_empty,
                );
                is_first_encounter = false;
                if will_be_empty || crossing_boundary {
                    cohort_state.subtract(addr_data);
                }
                if will_be_empty {
                    emptied = true;
                } else if crossing_boundary {
                    new_bucket
                        .select_mut(&mut cohorts.amount_range)
                        .add(addr_data);
                }
            }
            if emptied {
                lookup.move_to_empty(type_index);
            }
        }
    }
    Ok(())
}
