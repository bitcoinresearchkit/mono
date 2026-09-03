//! Projected next block: full template and incremental diff.

use std::sync::Arc;

use brk_types::{
    BlockTemplate, BlockTemplateDiff, BlockTemplateDiffEntry, MempoolBlock, NextBlockHash,
    Transaction, Txid,
};
use rustc_hash::{FxHashMap, FxHashSet};
use tracing::warn;

use crate::{Mempool, ResolvedBlockTemplateDiff, Snapshot, state::State};

/// Identity of every source that can change the full block-template JSON.
///
/// Holding the snapshot `Arc` also prevents pointer reuse while a cached
/// representation still refers to it.
#[derive(Clone)]
pub struct BlockTemplateSource {
    snapshot: Arc<Snapshot>,
    content_revision: u64,
}

impl PartialEq for BlockTemplateSource {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.snapshot, &other.snapshot)
            && self.content_revision == other.content_revision
    }
}

impl Eq for BlockTemplateSource {}

impl BlockTemplateSource {
    fn new(snapshot: Arc<Snapshot>, content_revision: u64) -> Self {
        Self {
            snapshot,
            content_revision,
        }
    }
}

impl Mempool {
    pub fn next_block_hash(&self) -> NextBlockHash {
        self.snapshot().next_block_hash
    }

    /// Cheap cache key for the currently observable full template.
    #[must_use]
    pub fn block_template_source(&self) -> BlockTemplateSource {
        let snapshot = self.snapshot();
        let content_revision = self.read().txs.content_revision();
        BlockTemplateSource::new(snapshot, content_revision)
    }

    /// Full projected next block: Core's `getblocktemplate` selection
    /// (block 0) with aggregate stats and full tx bodies in GBT order.
    #[must_use]
    pub fn block_template(&self) -> BlockTemplate {
        self.block_template_with_source().0
    }

    /// Full template and the exact source identity used to assemble it.
    #[must_use]
    pub fn block_template_with_source(&self) -> (BlockTemplate, BlockTemplateSource) {
        let snap = self.snapshot();
        let state = self.read();
        let transactions = Self::collect_txs(&state, snap.block0_txids());
        let source = BlockTemplateSource::new(Arc::clone(&snap), state.txs.content_revision());
        let template = BlockTemplate {
            hash: snap.next_block_hash,
            stats: snap
                .block_stats
                .first()
                .map(MempoolBlock::from)
                .unwrap_or_default(),
            transactions,
        };
        (template, source)
    }

    /// Delta of the projected next block since `since`. `None` when
    /// `since` has aged out of the rebuilder's history (server should
    /// 404 -> client falls back to `block_template`).
    ///
    /// `order` walks the new template in template order. Each entry is
    /// either a `Retained` index into the prior template (which the
    /// client cached when it obtained `since`) or a `New` inline body.
    /// `removed` is the convenience list of txids that left.
    #[must_use]
    pub fn block_template_diff(&self, since: NextBlockHash) -> Option<BlockTemplateDiff> {
        let resolved = self.resolve_block_template_diff(since)?;
        Some(self.block_template_diff_resolved(resolved).0)
    }

    /// Validate and capture the historical side of one diff request.
    #[must_use]
    pub fn resolve_block_template_diff(
        &self,
        since: NextBlockHash,
    ) -> Option<ResolvedBlockTemplateDiff> {
        let past = self.rebuilder().historical_block0(since)?;
        let source = self.block_template_source();
        Some(ResolvedBlockTemplateDiff::new(since, past, source))
    }

    /// Build a diff from already-resolved history and return its actual source.
    #[must_use]
    pub fn block_template_diff_resolved(
        &self,
        resolved: ResolvedBlockTemplateDiff,
    ) -> (BlockTemplateDiff, BlockTemplateSource) {
        let (since, past) = resolved.into_parts();
        let prior_index: FxHashMap<Txid, u32> = past
            .iter()
            .enumerate()
            .map(|(idx, txid)| (*txid, idx as u32))
            .collect();
        let snap = self.snapshot();
        let state = self.read();
        let mut order = Vec::with_capacity(snap.blocks.first().map_or(0, Vec::len));
        let mut current: FxHashSet<Txid> = FxHashSet::default();
        for txid in snap.block0_txids() {
            current.insert(txid);
            match prior_index.get(&txid) {
                Some(&idx) => order.push(BlockTemplateDiffEntry::Retained(idx)),
                None => match Self::lookup_body(&state, &txid) {
                    Some(tx) => order.push(BlockTemplateDiffEntry::New(tx)),
                    None => warn!(?txid, "block_template_diff: snapshot tx body missing"),
                },
            }
        }
        let source = BlockTemplateSource::new(Arc::clone(&snap), state.txs.content_revision());
        drop(state);
        let removed = past
            .iter()
            .filter(|txid| !current.contains(txid))
            .copied()
            .collect();
        (
            BlockTemplateDiff {
                hash: snap.next_block_hash,
                since,
                order,
                removed,
            },
            source,
        )
    }

    fn collect_txs(state: &State, txids: impl IntoIterator<Item = Txid>) -> Vec<Transaction> {
        txids
            .into_iter()
            .filter_map(|txid| {
                let body = Self::lookup_body(state, &txid);
                if body.is_none() {
                    warn!(?txid, "block_template: snapshot tx body missing");
                }
                body
            })
            .collect()
    }

    /// Body for a txid in a published snapshot. Graveyard fallback
    /// covers the eviction race: an Applier may have buried the tx
    /// after the snapshot was built. Burial retention (1h) >> snapshot
    /// cycle (~1s), so the invariant holds in practice. A `None` here
    /// is a soft anomaly the caller logs and drops.
    fn lookup_body(state: &State, txid: &Txid) -> Option<Transaction> {
        state
            .txs
            .get(txid)
            .or_else(|| state.graveyard.get(txid).map(|t| &t.tx))
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{BlockTemplateDiffEntry, FeeRate, Sats, TxOut, TxidPrefix, Vin};

    use super::*;
    use crate::{
        state::TxEntry,
        test_support::{fake_entry_info, fake_tx, p2wpkh_script},
    };

    fn insert_tx(mempool: &Mempool, seed: u8, fee: u64, vsize: u64) -> Txid {
        let tx = fake_tx(seed, &[None], &[(p2wpkh_script(seed + 1), 1_234)]);
        let txid = tx.txid;
        let info = fake_entry_info(txid, fee, vsize);
        let entry = TxEntry::new(&info, vsize, false);
        let mut state = mempool.test_state_lock().write();
        state.txs.insert(tx, entry);
        txid
    }

    #[test]
    fn block_template_hash_matches_next_block_hash() {
        let mempool = Mempool::for_test();
        let txid = insert_tx(&mempool, 0xA0, 1_234, 100);
        mempool.test_tick(&[txid], FeeRate::new(1.0));

        let template = mempool.block_template();
        assert_eq!(template.hash, mempool.next_block_hash());
        assert_eq!(template.transactions.len(), 1);
        assert_eq!(template.transactions[0].txid, txid);
    }

    #[test]
    fn block_template_source_tracks_snapshot_and_body_changes() {
        let mempool = Mempool::for_test();
        let txid = insert_tx(&mempool, 0xA7, 1_234, 100);
        mempool.test_tick(&[txid], FeeRate::new(1.0));
        let initial = mempool.block_template_source();

        let prefix = TxidPrefix::from(&txid);
        let prevout = TxOut::from((p2wpkh_script(0xA8), Sats::from(2_000u64)));
        mempool
            .test_state_lock()
            .write()
            .txs
            .apply_fills(&prefix, vec![(Vin::from(0usize), prevout)]);
        let after_body_change = mempool.block_template_source();
        assert!(initial != after_body_change);

        mempool.test_tick(&[txid], FeeRate::new(2.0));
        let after_snapshot_change = mempool.block_template_source();
        assert!(after_body_change != after_snapshot_change);
    }

    #[test]
    fn block_template_diff_round_trip_reconstructs_t1_from_t0() {
        // T0: pool has two txs, both in gbt -> block 0.
        let mempool = Mempool::for_test();
        let txid_a = insert_tx(&mempool, 0xA1, 1_111, 100);
        let txid_b = insert_tx(&mempool, 0xA2, 2_222, 100);
        mempool.test_tick(&[txid_a, txid_b], FeeRate::new(1.0));
        let t0 = mempool.block_template();

        // T1: add a third tx, advance gbt. block_template_diff(t0.hash) must
        // be reconstructible into the new block 0 ordering by combining the
        // retained prior-indexed bodies from T0 with the New bodies inline.
        let txid_c = insert_tx(&mempool, 0xA3, 3_333, 100);
        mempool.test_tick(&[txid_a, txid_b, txid_c], FeeRate::new(1.0));
        let t1 = mempool.block_template();

        let diff = mempool
            .block_template_diff(t0.hash)
            .expect("t0 is still in history");
        assert_eq!(diff.since, t0.hash);
        assert_eq!(diff.hash, t1.hash);

        let mut reconstructed = Vec::with_capacity(diff.order.len());
        for entry in &diff.order {
            match entry {
                BlockTemplateDiffEntry::Retained(idx) => {
                    reconstructed.push(t0.transactions[*idx as usize].clone());
                }
                BlockTemplateDiffEntry::New(tx) => reconstructed.push(tx.clone()),
            }
        }
        let expected: Vec<_> = t1.transactions.iter().map(|tx| tx.txid).collect();
        let got: Vec<_> = reconstructed.iter().map(|tx| tx.txid).collect();
        assert_eq!(got, expected, "diff round-trips back into T1 ordering");
        assert!(diff.removed.is_empty());
    }

    #[test]
    fn block_template_diff_removed_lists_evicted_txs() {
        let mempool = Mempool::for_test();
        let txid_a = insert_tx(&mempool, 0xA4, 1_111, 100);
        let txid_b = insert_tx(&mempool, 0xA5, 2_222, 100);
        mempool.test_tick(&[txid_a, txid_b], FeeRate::new(1.0));
        let t0 = mempool.block_template();

        // T1: txid_a no longer in gbt.
        mempool.test_tick(&[txid_b], FeeRate::new(1.0));
        let diff = mempool.block_template_diff(t0.hash).unwrap();
        assert_eq!(diff.removed, vec![txid_a]);
    }

    #[test]
    fn block_template_diff_unknown_since_returns_none() {
        let mempool = Mempool::for_test();
        mempool.test_tick(&[], FeeRate::new(1.0));
        let bogus = NextBlockHash::new(0xDEAD_BEEF);
        assert!(mempool.resolve_block_template_diff(bogus).is_none());
        assert!(mempool.block_template_diff(bogus).is_none());
    }

    #[test]
    fn resolved_block_template_diff_owns_its_single_history_lookup() {
        let mempool = Mempool::for_test();
        let first = insert_tx(&mempool, 0xB0, 1_000, 100);
        let mut txids = vec![first];
        mempool.test_tick(&txids, FeeRate::new(1.0));
        let since = mempool.next_block_hash();
        let resolved = mempool
            .resolve_block_template_diff(since)
            .expect("initial template in history");

        for seed in 0xB1..=0xBB {
            txids.push(insert_tx(&mempool, seed, 1_000, 100));
            mempool.test_tick(&txids, FeeRate::new(1.0));
        }
        assert!(mempool.resolve_block_template_diff(since).is_none());

        let (diff, _) = mempool.block_template_diff_resolved(resolved);
        assert_eq!(diff.since, since);
        assert_eq!(diff.order.len(), txids.len());
    }

    #[test]
    fn block_template_empty_pool_has_no_transactions() {
        let mempool = Mempool::for_test();
        mempool.test_tick(&[], FeeRate::new(2.0));
        let template = mempool.block_template();
        assert!(template.transactions.is_empty());
    }
}
