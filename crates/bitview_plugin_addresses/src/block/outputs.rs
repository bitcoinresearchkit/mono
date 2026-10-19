use std::collections::hash_map::Entry;

use bitview_primitives::TypeIndex;
use brk_types::{OutputType, Sats, TxIndex};

use crate::{
    addr::shard_of,
    block::{Received, ShardBlocks},
};

/// Group received value, output count and unique transaction uses per typed address, by shard.
pub fn process_outputs(
    blocks: &mut ShardBlocks,
    mut txs: impl Iterator<Item = TxIndex>,
    values: &[Sats],
    types: &[OutputType],
    indexes: &[TypeIndex],
) {
    let output_count = values.len();
    debug_assert_eq!(types.len(), output_count);
    debug_assert_eq!(indexes.len(), output_count);

    for ((&value, &output_type), &type_index) in values.iter().zip(types).zip(indexes) {
        let tx_index = txs.next().expect("incomplete output transaction ranges");

        if output_type.is_not_addr() {
            continue;
        }

        match blocks[shard_of(type_index)]
            .get_mut_unwrap(output_type)
            .received
            .entry(type_index)
        {
            Entry::Occupied(mut entry) => {
                entry.get_mut().add(value, tx_index);
            }
            Entry::Vacant(entry) => {
                entry.insert(Received::new(value, tx_index));
            }
        }
    }

    debug_assert!(txs.next().is_none(), "excess output transaction ranges");
}
