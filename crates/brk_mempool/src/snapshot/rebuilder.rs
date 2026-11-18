//! Builds the writer's graph; only completed candidates become public.

use std::sync::Arc;

use brk_types::{FeeRate, Txid};

use crate::State;

use super::{Snapshot, TxIndex, builder::ChunkCache, partition};

/// Projected blocks, mempool.space's count: block 0 and seven packed after it, the last holding
/// whatever is left.
const NUM_BLOCKS: usize = 8;

#[derive(Default)]
pub struct Rebuilder {
    snapshot: Arc<Snapshot>,
    chunks: ChunkCache,
    /// The same for the residual components across virtual block boundaries.
    leftovers: ChunkCache,
}

impl Rebuilder {
    pub fn restore(&mut self, snapshot: Arc<Snapshot>) {
        self.snapshot = snapshot;
    }

    /// Reuse the private graph only when all of its inputs are unchanged.
    /// Publication and diff history are committed together by the writer.
    pub fn tick(&mut self, state: &State, min_fee: FeeRate, modified_fees: &[(Txid, i64)]) {
        let snapshot = &self.snapshot;
        if snapshot.ensure_projection().is_ok()
            && snapshot.min_fee == min_fee
            && *snapshot.modified_fees == *modified_fees
            && snapshot.content_revision == state.txs.content_revision()
        {
            return;
        }

        let snapshot = Self::build_snapshot(
            state,
            min_fee,
            modified_fees,
            &mut self.chunks,
            &mut self.leftovers,
            &self.snapshot,
        );
        self.snapshot = Arc::new(snapshot);
    }

    fn build_snapshot(
        state: &State,
        min_fee: FeeRate,
        modified_fees: &[(Txid, i64)],
        cache: &mut ChunkCache,
        leftovers: &mut ChunkCache,
        previous: &Snapshot,
    ) -> Snapshot {
        let (txs, key_to_idx, chunks) = Snapshot::build_txs(&state.txs, modified_fees, cache);
        let mut projection = partition::partition(&txs, &chunks, NUM_BLOCKS, leftovers);
        // An empty pool still has a block 0, and a lone block holding more than a block stays the
        // catch-all after an empty one.
        if projection.blocks.len() == usize::from(projection.overflow) {
            projection.blocks.insert(0, Vec::new());
        }
        let record_of = |index: &TxIndex| {
            let record = state.txs.record_at(index.as_usize());
            debug_assert_eq!(record.entry.txid, txs[index.as_usize()].txid);
            (record.tx.clone(), record.digest)
        };

        // Each projected block's bodies, but a last one holding more than a block.
        let served = projection.blocks.len() - usize::from(projection.overflow);
        let blocks = projection.blocks[..served]
            .iter()
            .map(|block| block.iter().map(|(index, _)| record_of(index)).unzip())
            .collect();

        let mut snapshot = Snapshot::build(txs, &projection.blocks, key_to_idx, chunks, min_fee);
        snapshot.set_served(
            blocks,
            state.txs.content_revision(),
            previous.served_blocks(),
        );
        snapshot.block0 = projection.blocks[0]
            .iter()
            .map(|(index, _)| state.txs.record_at(index.as_usize()).clone())
            .collect();
        snapshot.modified_fees = modified_fees.into();
        snapshot
    }

    pub fn snapshot(&self) -> &Arc<Snapshot> {
        &self.snapshot
    }
}
