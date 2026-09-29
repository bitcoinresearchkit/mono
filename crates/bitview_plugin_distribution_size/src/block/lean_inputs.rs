use std::collections::hash_map::Entry;

use brk_types::{Cents, Height, OutputType, Sats, TxIndex, TypeIndex};

use crate::{
    addr::AddrTypeToTypeIndexMap, block::TxIndexes, block::address_spends::AddressSpends,
    block::detailed_spends::DetailedSpends,
};

/// Result of processing inputs for a block.
pub struct LeanInputsResult {
    /// Scalar totals for amount and output-type cohorts.
    pub detailed: DetailedSpends,
    /// Address spends in input order within each creation height.
    pub sent_data: AddressSpends,
    /// Transaction indexes per address for tx_count tracking.
    pub tx_index_vecs: AddrTypeToTypeIndexMap<TxIndexes>,
}

/// Process inputs (spent UTXOs) for a block.
///
/// Accumulate pre-collected input metadata into size cohorts and track
/// address-specific data for address cohort processing.
pub fn process_inputs_lean(
    txin_index_to_tx_index: &[TxIndex],
    txin_index_to_value: &[Sats],
    txin_index_to_output_type: &[OutputType],
    txin_index_to_type_index: &[TypeIndex],
    txin_index_to_prev_height: &[Height],
    current_price: Cents,
    prices: &[Cents],
) -> LeanInputsResult {
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
    let mut sent_data = AddressSpends::new(estimated_unique_heights);
    let mut tx_index_vecs = AddrTypeToTypeIndexMap::<TxIndexes>::with_capacity(estimated_per_type);

    let mut detailed = DetailedSpends::default();
    for local_idx in 0..input_count {
        let prev_height = txin_index_to_prev_height[local_idx];
        let value = txin_index_to_value[local_idx];
        let output_type = txin_index_to_output_type[local_idx];

        if !output_type.is_unspendable() {
            detailed.add(
                value,
                output_type,
                prices[usize::from(prev_height)],
                current_price,
            );
        }

        if output_type.is_not_addr() {
            continue;
        }

        let type_index = txin_index_to_type_index[local_idx];
        sent_data.push(prev_height, output_type, type_index, value);
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

    LeanInputsResult {
        detailed,
        sent_data,
        tx_index_vecs,
    }
}
