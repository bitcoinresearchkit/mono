//! Builds the writer's graph; only completed candidates become public.

use std::{mem, sync::Arc};

use brk_types::{FeeRate, Transaction, Txid};
use rapidhash::RapidHashSet;

use crate::State;

use super::{SnapTx, Snapshot, TxIndex, builder::ChunkCache, fees::FeeView, partition};

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
    /// `unloaded`: listed transactions not loaded yet; no block takes what can't follow them.
    /// `membership_ready`: every listed transaction loaded, or the missing ones stuck. With it the
    /// view is complete: it stands behind its fee tiers.
    pub fn tick(
        &mut self,
        state: &State,
        min_fee: FeeRate,
        modified_fees: &[(Txid, i64)],
        unloaded: &RapidHashSet<Txid>,
        membership_ready: bool,
    ) {
        let snapshot = &self.snapshot;
        if snapshot.ensure_projection().is_ok()
            && snapshot.complete == membership_ready
            && snapshot.min_fee == min_fee
            && *snapshot.modified_fees == *modified_fees
            && snapshot.unloaded == *unloaded
            && snapshot.content_revision == state.txs.content_revision()
        {
            return;
        }

        let snapshot = Self::build_snapshot(
            state,
            min_fee,
            modified_fees,
            unloaded,
            (!membership_ready).then_some(&self.snapshot.fees),
            &mut self.chunks,
            &mut self.leftovers,
            &self.snapshot,
        );
        self.snapshot = Arc::new(snapshot);
    }

    #[allow(clippy::too_many_arguments)]
    fn build_snapshot(
        state: &State,
        min_fee: FeeRate,
        modified_fees: &[(Txid, i64)],
        unloaded: &RapidHashSet<Txid>,
        held_fees: Option<&FeeView>,
        cache: &mut ChunkCache,
        leftovers: &mut ChunkCache,
        previous: &Snapshot,
    ) -> Snapshot {
        let (txs, key_to_idx, chunks) = Snapshot::build_txs(&state.txs, modified_fees, cache);
        let mut excluded = vec![false; txs.len()];
        if !unloaded.is_empty() {
            let bodies: Vec<&Transaction> =
                (0..txs.len()).map(|i| state.txs.record_at(i).tx.as_ref()).collect();
            withhold(&bodies, &txs, unloaded, &mut excluded);
        }
        let mut projection = partition::partition(&txs, &chunks, &excluded, NUM_BLOCKS, leftovers);
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

        let mut snapshot = Snapshot::build(
            txs,
            &projection.blocks,
            key_to_idx,
            chunks,
            min_fee,
            held_fees,
        );
        snapshot.set_served(blocks, state.txs.content_revision(), previous.served_blocks());
        snapshot.block0 = projection.blocks[0]
            .iter()
            .map(|(index, _)| state.txs.record_at(index.as_usize()).clone())
            .collect();
        snapshot.unloaded = unloaded.clone();
        snapshot.modified_fees = modified_fees.into();
        snapshot
    }

    pub fn snapshot(&self) -> &Arc<Snapshot> {
        &self.snapshot
    }
}

/// Marks what no block can hold: a transaction spending a listed transaction not loaded yet, and
/// its descendants. `bodies` and `txs` are indexed alike.
fn withhold(
    bodies: &[&Transaction],
    txs: &[SnapTx],
    unloaded: &RapidHashSet<Txid>,
    excluded: &mut [bool],
) {
    let mut stack: Vec<TxIndex> = bodies
        .iter()
        .enumerate()
        .filter(|&(i, tx)| {
            !excluded[i] && tx.input.iter().any(|input| unloaded.contains(&input.txid))
        })
        .map(|(i, _)| TxIndex::from(i))
        .collect();
    while let Some(index) = stack.pop() {
        if !mem::replace(&mut excluded[index.as_usize()], true) {
            stack.extend(txs[index.as_usize()].children.iter().copied());
        }
    }
}
