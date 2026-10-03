//! Builds the writer's graph; only completed candidates become public.

use std::sync::Arc;

use brk_types::{FeeRate, Txid, TxidPrefix};

use crate::State;

use super::{Snapshot, TxIndex, partition};

const NUM_BLOCKS: usize = 8;

#[derive(Default)]
pub struct Rebuilder {
    snapshot: Arc<Snapshot>,
}

impl Rebuilder {
    pub fn restore(&mut self, snapshot: Arc<Snapshot>) {
        self.snapshot = snapshot;
    }

    /// Reuse the private graph only when all of its inputs are unchanged.
    /// Publication and template history are committed together by the writer.
    pub fn tick(&mut self, state: &State, gbt_txids: &[Txid], min_fee: FeeRate) {
        let snapshot = &self.snapshot;
        if !snapshot.blocks.is_empty()
            && snapshot.min_fee == min_fee
            && snapshot.content_revision == state.txs.content_revision()
            && snapshot.block0_txids().eq(gbt_txids.iter().copied())
        {
            return;
        }

        self.snapshot = Arc::new(Self::build_snapshot(state, gbt_txids, min_fee));
    }

    fn build_snapshot(state: &State, gbt_txids: &[Txid], min_fee: FeeRate) -> Snapshot {
        let (txs, prefix_to_idx) = Snapshot::build_txs(&state.txs);
        let bodies: Vec<_> = gbt_txids
            .iter()
            .filter_map(|txid| state.txs.record(txid).map(|record| record.tx.clone()))
            .collect();

        let block0: Vec<TxIndex> = gbt_txids
            .iter()
            .filter_map(|txid| {
                prefix_to_idx
                    .get(&TxidPrefix::from(txid))
                    .copied()
                    .filter(|index| txs[index.as_usize()].txid == *txid)
            })
            .collect();
        let mut excluded = vec![0; txs.len()];
        for index in &block0 {
            excluded[index.as_usize()] = 1;
        }
        let rest = partition::partition(&txs, &excluded, NUM_BLOCKS.saturating_sub(1));

        let mut blocks = Vec::with_capacity(NUM_BLOCKS);
        blocks.push(block0);
        blocks.extend(rest);

        let missing = bodies.len() != gbt_txids.len();
        let mut snapshot = Snapshot::build(txs, blocks, prefix_to_idx, min_fee);
        snapshot.set_template(bodies, state.txs.content_revision(), missing);
        snapshot
    }

    pub fn snapshot(&self) -> Arc<Snapshot> {
        self.snapshot.clone()
    }
}
