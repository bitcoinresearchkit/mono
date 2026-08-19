use brk_types::{Sats, TxIndex, TypeIndex};
use smallvec::SmallVec;

use crate::{
    addr::{AddrTypeToTypeIndexMap, AddrTypeToVec},
    compute::TxOutData,
    state::Transacted,
};

/// Result of processing outputs for a block.
pub struct OutputsResult {
    /// Aggregated supply transacted in this block.
    pub transacted: Transacted,
    /// Per-address-type received data: (type_index, value) for each address.
    pub received_data: AddrTypeToVec<(TypeIndex, Sats)>,
    /// Transaction indexes per address for tx_count tracking.
    pub tx_index_vecs: AddrTypeToTypeIndexMap<SmallVec<[TxIndex; 4]>>,
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

    let estimated_per_type = (output_count / 8).max(8);
    let mut transacted = Transacted::default();
    let mut received_data = AddrTypeToVec::with_capacity(estimated_per_type);
    let mut tx_index_vecs =
        AddrTypeToTypeIndexMap::<SmallVec<[TxIndex; 4]>>::with_capacity(estimated_per_type);

    for (local_idx, txout_data) in txout_data_vec.iter().enumerate() {
        let value = txout_data.value;
        let output_type = txout_data.output_type;
        transacted.iterate(value, output_type);

        if output_type.is_not_addr() {
            continue;
        }

        let type_index = txout_data.type_index;
        received_data
            .get_mut(output_type)
            .unwrap()
            .push((type_index, value));
        tx_index_vecs
            .get_mut(output_type)
            .unwrap()
            .entry(type_index)
            .or_default()
            .push(txout_index_to_tx_index[local_idx]);
    }

    OutputsResult {
        transacted,
        received_data,
        tx_index_vecs,
    }
}
