//! Builds the writer's graph; only completed candidates become public.

use std::{mem, sync::Arc};

use brk_types::{FeeRate, Transaction, Txid, Vout};
use rapidhash::RapidHashSet;

use crate::{State, key::TxKey};

use super::{SnapTx, Snapshot, TxIndex, builder::ChunkCache, fees::FeeView, partition};

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
    /// `unloaded`: listed transactions not loaded yet; the blocks after block 0 withhold what
    /// can't follow them or `state.template_only` (block 0's bodies the listing lacks).
    /// `membership_ready`: every listed transaction loaded, or the missing ones stuck. With it and
    /// no template-only bodies the view is complete: it stands behind its fee tiers.
    /// `gbt_txids`: Core's template, block 0; without one, the packer's first block is.
    pub fn tick(
        &mut self,
        state: &State,
        gbt_txids: Option<&[Txid]>,
        min_fee: FeeRate,
        fee_deltas: &[(Txid, i64)],
        unloaded: &RapidHashSet<Txid>,
        membership_ready: bool,
    ) {
        let complete = state.template_only.is_empty() && membership_ready;
        let snapshot = &self.snapshot;
        if snapshot.ensure_projection().is_ok()
            && snapshot.complete == complete
            && snapshot.min_fee == min_fee
            && *snapshot.fee_deltas == *fee_deltas
            && snapshot
                .template_only
                .iter()
                .copied()
                .eq(state.template_only.iter().map(|record| record.digest))
            && snapshot.unloaded == *unloaded
            && snapshot.content_revision == state.txs.content_revision()
            && match gbt_txids {
                Some(gbt_txids) => {
                    !snapshot.block0_packed && snapshot.block0_txids().eq(gbt_txids.iter().copied())
                }
                None => snapshot.block0_packed,
            }
        {
            return;
        }

        let snapshot = Self::build_snapshot(
            state,
            gbt_txids,
            min_fee,
            fee_deltas,
            unloaded,
            (!complete).then_some(&self.snapshot.fees),
            &mut self.chunks,
        );
        self.snapshot = Arc::new(snapshot);
    }

    #[allow(clippy::too_many_arguments)]
    fn build_snapshot(
        state: &State,
        gbt_txids: Option<&[Txid]>,
        min_fee: FeeRate,
        fee_deltas: &[(Txid, i64)],
        unloaded: &RapidHashSet<Txid>,
        held_fees: Option<&FeeView>,
        cache: &mut ChunkCache,
    ) -> Snapshot {
        let (txs, key_to_idx, chunks) =
            Snapshot::build_txs(&state.txs, &state.template_only, fee_deltas, cache);
        let pooled = state.txs.len();
        let record_at = |i: usize| {
            if i < pooled {
                state.txs.record_at(i)
            } else {
                &state.template_only[i - pooled]
            }
        };
        // Core's selection, each member at its chunk's feerate.
        let template_block0: Vec<(TxIndex, FeeRate)> = gbt_txids
            .into_iter()
            .flatten()
            .filter_map(|txid| {
                key_to_idx
                    .get(&TxKey::from(txid))
                    .copied()
                    .filter(|index| txs[index.as_usize()].txid == *txid)
            })
            .map(|index| (index, txs[index.as_usize()].chunk_rate))
            .collect();
        let mut excluded = vec![false; txs.len()];
        for (index, _) in &template_block0 {
            excluded[index.as_usize()] = true;
        }
        if !state.template_only.is_empty() || !unloaded.is_empty() {
            let bodies: Vec<&Transaction> = (0..txs.len()).map(|i| record_at(i).tx.as_ref()).collect();
            withhold(&bodies, &txs, pooled, unloaded, &mut excluded);
        }
        // Without a template, the packer's first block stands in for Core's.
        let packed = gbt_txids.is_none();
        let mut projection = partition::partition(
            &txs,
            &chunks,
            &excluded,
            NUM_BLOCKS - usize::from(!packed),
        );
        let block0 = if packed {
            match projection.blocks.first() {
                // A lone block holding more than a block stays the catch-all, after an empty one.
                Some(_) if projection.blocks.len() > 1 || !projection.overflow => {
                    projection.blocks.remove(0)
                }
                _ => Vec::new(),
            }
        } else {
            template_block0
        };
        let record_of = |index: &TxIndex| {
            let record = record_at(index.as_usize());
            debug_assert_eq!(record.entry.txid, txs[index.as_usize()].txid);
            (record.tx.clone(), record.digest)
        };

        // Each projected block's bodies, but a last one holding more than a block.
        let served = projection.blocks.len() - usize::from(projection.overflow);
        let mut blocks = Vec::with_capacity(1 + served);
        for block in std::iter::once(&block0).chain(&projection.blocks[..served]) {
            blocks.push(block.iter().map(|(index, _)| record_of(index)).unzip());
        }

        let missing = gbt_txids.is_some_and(|gbt_txids| block0.len() != gbt_txids.len());
        let mut snapshot = Snapshot::build(
            txs,
            block0,
            packed,
            &projection.blocks,
            key_to_idx,
            chunks,
            min_fee,
            held_fees,
        );
        snapshot.set_served(blocks, state.txs.content_revision(), missing);
        snapshot.template_only = state
            .template_only
            .iter()
            .map(|record| record.digest)
            .collect();
        snapshot.unloaded = unloaded.clone();
        snapshot.fee_deltas = fee_deltas.into();
        snapshot
    }

    pub fn snapshot(&self) -> &Arc<Snapshot> {
        &self.snapshot
    }
}

/// Marks, beside block 0's members already `excluded`, what the blocks after it can't hold: a
/// transaction spending an outpoint a template-only one (from index `template_only` on) spends
/// (the template can be older than the listing, a replacement listed meanwhile), one spending a
/// listed transaction not loaded yet, and their descendants. `bodies` and `txs` are indexed
/// alike.
pub(crate) fn withhold(
    bodies: &[&Transaction],
    txs: &[SnapTx],
    template_only: usize,
    unloaded: &RapidHashSet<Txid>,
    excluded: &mut [bool],
) {
    let spent: RapidHashSet<(Txid, Vout)> = bodies[template_only..]
        .iter()
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
