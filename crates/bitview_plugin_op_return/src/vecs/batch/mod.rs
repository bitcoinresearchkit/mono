mod carrier;

use std::ops::Range;

use bitview_plugin_indexer::Indexer;
use bitview_plugin_transactions::FeesVecs;
use brk_types::{
    Bytes, OP_RETURN_KIND_COUNT, OpReturnIndex, OpReturnKind, OpReturnPolicyId, Sats, StoredU32,
    TxIndex, VSize, Weight,
};
use rayon::join;
use vecdb::{AnyVec, ColumnId, ReadableVec, VecIndex};

use super::Vecs;
use crate::{breakdown::BlockMetrics, policy::Policy};
use carrier::Carrier;

pub struct Batch {
    block_offsets: Vec<usize>,
    tx_indexes: Vec<TxIndex>,
    kinds: Vec<OpReturnKind>,
    post_op_return_bytes: Vec<StoredU32>,
    weights: Vec<Weight>,
    fees: Vec<Sats>,
}

impl Batch {
    pub fn collect(indexer: &Indexer, fee_vecs: &FeesVecs, heights: Range<usize>) -> Self {
        let vecs = indexer.vecs();
        let raw = &vecs.op_return;
        let mut first_indexes = raw
            .first_index
            .collect_range_at(heights.start, (heights.end + 1).min(raw.first_index.len()));
        if heights.end == raw.first_index.len() {
            first_indexes.push(OpReturnIndex::from(raw.to_tx_index.len()));
        }

        let source_start = first_indexes.first().unwrap().to_usize();
        let source_end = first_indexes.last().unwrap().to_usize();
        let ((tx_indexes, kinds), post_op_return_bytes) = join(
            || {
                join(
                    || raw.to_tx_index.collect_range_at(source_start, source_end),
                    || raw.kind.collect_range_at(source_start, source_end),
                )
            },
            || {
                raw.post_op_return_bytes
                    .collect_range_at(source_start, source_end)
            },
        );

        let mut carrier_tx_positions = Vec::with_capacity(tx_indexes.len());
        for tx_index in &tx_indexes {
            let tx_position = tx_index.to_usize();
            if carrier_tx_positions.last() != Some(&tx_position) {
                carrier_tx_positions.push(tx_position);
            }
        }
        let (weights, fees) = join(
            || {
                vecs.transactions
                    .weight
                    .read_sorted_at(&carrier_tx_positions)
            },
            || fee_vecs.fee.tx_index.read_sorted_at(&carrier_tx_positions),
        );

        Self {
            block_offsets: first_indexes
                .into_iter()
                .map(|index| index.to_usize() - source_start)
                .collect(),
            tx_indexes,
            kinds,
            post_op_return_bytes,
            weights,
            fees,
        }
    }

    pub fn push_into(self, target: &mut Vecs) {
        let Self {
            block_offsets,
            tx_indexes,
            kinds,
            post_op_return_bytes,
            weights,
            fees,
        } = self;
        debug_assert_eq!(tx_indexes.len(), kinds.len());
        debug_assert_eq!(tx_indexes.len(), post_op_return_bytes.len());
        debug_assert_eq!(weights.len(), fees.len());

        let mut carrier_index = 0;

        for offsets in block_offsets.windows(2) {
            let mut total = BlockMetrics::default();
            let mut by_kind = [BlockMetrics::default(); OP_RETURN_KIND_COUNT];
            let mut policy = Policy::default();
            let mut current_tx = None;
            let mut carrier = Carrier::default();

            for record_index in offsets[0]..offsets[1] {
                let tx_index = tx_indexes[record_index];
                let kind = kinds[record_index];
                let bytes = Bytes::from(u32::from(post_op_return_bytes[record_index]));
                let kind_index = kind.index();

                if current_tx != Some(tx_index) {
                    carrier.finalize_into(&mut total, &mut by_kind, &mut policy);
                    current_tx = Some(tx_index);
                    carrier =
                        Carrier::new(VSize::from(weights[carrier_index]), fees[carrier_index]);
                    carrier_index += 1;
                }

                total.data_bytes += bytes;
                by_kind[kind_index].output_count += 1;
                by_kind[kind_index].data_bytes += bytes;
                carrier.add_output(kind, bytes);
            }

            carrier.finalize_into(&mut total, &mut by_kind, &mut policy);
            target.total.push(total);
            target.by_kind.push(by_kind);
            target
                .policy
                .push(OpReturnPolicyId::from_fn(|id| *policy.get(id)));
        }

        debug_assert_eq!(carrier_index, weights.len());
    }
}
