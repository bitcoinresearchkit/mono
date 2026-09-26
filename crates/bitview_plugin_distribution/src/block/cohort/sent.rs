use brk_error::Result;

use bitview_cohort::AmountRangeId;
use brk_types::{Cents, Height, OutputType, Sats, TypeIndex};
use rustc_hash::FxHashMap;
use vecdb::VecIndex;

use crate::{
    addr::{AddrMetricsState, AddrSendPreState},
    state::AddrStates,
};

use super::{super::cache::AddrLookup, transfer_address_cache::TransferAddressCache};

/// Process sent UTXOs for address cohort membership and empty-address transitions.
pub fn process_sent(
    sent_data: FxHashMap<Height, Vec<(OutputType, TypeIndex, Sats)>>,
    cohorts: &mut AddrStates,
    lookup: &mut AddrLookup<'_>,
    current_price: Cents,
    state: &mut AddrMetricsState,
    addresses: &mut TransferAddressCache,
    height_to_price: &[Cents],
) -> Result<()> {
    for (receive_height, spends) in sent_data {
        let prev_price = height_to_price[receive_height.to_usize()];

        for (output_type, type_index, value) in spends {
            let mut lookup = lookup.select(output_type);
            let mut metrics = state.select(output_type);

            let addr_data = lookup.get_for_send(type_index);
            let pre = AddrSendPreState::capture(addr_data, output_type);

            let prev_balance = addr_data.balance();
            let (is_first_encounter, also_received) =
                addresses.observe_send(output_type, type_index);
            let will_be_empty = addr_data.has_1_utxos();

            let prev_bucket = AmountRangeId::from(prev_balance);
            let cohort_state = prev_bucket.select_mut(&mut cohorts.amount_range);

            // Mutates addr_data.spent_txo_count (+= 1). on_send_applied reads the post-spend view.
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

            if will_be_empty || crossing_boundary {
                cohort_state.subtract(addr_data);
            }
            if will_be_empty {
                lookup.move_to_empty(type_index);
            } else if crossing_boundary {
                new_bucket
                    .select_mut(&mut cohorts.amount_range)
                    .add(addr_data);
            }
        }
    }

    Ok(())
}
