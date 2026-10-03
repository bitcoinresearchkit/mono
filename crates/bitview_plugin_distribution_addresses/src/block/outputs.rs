use std::collections::hash_map::Entry;

use brk_types::{OutputType, Sats, TxIndex, TypeIndex};

use crate::{addr::AddrTypeToTypeIndexMap, block::Received};

/// Group received value, output count and unique transaction uses per typed address.
pub fn process_outputs(
    mut txs: impl Iterator<Item = TxIndex>,
    values: &[Sats],
    types: &[OutputType],
    indexes: &[TypeIndex],
) -> AddrTypeToTypeIndexMap<Received> {
    let output_count = values.len();
    debug_assert_eq!(types.len(), output_count);
    debug_assert_eq!(indexes.len(), output_count);

    let estimated_per_type = (output_count / 8).max(8);
    let mut received = AddrTypeToTypeIndexMap::<Received>::with_capacity(estimated_per_type);

    for ((&value, &output_type), &type_index) in values.iter().zip(types).zip(indexes) {
        let tx_index = txs.next().expect("incomplete output transaction ranges");

        if output_type.is_not_addr() {
            continue;
        }

        match received.get_mut(output_type).unwrap().entry(type_index) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().add(value, tx_index);
            }
            Entry::Vacant(entry) => {
                entry.insert(Received::new(value, tx_index));
            }
        }
    }

    assert!(txs.next().is_none(), "excess output transaction ranges");
    received
}
