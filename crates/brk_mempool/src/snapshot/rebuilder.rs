//! Builds the writer's graph; only completed candidates become public.

use std::sync::Arc;

use brk_types::{FeeRate, Txid};

use crate::{State, key::TxKey};

use super::{Snapshot, TxIndex, builder::ChunkCache, partition};

const NUM_BLOCKS: usize = 8;

#[derive(Default)]
pub struct Rebuilder {
    snapshot: Arc<Snapshot>,
    chunks: ChunkCache,
}

impl Rebuilder {
    pub fn restore(&mut self, snapshot: Arc<Snapshot>) {
        self.snapshot = snapshot;
    }

    /// Reuse the private graph only when all of its inputs are unchanged.
    /// Publication and template history are committed together by the writer.
    pub fn tick(&mut self, state: &State, gbt_txids: &[Txid], min_fee: FeeRate) {
        let snapshot = &self.snapshot;
        if snapshot.ensure_projection().is_ok()
            && snapshot.min_fee == min_fee
            && snapshot.content_revision == state.txs.content_revision()
            && snapshot.block0_txids().eq(gbt_txids.iter().copied())
        {
            return;
        }

        self.snapshot = Arc::new(Self::build_snapshot(state, gbt_txids, min_fee, &mut self.chunks));
    }

    fn build_snapshot(
        state: &State,
        gbt_txids: &[Txid],
        min_fee: FeeRate,
        cache: &mut ChunkCache,
    ) -> Snapshot {
        let (txs, key_to_idx, chunks) = Snapshot::build_txs(&state.txs, cache);
        let (bodies, digests): (Vec<_>, Vec<_>) = gbt_txids
            .iter()
            .filter_map(|txid| state.txs.record(txid))
            .map(|record| (record.tx.clone(), record.digest))
            .unzip();

        let block0: Vec<TxIndex> = gbt_txids
            .iter()
            .filter_map(|txid| {
                key_to_idx
                    .get(&TxKey::from(txid))
                    .copied()
                    .filter(|index| txs[index.as_usize()].txid == *txid)
            })
            .collect();
        let mut excluded = vec![false; txs.len()];
        for index in &block0 {
            excluded[index.as_usize()] = true;
        }
        let rest = partition::partition(&txs, &chunks, &excluded, NUM_BLOCKS - 1);

        let missing = bodies.len() != gbt_txids.len();
        let mut snapshot = Snapshot::build(txs, block0, &rest, key_to_idx, min_fee);
        snapshot.set_template(bodies, &digests, state.txs.content_revision(), missing);
        snapshot
    }

    pub fn snapshot(&self) -> &Arc<Snapshot> {
        &self.snapshot
    }
}
