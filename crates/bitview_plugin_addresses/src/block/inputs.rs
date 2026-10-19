use std::collections::hash_map::Entry;

use bitview_primitives::TypeIndex;
use brk_types::{Cents, Height, OutputType, Sats, TxIndex};

use crate::{
    addr::shard_of,
    block::{ShardBlocks, TxIndexes},
};

/// Price address spends at their creation heights (`prices` by height) and retain unique
/// transaction uses.
pub fn process_inputs(
    blocks: &mut ShardBlocks,
    txs: impl Iterator<Item = TxIndex>,
    values: &[Sats],
    output_types: &[OutputType],
    type_indexes: &[TypeIndex],
    creation_heights: &[Height],
    prices: &[Cents],
) {
    let input_count = values.len();
    debug_assert_eq!(output_types.len(), input_count);
    debug_assert_eq!(type_indexes.len(), input_count);
    debug_assert_eq!(creation_heights.len(), input_count);

    // Re-sliced to one length, so a single bounds check covers the four reads.
    let output_types = &output_types[..input_count];
    let type_indexes = &type_indexes[..input_count];
    let creation_heights = &creation_heights[..input_count];

    let mut processed = 0;
    for (local_idx, tx_index) in txs.enumerate() {
        processed = local_idx + 1;
        let creation_height = creation_heights[local_idx];
        let value = values[local_idx];
        let output_type = output_types[local_idx];

        if output_type.is_not_addr() {
            continue;
        }

        let type_index = type_indexes[local_idx];
        let block = blocks[shard_of(type_index)].get_mut_unwrap(output_type);
        block
            .spends
            .push((type_index, value, prices[usize::from(creation_height)]));
        match block.sent_txs.entry(type_index) {
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
}
