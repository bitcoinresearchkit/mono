use std::collections::hash_map::Entry;

use bitview_primitives::{Count, TypeIndex};
use brk_types::{OutputType, Sats, TxIndex};

use crate::{
    addr::shard_of,
    block::{Received, ShardBlocks},
};

/// Group received value, output count and unique transaction uses per typed address, by shard.
/// Returns how many outputs pay an address an input of the same transaction spends from: `blocks`
/// must already hold the block's spends.
pub fn process_outputs(
    blocks: &mut ShardBlocks,
    mut txs: impl Iterator<Item = TxIndex>,
    values: &[Sats],
    types: &[OutputType],
    indexes: &[TypeIndex],
) -> Count {
    let output_count = values.len();
    debug_assert_eq!(types.len(), output_count);
    debug_assert_eq!(indexes.len(), output_count);

    let mut direct_reuse = 0u64;
    for ((&value, &output_type), &type_index) in values.iter().zip(types).zip(indexes) {
        let tx_index = txs.next().expect("incomplete output transaction ranges");

        if output_type.is_not_addr() {
            continue;
        }

        let block = blocks[shard_of(type_index)].get_mut_unwrap(output_type);
        match block.received.entry(type_index) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().add(value, tx_index);
            }
            Entry::Vacant(entry) => {
                entry.insert(Received::new(value, tx_index));
            }
        }
        direct_reuse += u64::from(
            block
                .sent_txs
                .get(&type_index)
                .is_some_and(|txs| txs.contains(tx_index)),
        );
    }

    debug_assert!(txs.next().is_none(), "excess output transaction ranges");
    Count::from(direct_reuse)
}
