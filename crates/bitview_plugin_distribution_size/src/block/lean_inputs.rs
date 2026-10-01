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
    txs: impl Iterator<Item = TxIndex>,
    txin_index_to_value: &[Sats],
    txin_index_to_output_type: &[OutputType],
    txin_index_to_type_index: &[TypeIndex],
    txin_index_to_prev_height: &[Height],
    current_price: Cents,
    prices: &[Cents],
) -> LeanInputsResult {
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

    let mut detailed = DetailedSpends::default();
    let mut processed = 0;
    for (local_idx, tx_index) in txs.enumerate() {
        processed = local_idx + 1;
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
    LeanInputsResult {
        detailed,
        sent_data,
        tx_index_vecs,
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use super::*;

    #[test]
    fn input_ranges_keep_transaction_counts_and_origin_prices_per_typed_address() {
        let values = [3u64, 0, 5, 6, 8].map(Sats::new);
        let types = [
            OutputType::P2PKH,
            OutputType::P2PKH,
            OutputType::P2PKH,
            OutputType::P2SH,
            OutputType::OpReturn,
        ];
        let indexes = [TypeIndex::new(7); 5];
        let origins = [0u32, 0, 1, 1, 1].map(Height::new);
        let prices = [Cents::new(100), Cents::new(200)];
        let txs = [10u32, 10, 11, 11, 12].map(TxIndex::new);
        let result = process_inputs_lean(
            txs.into_iter(),
            &values,
            &types,
            &indexes,
            &origins,
            Cents::new(300),
            &prices,
        );
        let received = result.tx_index_vecs.get_unwrap(OutputType::P2PKH);
        assert_eq!(received[&indexes[0]].len(), 2);
        assert_eq!(
            result.tx_index_vecs.get_unwrap(OutputType::P2SH)[&indexes[0]].len(),
            1
        );
        let sent = result.sent_data.into_typed(&prices);
        // Group ordering is deliberately unspecified; per-origin input order is retained.
        let mut actual = sent.get_unwrap(OutputType::P2PKH).clone();
        actual.sort_by_key(|(_, _, price)| price.inner());
        assert_eq!(
            actual,
            [
                (indexes[0], Sats::new(3), prices[0]),
                (indexes[0], Sats::ZERO, prices[0]),
                (indexes[0], Sats::new(5), prices[1])
            ]
        );
        assert_eq!(
            sent.get_unwrap(OutputType::P2SH),
            &[(indexes[0], Sats::new(6), prices[1])]
        );
        let empty = process_inputs_lean(iter::empty(), &[], &[], &[], &[], Cents::ZERO, &[]);
        assert!(
            empty
                .tx_index_vecs
                .iter()
                .all(|(_, entries)| entries.is_empty())
        );
    }
}
