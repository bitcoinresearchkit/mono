use std::collections::hash_map::Entry;

use brk_types::{Height, OutputType, Sats, TxIndex, TypeIndex};
use rustc_hash::FxHashMap;

use crate::{addr::AddrTypeToTypeIndexMap, block::TxIndexes, state::Transacted};

/// Result of processing inputs for a block.
pub struct InputsResult {
    /// Map from UTXO creation height -> aggregated sent supply.
    pub height_to_sent: FxHashMap<Height, Transacted>,
    /// Address spends in input order within each creation height.
    pub sent_data: FxHashMap<Height, Vec<(OutputType, TypeIndex, Sats)>>,
    /// Transaction indexes per address for tx_count tracking.
    pub tx_index_vecs: AddrTypeToTypeIndexMap<TxIndexes>,
}

/// Process inputs (spent UTXOs) for a block.
///
/// Accumulate pre-collected input metadata by creation height and track
/// address-specific data for address cohort processing.
pub fn process_inputs(
    txin_index_to_tx_index: &[TxIndex],
    txin_index_to_value: &[Sats],
    txin_index_to_output_type: &[OutputType],
    txin_index_to_type_index: &[TypeIndex],
    txin_index_to_prev_height: &[Height],
) -> InputsResult {
    let input_count = txin_index_to_value.len();
    debug_assert_eq!(txin_index_to_tx_index.len(), input_count);
    debug_assert_eq!(txin_index_to_output_type.len(), input_count);
    debug_assert_eq!(txin_index_to_type_index.len(), input_count);
    debug_assert_eq!(txin_index_to_prev_height.len(), input_count);

    let txin_index_to_tx_index = &txin_index_to_tx_index[..input_count];
    let txin_index_to_output_type = &txin_index_to_output_type[..input_count];
    let txin_index_to_type_index = &txin_index_to_type_index[..input_count];
    let txin_index_to_prev_height = &txin_index_to_prev_height[..input_count];

    // Estimate: unique heights bounded by block depth, addresses spread across ~8 types
    let estimated_unique_heights = (input_count / 4).max(16);
    let estimated_per_type = (input_count / 8).max(8);
    let mut height_to_sent = FxHashMap::<Height, Transacted>::with_capacity_and_hasher(
        estimated_unique_heights,
        Default::default(),
    );
    let mut sent_data =
        FxHashMap::<Height, Vec<(OutputType, TypeIndex, Sats)>>::with_capacity_and_hasher(
            estimated_unique_heights,
            Default::default(),
        );
    let mut tx_index_vecs = AddrTypeToTypeIndexMap::<TxIndexes>::with_capacity(estimated_per_type);

    for local_idx in 0..input_count {
        let prev_height = txin_index_to_prev_height[local_idx];
        let value = txin_index_to_value[local_idx];
        let output_type = txin_index_to_output_type[local_idx];

        height_to_sent
            .entry(prev_height)
            .or_default()
            .iterate(value, output_type);

        if output_type.is_not_addr() {
            continue;
        }

        let type_index = txin_index_to_type_index[local_idx];
        sent_data
            .entry(prev_height)
            .or_default()
            .push((output_type, type_index, value));
        match tx_index_vecs
            .get_mut(output_type)
            .unwrap()
            .entry(type_index)
        {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(txin_index_to_tx_index[local_idx]);
            }
            Entry::Vacant(entry) => {
                entry.insert(TxIndexes::new(txin_index_to_tx_index[local_idx]));
            }
        }
    }

    InputsResult {
        height_to_sent,
        sent_data,
        tx_index_vecs,
    }
}
