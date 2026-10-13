use std::collections::hash_map::Entry;

use bitview_cohort::ByAddrType;
use bitview_primitives::TypeIndex;
use brk_types::{Cents, Height, OutputType, Sats, TxIndex};

use crate::{
    addr::{AddrTypeToTypeIndexMap, SHARDS, shard_of},
    block::TxIndexes,
};

/// Result of processing inputs for a block.
pub struct InputsResult {
    /// Address spends in input order with their creation prices, by shard and type.
    pub spends: [ByAddrType<Vec<(TypeIndex, Sats, Cents)>>; SHARDS],
    /// Transaction indexes per address for tx_count tracking, by shard.
    pub tx_index_vecs: [AddrTypeToTypeIndexMap<TxIndexes>; SHARDS],
}

/// Process inputs (spent UTXOs) for a block.
///
/// Price address spends at their creation heights (`prices` by height) and retain unique
/// transaction uses.
pub fn process_inputs(
    txs: impl Iterator<Item = TxIndex>,
    txin_index_to_value: &[Sats],
    txin_index_to_output_type: &[OutputType],
    txin_index_to_type_index: &[TypeIndex],
    txin_index_to_prev_height: &[Height],
    prices: &[Cents],
) -> InputsResult {
    let input_count = txin_index_to_value.len();
    debug_assert_eq!(txin_index_to_output_type.len(), input_count);
    debug_assert_eq!(txin_index_to_type_index.len(), input_count);
    debug_assert_eq!(txin_index_to_prev_height.len(), input_count);

    let txin_index_to_output_type = &txin_index_to_output_type[..input_count];
    let txin_index_to_type_index = &txin_index_to_type_index[..input_count];
    let txin_index_to_prev_height = &txin_index_to_prev_height[..input_count];

    // Addresses are spread across eight types.
    let estimated = (input_count / 8 / SHARDS).max(8);
    let mut spends: [ByAddrType<Vec<(TypeIndex, Sats, Cents)>>; SHARDS] = Default::default();
    let mut tx_index_vecs =
        std::array::from_fn(|_| AddrTypeToTypeIndexMap::<TxIndexes>::with_capacity(estimated));

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
        let shard = shard_of(type_index);
        spends[shard].get_mut_unwrap(output_type).push((
            type_index,
            value,
            prices[usize::from(prev_height)],
        ));
        match tx_index_vecs[shard]
            .get_mut_unwrap(output_type)
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

    debug_assert_eq!(
        processed, input_count,
        "incomplete input transaction ranges"
    );
    InputsResult {
        spends,
        tx_index_vecs,
    }
}
