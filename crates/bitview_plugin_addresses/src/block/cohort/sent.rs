use bitview_cohort::AmountRangeId;
use bitview_primitives::TypeIndex;
use brk_error::Result;
use brk_types::{Cents, CentsSats, Sats};

use super::{super::cache::AddrTypeLookup, transfer_address_cache::TransferAddressCache};
use crate::{
    addr::{AddrSendPreState, AddrTypeMetricsState},
    state::CohortLog,
};

/// Apply one shard's spends for a block, in the block's established spend order.
pub fn process_sent(
    spends: &[(TypeIndex, Sats, Cents)],
    cohorts: &mut CohortLog,
    lookup: &mut AddrTypeLookup<'_>,
    current_price: Cents,
    metrics: &mut AddrTypeMetricsState<'_>,
    addresses: &mut TransferAddressCache,
) -> Result<()> {
    let output_type = metrics.output_type();
    for group in spends.chunk_by(|a, b| a.0 == b.0) {
        let type_index = group[0].0;
        let (mut is_first_encounter, also_received) = addresses.observe_send(type_index);
        let addr_data = lookup.get_for_send(type_index);
        for &(_, value, prev_price) in group {
            debug_assert!(addr_data.is_funded());
            let pre = AddrSendPreState::capture(addr_data, output_type);
            let prev_balance = addr_data.balance();
            let will_be_empty = addr_data.has_1_utxos();
            let prev_bucket = AmountRangeId::from(prev_balance);
            cohorts.send(prev_bucket, addr_data, value, current_price, prev_price)?;
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
                cohorts.subtract(prev_bucket, addr_data);
            }
            if will_be_empty {
                // Exact cost basis: an emptied address matches its stored empty form.
                debug_assert_eq!(addr_data.realized_cap_raw(), CentsSats::ZERO);
            } else if crossing_boundary {
                cohorts.add(new_bucket, addr_data);
            }
        }
    }
    Ok(())
}
