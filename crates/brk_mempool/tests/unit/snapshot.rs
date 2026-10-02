use brk_types::{Sats, VSize, Weight};
use smallvec::SmallVec;

use super::*;

impl Snapshot {
    /// Test-only: stitch a snapshot from `(prefix, chunk_rate)` pairs
    /// without running the full builder.
    pub(crate) fn for_test_with_chunk_rates(entries: &[(TxidPrefix, FeeRate, Txid)]) -> Self {
        let mut prefix_to_idx = PrefixIndex::default();
        let mut txs = Vec::with_capacity(entries.len());
        for (i, (prefix, rate, txid)) in entries.iter().enumerate() {
            prefix_to_idx.insert(*prefix, TxIndex::from(i));
            txs.push(SnapTx {
                txid: *txid,
                fee: Sats::ZERO,
                vsize: VSize::from(0u64),
                weight: Weight::from(0u64),
                size: 0,
                chunk_rate: *rate,
                parents: SmallVec::new(),
                children: SmallVec::new(),
            });
        }
        Self {
            txs,
            blocks: vec![],
            block_stats: vec![],
            fees: RecommendedFees::default(),
            min_fee: FeeRate::default(),
            next_block_hash: NextBlockHash::ZERO,
            prefix_to_idx,
            template_transactions: Arc::from([]),
            content_revision: 0,
            template_missing: false,
        }
    }
}
