//! Builds the writer's graph; only completed candidates become public.

use std::{mem, sync::Arc};

use brk_types::{FeeRate, Transaction, Txid, Vout};
use rapidhash::RapidHashSet;

use crate::{State, key::TxKey};

use super::{SnapTx, Snapshot, TxIndex, builder::ChunkCache, partition};

/// Projected blocks, mempool.space's count: block 0 and seven packed after it, the last holding
/// whatever is left.
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
    /// `unloaded`: listed transactions not loaded yet. With it and `state.template_only` empty
    /// the view is complete; otherwise blocks after block 0 withhold what can't follow it.
    pub fn tick(
        &mut self,
        state: &State,
        gbt_txids: &[Txid],
        min_fee: FeeRate,
        fee_deltas: &[(Txid, i64)],
        unloaded: &RapidHashSet<Txid>,
    ) {
        let snapshot = &self.snapshot;
        if snapshot.ensure_projection().is_ok()
            && snapshot.min_fee == min_fee
            && *snapshot.fee_deltas == *fee_deltas
            && snapshot.template_only == state.template_only
            && snapshot.unloaded == *unloaded
            && snapshot.content_revision == state.txs.content_revision()
            && snapshot.block0_txids().eq(gbt_txids.iter().copied())
        {
            return;
        }

        self.snapshot = Arc::new(Self::build_snapshot(
            state,
            gbt_txids,
            min_fee,
            fee_deltas,
            unloaded,
            &mut self.chunks,
        ));
    }

    fn build_snapshot(
        state: &State,
        gbt_txids: &[Txid],
        min_fee: FeeRate,
        fee_deltas: &[(Txid, i64)],
        unloaded: &RapidHashSet<Txid>,
        cache: &mut ChunkCache,
    ) -> Snapshot {
        let complete = state.template_only.is_empty() && unloaded.is_empty();
        let (txs, key_to_idx, chunks) = Snapshot::build_txs(&state.txs, fee_deltas, cache);
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
        if !complete {
            let bodies: Vec<&Transaction> = (0..txs.len())
                .map(|i| state.txs.record_at(i).tx.as_ref())
                .collect();
            withhold(&bodies, &txs, &state.template_only, unloaded, &mut excluded);
        }
        let projection = partition::partition(&txs, &chunks, &excluded, NUM_BLOCKS - 1);

        // Each projected block's bodies, but a last one holding more than a block.
        let served = projection.blocks.len() - usize::from(projection.overflow);
        let mut blocks = Vec::with_capacity(1 + served);
        blocks.push((bodies, digests));
        for block in &projection.blocks[..served] {
            blocks.push(
                block
                    .iter()
                    .map(|&(m, _)| {
                        let record = state.txs.record_at(m.as_usize());
                        debug_assert_eq!(record.entry.txid, txs[m.as_usize()].txid);
                        (record.tx.clone(), record.digest)
                    })
                    .unzip(),
            );
        }

        let missing = blocks[0].0.len() != gbt_txids.len();
        let mut snapshot =
            Snapshot::build(txs, block0, &projection.blocks, key_to_idx, chunks, min_fee);
        snapshot.set_served(blocks, state.txs.content_revision(), missing);
        snapshot.template_only = state.template_only.clone();
        snapshot.unloaded = unloaded.clone();
        snapshot.fee_deltas = fee_deltas.into();
        snapshot
    }

    pub fn snapshot(&self) -> &Arc<Snapshot> {
        &self.snapshot
    }
}

/// Marks, beside block 0's members already `excluded`, what the blocks after it can't hold: a
/// transaction spending an outpoint a template-only one spends (the template can be older than
/// the listing, a replacement listed meanwhile), one spending a listed transaction not loaded
/// yet, and their descendants. `bodies` and `txs` are indexed alike.
pub(crate) fn withhold(
    bodies: &[&Transaction],
    txs: &[SnapTx],
    template_only: &RapidHashSet<Txid>,
    unloaded: &RapidHashSet<Txid>,
    excluded: &mut [bool],
) {
    let spent: RapidHashSet<(Txid, Vout)> = bodies
        .iter()
        .filter(|tx| template_only.contains(&tx.txid))
        .flat_map(|tx| tx.input.iter().map(|input| (input.txid, input.vout)))
        .collect();
    let mut stack: Vec<TxIndex> = bodies
        .iter()
        .enumerate()
        .filter(|&(i, tx)| {
            !excluded[i]
                && tx.input.iter().any(|input| {
                    unloaded.contains(&input.txid) || spent.contains(&(input.txid, input.vout))
                })
        })
        .map(|(i, _)| TxIndex::from(i))
        .collect();
    while let Some(index) = stack.pop() {
        if !mem::replace(&mut excluded[index.as_usize()], true) {
            stack.extend(txs[index.as_usize()].children.iter().copied());
        }
    }
}
