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

#[cfg(test)]
mod tests {
    use super::process_typed_sent;
    use crate::{
        addr::{AddrMetricsState, AddrTypeToTypeIndexMap},
        block::{AddrCache, Received, TransferAddressCache, process_inputs, process_received},
        state::{AddrStates, RealizedOps},
    };
    use brk_types::{Cents, Height, OutputType, Sats, TxIndex, TypeIndex};

    #[test]
    fn interleaved_spends_count_each_typed_address_once_and_empty_it() {
        let mut cohorts = AddrStates::new();
        let mut cache = AddrCache::default();
        let mut metrics = AddrMetricsState::default();
        let addresses = [
            (OutputType::P2PKH, 7),
            (OutputType::P2PKH, 8),
            (OutputType::P2TR, 7),
        ];
        for (tx, price) in [(0, 100), (1, 150)] {
            let mut funded = AddrTypeToTypeIndexMap::default();
            for (ty, index) in addresses {
                funded.insert_for_type(
                    ty,
                    TypeIndex::new(index),
                    Received::new(Sats::ONE_BTC, TxIndex::new(tx)),
                );
            }
            process_received(
                funded,
                &mut cohorts,
                &mut cache.as_lookup(),
                Cents::new(price),
                &mut metrics,
            );
        }
        metrics.reset_per_block();

        // A zero-value output still makes its address bidirectional, and must be spent.
        let mut received = AddrTypeToTypeIndexMap::default();
        received.insert_for_type(
            OutputType::P2PKH,
            TypeIndex::new(7),
            Received::new(Sats::ZERO, TxIndex::new(2)),
        );
        let mut transfers = TransferAddressCache::default();
        transfers.prepare([(OutputType::P2PKH, TypeIndex::new(7))].into_iter());
        process_received(
            received,
            &mut cohorts,
            &mut cache.as_lookup(),
            Cents::new(200),
            &mut metrics,
        );

        let rows = [
            (0, OutputType::P2PKH, 7, Sats::ONE_BTC, 10),
            (0, OutputType::P2PKH, 8, Sats::ONE_BTC, 10),
            (1, OutputType::P2PKH, 7, Sats::ONE_BTC, 11),
            (0, OutputType::P2TR, 7, Sats::ONE_BTC, 11),
            (2, OutputType::P2PKH, 7, Sats::ZERO, 12),
            (1, OutputType::P2PKH, 8, Sats::ONE_BTC, 13),
            (1, OutputType::P2TR, 7, Sats::ONE_BTC, 13),
            (2, OutputType::OpReturn, 7, Sats::ZERO, 14),
        ];
        let inputs = process_inputs(
            rows.into_iter().map(|row| TxIndex::new(row.4)),
            &rows.map(|row| row.3),
            &rows.map(|row| row.1),
            &rows.map(|row| TypeIndex::new(row.2)),
            &rows.map(|row| Height::new(row.0)),
        );
        for (ty, index) in addresses {
            assert_eq!(
                inputs.tx_index_vecs.get_unwrap(ty)[&TypeIndex::new(index)].len(),
                if ty == OutputType::P2PKH && index == 7 {
                    3
                } else {
                    2
                },
            );
        }
        let sent = inputs
            .sent_data
            .into_typed(&[100, 150, 200].map(Cents::new));
        let mut pkh = sent.get_unwrap(OutputType::P2PKH).clone();
        pkh.sort_by_key(|&(index, _, price)| (index, price));
        assert_eq!(
            pkh,
            [
                (7, Sats::ONE_BTC, 100),
                (7, Sats::ONE_BTC, 150),
                (7, Sats::ZERO, 200),
                (8, Sats::ONE_BTC, 100),
                (8, Sats::ONE_BTC, 150)
            ]
            .map(|(index, value, price)| (
                TypeIndex::new(index),
                value,
                Cents::new(price)
            )),
        );
        process_typed_sent(
            sent,
            &mut cohorts,
            &mut cache.as_lookup(),
            Cents::new(200),
            &mut metrics,
            &mut transfers,
        )
        .unwrap();

        for (ty, expected) in [(OutputType::P2PKH, 2), (OutputType::P2TR, 1)] {
            assert_eq!(*metrics.funded.get_unwrap(ty), 0);
            assert_eq!(*metrics.empty.get_unwrap(ty), expected);
            assert_eq!(u64::from(metrics.activity.get_unwrap(ty).sending), expected);
            assert_eq!(*metrics.reused.active.get_unwrap(ty), expected);
            assert_eq!(*metrics.respent.active.get_unwrap(ty), expected);
            assert_eq!(*metrics.exposed.funded.get_unwrap(ty), 0);
        }
        assert_eq!(
            metrics.activity.get_unwrap(OutputType::P2PKH).bidirectional,
            1
        );
        assert_eq!(
            metrics.activity.get_unwrap(OutputType::P2TR).bidirectional,
            0
        );
        for (ty, index) in addresses {
            let lookup = cache.as_lookup();
            assert!(
                !lookup
                    .funded
                    .get_unwrap(ty)
                    .contains_key(&TypeIndex::new(index))
            );
            let empty = &lookup.empty.get_unwrap(ty)[&TypeIndex::new(index)];
            assert_eq!(empty.transfered, Sats::ONE_BTC + Sats::ONE_BTC);
            assert_eq!(
                empty.funded_txo_count,
                if ty == OutputType::P2PKH && index == 7 {
                    3
                } else {
                    2
                }
            );
        }
        let mut profit = Cents::ZERO;
        for cohort in cohorts.amount_range.iter() {
            assert_eq!(cohort.addr_count, 0);
            assert_eq!(cohort.inner.supply.value, Sats::ZERO);
            assert_eq!(cohort.inner.supply.utxo_count, 0);
            assert_eq!(cohort.inner.realized.cap(), Cents::ZERO);
            profit += cohort.inner.realized.profit();
        }
        assert_eq!(profit, Cents::new(450));
    }
}
