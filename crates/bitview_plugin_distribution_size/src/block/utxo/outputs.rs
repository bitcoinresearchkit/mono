use std::collections::hash_map::Entry;

use brk_types::TxIndex;

use crate::{addr::AddrTypeToTypeIndexMap, block::Received, compute::TxOutData, state::Transacted};

/// Result of processing outputs for a block.
pub struct OutputsResult {
    /// Aggregated supply transacted in this block.
    pub transacted: Transacted,
    /// Value, output count and transaction indexes grouped once per address.
    pub received: AddrTypeToTypeIndexMap<Received>,
}

/// Process outputs (new UTXOs) for a block.
///
/// For each output:
/// 1. Read pre-collected value, output type, and type_index
/// 2. Accumulate into Transacted by type and amount
/// 3. Track address-specific data for address cohort processing
pub fn process_outputs(
    txout_index_to_tx_index: &[TxIndex],
    txout_data_vec: &[TxOutData],
) -> OutputsResult {
    let output_count = txout_data_vec.len();
    debug_assert_eq!(txout_index_to_tx_index.len(), output_count);
    let txout_index_to_tx_index = &txout_index_to_tx_index[..output_count];

    let estimated_per_type = (output_count / 8).max(8);
    let mut transacted = Transacted::default();
    let mut received = AddrTypeToTypeIndexMap::<Received>::with_capacity(estimated_per_type);

    for local_idx in 0..output_count {
        let txout_data = &txout_data_vec[local_idx];
        let value = txout_data.value;
        let output_type = txout_data.output_type;
        transacted.iterate(value, output_type);

        if output_type.is_not_addr() {
            continue;
        }

        let type_index = txout_data.type_index;
        match received.get_mut(output_type).unwrap().entry(type_index) {
            Entry::Occupied(mut entry) => {
                entry
                    .get_mut()
                    .add(value, txout_index_to_tx_index[local_idx]);
            }
            Entry::Vacant(entry) => {
                entry.insert(Received::new(value, txout_index_to_tx_index[local_idx]));
            }
        }
    }

    OutputsResult {
        transacted,
        received,
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{
        Cents, DecodedAddrState, FundedAddrData, OutputType, Sats, TxIndex, TypeIndex, Version,
    };
    use rayon::prelude::*;
    use tempfile::tempdir;
    use vecdb::{Bytes, Database, ReadableVec, Stamp};

    use super::process_outputs;
    use crate::{
        addr::{AddrMetricsState, AddrStateVecs, AddrTypeToTypeIndexMap, SourcedAddrData},
        block::{AddrCache, TxIndexes, process_received},
        compute::TxOutData,
        state::AddrStates,
        test_cache,
    };

    #[test]
    fn groups_value_count_and_unique_transactions_per_typed_address() {
        let index = TypeIndex::from(7_u32);
        let rows = [
            (OutputType::P2PKH, 3),
            (OutputType::P2PKH, 4),
            (OutputType::P2PKH, 5),
            (OutputType::P2SH, 6),
            (OutputType::OpReturn, 8),
        ]
        .map(|(output_type, value)| TxOutData {
            value: Sats::new(value),
            output_type,
            type_index: index,
        });
        let txs = [10, 10, 11, 11, 12].map(TxIndex::new);
        let result = process_outputs(&txs, &rows);
        let received = &result.received.get_unwrap(OutputType::P2PKH)[&index];
        assert_eq!(received.total_value, Sats::new(12));
        assert_eq!(received.output_count, 3);
        assert_eq!(received.tx_indexes.len(), 2);
        assert_eq!(
            result.received.get_unwrap(OutputType::P2SH)[&index].total_value,
            Sats::new(6)
        );
        assert_eq!(
            result
                .received
                .iter()
                .map(|(_, entries)| entries.len())
                .sum::<usize>(),
            2
        );
        assert!(
            process_outputs(&[], &[])
                .received
                .iter()
                .all(|(_, entries)| entries.is_empty())
        );
    }

    #[test]
    fn grouped_receives_survive_flush_resume_and_address_rollback() {
        test_cache::init_cache();
        let dir = tempdir().unwrap();
        let mut cohorts = AddrStates::new();
        let mut metrics = AddrMetricsState::default();
        let db_path = dir.path().join("addresses");
        let ty = OutputType::P2PKH;
        let rows = [(0, 3), (0, 4), (0, 5), (1, 6)].map(|(id, value)| TxOutData {
            value: Sats::new(value),
            output_type: ty,
            type_index: TypeIndex::new(id),
        });
        let txs = [10, 10, 11, 11].map(TxIndex::new);
        let mut first = Vec::new();

        for block in 1..=2 {
            let db = Database::open(&db_path).unwrap();
            let mut stored = AddrStateVecs::forced_import(&db, Version::ONE).unwrap();
            let mut cache = AddrCache::default();
            for id in 0..2 {
                let id = TypeIndex::new(id);
                let data = if block == 1 {
                    SourcedAddrData::New(FundedAddrData::default())
                } else {
                    let DecodedAddrState::Funded(index) = stored.get_once(ty, id).unwrap().decode()
                    else {
                        panic!("expected persisted funded address");
                    };
                    SourcedAddrData::FromFunded(index, stored.funded.collect_one(index).unwrap())
                };
                cache.as_lookup().funded.insert_for_type(ty, id, data);
            }
            let output = process_outputs(&txs, &rows);
            let mut inputs = AddrTypeToTypeIndexMap::default();
            let mut sent = TxIndexes::new(TxIndex::new(11));
            sent.push(TxIndex::new(12));
            inputs.insert_for_type(ty, TypeIndex::new(0), sent);
            cache.update_tx_counts(&output.received, inputs);
            metrics.reset_per_block();
            process_received(
                output.received,
                &mut cohorts,
                &mut cache.as_lookup(),
                Cents::new(100),
                &mut metrics,
            );
            for id in 0..2 {
                let lookup = cache.as_lookup();
                let data = &lookup.funded.get_unwrap(ty)[&TypeIndex::new(id)];
                let count = if id == 0 { 3 } else { 1 };
                assert_eq!(data.tx_count, block * count);
                assert_eq!(data.funded_txo_count, block * count);
                assert_eq!(
                    data.received,
                    Sats::new(u64::from(block) * if id == 0 { 12 } else { 6 })
                );
                if block == 1 {
                    first.push(data.to_bytes().to_vec());
                }
            }
            cache.flush_into(&mut stored).unwrap();
            stored
                .par_iter_mut()
                .try_for_each(|vec| {
                    vec.any_stamped_write_with_changes(Stamp::new(u64::from(block)))
                })
                .unwrap();
        }

        let db = Database::open(&db_path).unwrap();
        let mut stored = AddrStateVecs::forced_import(&db, Version::ONE).unwrap();
        for id in 0..2 {
            let DecodedAddrState::Funded(index) =
                stored.get_once(ty, TypeIndex::new(id)).unwrap().decode()
            else {
                panic!("expected persisted funded address");
            };
            assert_eq!(
                stored.funded.collect_one(index).unwrap().tx_count,
                if id == 0 { 6 } else { 2 }
            );
        }
        assert!(
            stored
                .rollback_before(Stamp::new(2))
                .unwrap()
                .into_iter()
                .all(|stamp| stamp == Stamp::new(1))
        );
        for (id, expected) in first.iter().enumerate() {
            let DecodedAddrState::Funded(index) =
                stored.get_once(ty, TypeIndex::from(id)).unwrap().decode()
            else {
                panic!("expected restored funded address");
            };
            assert_eq!(
                stored
                    .funded
                    .collect_one(index)
                    .unwrap()
                    .to_bytes()
                    .as_slice(),
                expected
            );
        }
    }
}
