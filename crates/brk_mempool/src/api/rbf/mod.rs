//! RBF tree extraction. Returns owned trees so the caller can enrich
//! with indexer data (`mined`, effective fee rate) after the lock
//! drops: enriching under the lock re-enters `Mempool` and would
//! recursively acquire the same read lock.

use brk_types::{FeeRate, Sats, Transaction, Txid};
use rustc_hash::FxHashSet;

use crate::{
    Mempool,
    snapshot::Snapshot,
    state::TxEntry,
    stores::{TxGraveyard, TxStore},
};

mod for_tx;
mod node;

pub use for_tx::RbfForTx;
pub use node::RbfNode;

impl Mempool {
    /// Walk forward through `Replaced { by }` to the terminal replacer
    /// and return its full predecessor tree, plus the requested tx's
    /// direct predecessors. Single read-lock window.
    #[must_use]
    pub fn rbf_for_tx(&self, txid: &Txid) -> RbfForTx {
        let mut rbf = {
            let state = self.read();
            let root_txid = state.graveyard.replacement_root_of(*txid);
            let replaces = state
                .graveyard
                .predecessors_of(txid)
                .map(|(predecessor, _)| *predecessor)
                .collect();
            let root = Self::build_rbf_node(&root_txid, &state.txs, &state.graveyard);
            RbfForTx { root, replaces }
        };
        if let Some(root) = rbf.root.as_mut() {
            Self::apply_snapshot_rates(root, &self.snapshot());
        }
        rbf
    }

    /// Recent terminal-replacer trees, most-recent first, deduplicated
    /// by root, capped at `limit`. `full_rbf_only` drops trees with no
    /// non-signaling predecessor.
    #[must_use]
    pub fn recent_rbf_trees(&self, full_rbf_only: bool, limit: usize) -> Vec<RbfNode> {
        let mut trees: Vec<RbfNode> = {
            let state = self.read();
            let mut seen: FxHashSet<Txid> = FxHashSet::default();
            state
                .graveyard
                .replaced_iter_recent_first()
                .filter_map(|(_, by)| {
                    let root = state.graveyard.replacement_root_of(*by);
                    seen.insert(root).then_some(root)
                })
                .filter_map(|root| Self::build_rbf_node(&root, &state.txs, &state.graveyard))
                .filter(|node| !full_rbf_only || node.full_rbf)
                .take(limit)
                .collect()
        };
        if !trees.is_empty() {
            let snapshot = self.snapshot();
            for root in &mut trees {
                Self::apply_snapshot_rates(root, &snapshot);
            }
        }
        trees
    }

    fn build_rbf_node(txid: &Txid, txs: &TxStore, graveyard: &TxGraveyard) -> Option<RbfNode> {
        let (tx, entry, rate, in_mempool) = Self::resolve_rbf_node(txid, txs, graveyard)?;

        let replaces: Vec<RbfNode> = graveyard
            .predecessors_of(txid)
            .filter_map(|(predecessor, _)| Self::build_rbf_node(predecessor, txs, graveyard))
            .collect();

        let full_rbf = replaces.iter().any(|c| !c.rbf || c.full_rbf);
        let value: Sats = tx.output.iter().map(|o| o.value).sum();

        Some(RbfNode {
            txid: *txid,
            fee: entry.fee,
            vsize: entry.vsize,
            value,
            first_seen: entry.first_seen,
            rate,
            in_mempool,
            rbf: entry.rbf,
            full_rbf,
            replaces,
        })
    }

    fn resolve_rbf_node<'a>(
        txid: &Txid,
        txs: &'a TxStore,
        graveyard: &'a TxGraveyard,
    ) -> Option<(&'a Transaction, &'a TxEntry, FeeRate, bool)> {
        if let Some(record) = txs.record(txid) {
            return Some((&record.tx, &record.entry, record.entry.fee_rate(), true));
        }
        graveyard
            .get(txid)
            .map(|tombstone| (&tombstone.tx, &tombstone.entry, tombstone.chunk_rate, false))
    }

    fn apply_snapshot_rates(node: &mut RbfNode, snapshot: &Snapshot) {
        if node.in_mempool
            && let Some(rate) = snapshot.chunk_rate_for(&node.txid)
        {
            node.rate = rate;
        }
        for predecessor in &mut node.replaces {
            Self::apply_snapshot_rates(predecessor, snapshot);
        }
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::hashes::Hash;
    use brk_types::{FeeRate, TxidPrefix};

    use super::*;
    use crate::{
        Mempool, TxRemoval,
        state::TxEntry,
        test_support::{fake_entry_info, fake_tx, fake_txid, p2wpkh_script},
    };

    /// Place a live tx (the replacer) and bury one or more predecessors
    /// pointing at it. `bury_chain` carries `(seed, predecessor_of_next)`
    /// pairs in oldest-first order. Each links forward to the next entry
    /// or to `live_seed` when last.
    fn build_rbf_world(live_seed: u8, predecessors: &[u8]) -> (Mempool, Txid, Vec<Txid>) {
        let mempool = Mempool::for_test();
        let live_tx = fake_tx(live_seed, &[None], &[(p2wpkh_script(live_seed + 1), 1_234)]);
        let live_txid = live_tx.txid;
        let live_entry = TxEntry::new(&fake_entry_info(live_txid, 5_000, 100), 100, true);

        let mut pred_txids = Vec::with_capacity(predecessors.len());
        let mut state = mempool.test_state_lock().write();
        for (i, seed) in predecessors.iter().enumerate() {
            let tx = fake_tx(*seed, &[None], &[(p2wpkh_script(seed + 1), 1_234)]);
            let txid = tx.txid;
            // Each predecessor signals BIP-125 (rbf=true) so full_rbf stays clear.
            let entry = TxEntry::new(&fake_entry_info(txid, 1_000, 100), 100, true);
            let by = predecessors
                .get(i + 1)
                .map(|next_seed| fake_txid(*next_seed))
                .unwrap_or(live_txid);
            let rate = FeeRate::from((entry.fee, entry.vsize));
            state
                .graveyard
                .bury(tx, entry, rate, TxRemoval::Replaced { by });
            pred_txids.push(txid);
        }
        state.txs.insert(live_tx, live_entry);
        drop(state);
        (mempool, live_txid, pred_txids)
    }

    #[test]
    fn rbf_for_tx_single_replacement_returns_root_and_replaces() {
        // pred -> live. rbf_for_tx(pred) walks forward to live and lists
        // pred under its `replaces` tree.
        let (mempool, live, preds) = build_rbf_world(0xC0, &[0xC1]);
        let pred = preds[0];

        let rbf = mempool.rbf_for_tx(&pred);
        let root = rbf.root.expect("terminal replacer reachable");
        assert_eq!(root.txid, live);
        assert!(root.in_mempool);
        assert!(!root.replaces[0].in_mempool);
        let replaced_txids: Vec<Txid> = root.replaces.iter().map(|n| n.txid).collect();
        assert_eq!(replaced_txids, vec![pred]);
        // Convenience list: direct predecessors of the requested tx.
        assert!(
            rbf.replaces.is_empty(),
            "pred has no predecessors of its own"
        );
    }

    #[test]
    fn rbf_for_tx_chain_walks_to_terminal_root() {
        // A -> B -> C(live). rbf_for_tx(A) walks A -> B -> C, root is C.
        // root.replaces is B, B.replaces is A.
        let (mempool, live, preds) = build_rbf_world(0xC2, &[0xC3, 0xC4]);
        let a = preds[0];
        let b = preds[1];

        let rbf = mempool.rbf_for_tx(&a);
        let root = rbf.root.expect("terminal replacer reachable");
        assert_eq!(root.txid, live);
        assert_eq!(root.replaces.len(), 1);
        assert_eq!(root.replaces[0].txid, b);
        assert_eq!(root.replaces[0].replaces.len(), 1);
        assert_eq!(root.replaces[0].replaces[0].txid, a);
    }

    #[test]
    fn rbf_for_tx_unknown_tx_returns_none_root() {
        let mempool = Mempool::for_test();
        let bogus = Txid::COINBASE;
        let rbf = mempool.rbf_for_tx(&bogus);
        assert!(rbf.root.is_none());
        assert!(rbf.replaces.is_empty());
    }

    #[test]
    fn rbf_for_tx_rejects_live_prefix_collision() {
        let (mempool, live, _) = build_rbf_world(0xCA, &[]);
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(live.as_slice());
        bytes[8] ^= 1;
        let collision = Txid::from(bitcoin::Txid::from_byte_array(bytes));
        assert_eq!(TxidPrefix::from(&live), TxidPrefix::from(&collision));

        assert!(mempool.rbf_for_tx(&collision).is_empty());
    }

    #[test]
    fn recent_rbf_trees_dedup_by_root_and_respect_limit() {
        // Chain 0xC6 -> 0xC7 -> live plus a sibling 0xC8 also replaced by
        // live. All paths roll up to the same root, so the recent listing
        // dedups them down to a single tree.
        let (mempool, live, _preds) = build_rbf_world(0xC5, &[0xC6, 0xC7]);
        {
            let mut state = mempool.test_state_lock().write();
            let extra = fake_tx(0xC8, &[None], &[(p2wpkh_script(0xC9), 1_234)]);
            let extra_txid = extra.txid;
            let entry = TxEntry::new(&fake_entry_info(extra_txid, 999, 100), 100, true);
            let rate = FeeRate::from((entry.fee, entry.vsize));
            state
                .graveyard
                .bury(extra, entry, rate, TxRemoval::Replaced { by: live });
        }
        let trees = mempool.recent_rbf_trees(false, 10);
        assert_eq!(trees.len(), 1, "all paths roll up to one root");
        assert_eq!(trees[0].txid, live);

        let capped = mempool.recent_rbf_trees(false, 0);
        assert!(capped.is_empty(), "limit honored");
    }
}
