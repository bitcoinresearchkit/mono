use std::collections::hash_map::Entry;

use brk_types::{Height, OutputType, Sats, TxIndex, TypeIndex};

use crate::{addr::AddrTypeToTypeIndexMap, block::TxIndexes, block::address_spends::AddressSpends};

/// Result of processing inputs for a block.
pub struct InputsResult {
    /// Address spends in input order within each creation height.
    pub sent_data: AddressSpends,
    /// Transaction indexes per address for tx_count tracking.
    pub tx_index_vecs: AddrTypeToTypeIndexMap<TxIndexes>,
}

/// Process inputs (spent UTXOs) for a block.
///
/// Group address spends by creation height and retain unique transaction uses.
pub fn process_inputs(
    txs: impl Iterator<Item = TxIndex>,
    txin_index_to_value: &[Sats],
    txin_index_to_output_type: &[OutputType],
    txin_index_to_type_index: &[TypeIndex],
    txin_index_to_prev_height: &[Height],
) -> InputsResult {
    let input_count = txin_index_to_value.len();
    debug_assert_eq!(txin_index_to_output_type.len(), input_count);
    debug_assert_eq!(txin_index_to_type_index.len(), input_count);
    debug_assert_eq!(txin_index_to_prev_height.len(), input_count);

    let txin_index_to_output_type = &txin_index_to_output_type[..input_count];
    let txin_index_to_type_index = &txin_index_to_type_index[..input_count];
    let txin_index_to_prev_height = &txin_index_to_prev_height[..input_count];

    // Addresses are spread across eight types.
    let estimated_per_type = (input_count / 8).max(8);
    let mut sent_data = AddressSpends::new(input_count);
    let mut tx_index_vecs = AddrTypeToTypeIndexMap::<TxIndexes>::with_capacity(estimated_per_type);

    let mut processed = 0;
    for (local_idx, tx_index) in txs.enumerate() {
        processed = local_idx + 1;
        let prev_height = txin_index_to_prev_height[local_idx];
        let value = txin_index_to_value[local_idx];
        let output_type = txin_index_to_output_type[local_idx];

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
                entry.get_mut().push(tx_index);
            }
            Entry::Vacant(entry) => {
                entry.insert(TxIndexes::new(tx_index));
            }
        }
    }

    assert_eq!(
        processed, input_count,
        "incomplete input transaction ranges"
    );
    InputsResult {
        sent_data,
        tx_index_vecs,
    }
}
