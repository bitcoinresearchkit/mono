//! # Locking
//!
//! Two locks live on `Rebuilder`: `history` and `snapshot`. Writes always
//! land on `history` first, then `snapshot`, so any `next_block_hash` a
//! reader sees in the published snapshot is already recorded in
//! `historical_block0`. No read path ever holds both, and no path holds
//! a `State` guard together with either Rebuilder lock - the cycle reads
//! `State` once to build the snapshot, then drops it before touching
//! these locks.

use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use brk_types::{FeeRate, NextBlockHash, Txid, TxidPrefix};
use parking_lot::RwLock;

use crate::State;

use super::{Partitioner, Snapshot, TxIndex};

const NUM_BLOCKS: usize = 8;
const HISTORY: usize = 10;

#[derive(Default)]
pub struct Rebuilder {
    snapshot: RwLock<Arc<Snapshot>>,
    /// Past block-0 txid lists keyed by `next_block_hash`, oldest first.
    /// Ordered so `block_template_diff` can emit `Retained(prior_index)`
    /// entries that line up with the client's cached prior template.
    history: RwLock<VecDeque<(NextBlockHash, Vec<Txid>)>>,
    rebuild_count: AtomicU64,
}

impl Rebuilder {
    /// Reuse the published projection only when all of its inputs are
    /// unchanged. History is updated before the snapshot Arc is swapped,
    /// so a reader can never observe a hash that has not been recorded.
    pub fn tick(
        &self,
        lock: &RwLock<State>,
        gbt_txids: &[Txid],
        min_fee: FeeRate,
        membership_changed: bool,
    ) {
        if self.can_reuse(gbt_txids, min_fee, membership_changed) {
            return;
        }

        let snap = Self::build_snapshot(lock, gbt_txids, min_fee);
        let block0: Vec<Txid> = snap.block0_txids().collect();
        let next_hash = snap.next_block_hash;

        let mut hist = self.history.write();
        hist.retain(|(h, _)| *h != next_hash);
        hist.push_back((next_hash, block0));
        while hist.len() > HISTORY {
            hist.pop_front();
        }
        drop(hist);

        *self.snapshot.write() = Arc::new(snap);

        self.rebuild_count.fetch_add(1, Ordering::Relaxed);
    }

    fn can_reuse(&self, gbt_txids: &[Txid], min_fee: FeeRate, membership_changed: bool) -> bool {
        if membership_changed {
            return false;
        }
        let snapshot = self.snapshot.read();
        !snapshot.blocks.is_empty()
            && snapshot.min_fee == min_fee
            && snapshot.block0_txids().eq(gbt_txids.iter().copied())
    }

    /// Past block-0 ordered txid list for `hash`, or `None` if it has
    /// aged out (or was never seen). Used by `block_template_diff` to
    /// decide 200 vs 404 and to resolve `Retained(prior_index)` entries.
    pub fn historical_block0(&self, hash: NextBlockHash) -> Option<Vec<Txid>> {
        self.history
            .read()
            .iter()
            .find(|(h, _)| *h == hash)
            .map(|(_, block0)| block0.clone())
    }

    pub fn rebuild_count(&self) -> u64 {
        self.rebuild_count.load(Ordering::Relaxed)
    }

    fn build_snapshot(lock: &RwLock<State>, gbt_txids: &[Txid], min_fee: FeeRate) -> Snapshot {
        let (txs, prefix_to_idx) = {
            let state = lock.read();
            Snapshot::build_txs(&state.txs)
        };

        let block0: Vec<TxIndex> = gbt_txids
            .iter()
            .filter_map(|txid| prefix_to_idx.get(&TxidPrefix::from(txid)).copied())
            .collect();
        let mut excluded = vec![0; txs.len()];
        for index in &block0 {
            excluded[index.as_usize()] = 1;
        }
        let rest = Partitioner::partition(&txs, &excluded, NUM_BLOCKS.saturating_sub(1));

        let mut blocks = Vec::with_capacity(NUM_BLOCKS);
        blocks.push(block0);
        blocks.extend(rest);

        Snapshot::build(txs, blocks, prefix_to_idx, min_fee)
    }

    pub fn snapshot(&self) -> Arc<Snapshot> {
        self.snapshot.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{Sats, VSize};

    use super::*;
    use crate::{
        state::TxEntry,
        test_support::{fake_entry_info, fake_tx, p2wpkh_script},
    };

    fn state_with(seeds: &[u8]) -> (RwLock<State>, Vec<Txid>) {
        let state = RwLock::new(State::default());
        let mut txids = Vec::with_capacity(seeds.len());
        for &seed in seeds {
            let tx = fake_tx(seed, &[], &[(p2wpkh_script(seed), 1_000)]);
            let txid = tx.txid;
            let entry = TxEntry::new(&fake_entry_info(txid, 100, 100), 100, false);
            state.write().txs.insert(tx, entry);
            txids.push(txid);
        }
        (state, txids)
    }

    fn min_fee(sats: u64) -> FeeRate {
        FeeRate::from((Sats::from(sats), VSize::from(1_000u64)))
    }

    #[test]
    fn first_tick_always_builds() {
        let rebuilder = Rebuilder::default();
        let state = RwLock::new(State::default());

        rebuilder.tick(&state, &[], min_fee(1), false);

        assert_eq!(rebuilder.rebuild_count(), 1);
    }

    #[test]
    fn identical_inputs_reuse_snapshot() {
        let rebuilder = Rebuilder::default();
        let (state, txids) = state_with(&[1, 2]);
        rebuilder.tick(&state, &txids, min_fee(1), true);

        rebuilder.tick(&state, &txids, min_fee(1), false);

        assert_eq!(rebuilder.rebuild_count(), 1);
    }

    #[test]
    fn reordered_template_rebuilds() {
        let rebuilder = Rebuilder::default();
        let (state, mut txids) = state_with(&[1, 2]);
        rebuilder.tick(&state, &txids, min_fee(1), true);
        txids.reverse();

        rebuilder.tick(&state, &txids, min_fee(1), false);

        assert_eq!(rebuilder.rebuild_count(), 2);
    }

    #[test]
    fn changed_min_fee_rebuilds() {
        let rebuilder = Rebuilder::default();
        let (state, txids) = state_with(&[1]);
        rebuilder.tick(&state, &txids, min_fee(1), true);

        rebuilder.tick(&state, &txids, min_fee(2), false);

        assert_eq!(rebuilder.rebuild_count(), 2);
    }

    #[test]
    fn changed_pool_rebuilds() {
        let rebuilder = Rebuilder::default();
        let (state, txids) = state_with(&[1]);
        rebuilder.tick(&state, &txids, min_fee(1), true);

        rebuilder.tick(&state, &txids, min_fee(1), true);

        assert_eq!(rebuilder.rebuild_count(), 2);
    }
}
