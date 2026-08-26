//! Turn `Fetched` raws into a typed diff for the Applier. Pure CPU,
//! holds a read guard on `State` for the cycle. New entries are
//! classified into two buckets:
//!
//! - **revivable** - in the graveyard, resurrected from the tombstone.
//! - **fresh** - decoded from `new_raws`, prevouts resolved against
//!   the live mempool only. Confirmed-parent prevouts land as
//!   `prevout: None` and are filled post-apply by the resolver passed
//!   to `Mempool::tick_with`.
//!
//! Existing entries are not re-classified - they keep their first-sight
//! state on the live store. Removals are inferred by cross-referencing
//! inputs against the full `live_txids` set from the cycle's pull.

use brk_types::{MempoolEntryInfo, Transaction, Txid, TxidPrefix, Vout};
use parking_lot::RwLock;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    State,
    stores::{TxGraveyard, TxStore},
};

mod tx_addition;
mod tx_removal;
mod txs_pulled;

pub use tx_addition::TxAddition;
pub use tx_removal::TxRemoval;
pub use txs_pulled::TxsPulled;

pub struct Preparer;

impl Preparer {
    pub fn prepare(
        live_txids: &[Txid],
        new_entries: Vec<MempoolEntryInfo>,
        new_txs: FxHashMap<Txid, bitcoin::Transaction>,
        lock: &RwLock<State>,
    ) -> TxsPulled {
        let state = lock.read();

        let live: FxHashSet<TxidPrefix> = live_txids.iter().map(TxidPrefix::from).collect();
        let added = Self::classify_additions(new_entries, new_txs, &state.txs, &state.graveyard);
        let removed = Self::classify_removals(&live, &added, &state.txs);

        TxsPulled {
            live_len: live.len(),
            added,
            removed,
        }
    }

    fn classify_additions(
        new_entries: Vec<MempoolEntryInfo>,
        mut new_txs: FxHashMap<Txid, bitcoin::Transaction>,
        known: &TxStore,
        graveyard: &TxGraveyard,
    ) -> Vec<TxAddition> {
        new_entries
            .iter()
            .filter_map(|info| Self::classify_addition(info, known, graveyard, &mut new_txs))
            .collect()
    }

    fn classify_addition(
        info: &MempoolEntryInfo,
        known: &TxStore,
        graveyard: &TxGraveyard,
        new_txs: &mut FxHashMap<Txid, bitcoin::Transaction>,
    ) -> Option<TxAddition> {
        if known.contains(&info.txid) {
            return None;
        }
        if let Some(tomb) = graveyard.get(&info.txid) {
            return Some(TxAddition::revived(info, tomb));
        }
        let tx = new_txs.remove(&info.txid)?;
        Some(TxAddition::fresh(info, tx, known))
    }

    /// One `(prefix, reason)` per known tx that's gone from the live set,
    /// in `known` iteration order.
    ///
    /// Cost is `O(R * avg_inputs)` where R is the removed-tx count and
    /// `avg_inputs` is small for non-pathological txs. Worst case is a
    /// `mempoolminfee` jump dropping ~10k txs in one cycle - still well
    /// under the cycle budget.
    fn classify_removals(
        live: &FxHashSet<TxidPrefix>,
        added: &[TxAddition],
        known: &TxStore,
    ) -> Vec<(TxidPrefix, TxRemoval)> {
        let spent_by = Self::build_spent_by(added);
        known
            .records()
            .filter_map(|(prefix, record)| {
                if live.contains(prefix) {
                    return None;
                }
                Some((*prefix, Self::removal_reason(&record.tx, &spent_by)))
            })
            .collect()
    }

    fn removal_reason(tx: &Transaction, spent_by: &FxHashMap<(Txid, Vout), Txid>) -> TxRemoval {
        tx.input
            .iter()
            .find_map(|i| spent_by.get(&(i.txid, i.vout)).copied())
            .map_or(TxRemoval::Vanished, |by| TxRemoval::Replaced { by })
    }

    /// Only `Fresh` additions carry tx input data. Revived txs were
    /// already in-pool, so they can't be new spenders of anything.
    fn build_spent_by(added: &[TxAddition]) -> FxHashMap<(Txid, Vout), Txid> {
        let mut spent_by: FxHashMap<(Txid, Vout), Txid> = FxHashMap::default();
        for addition in added {
            if let TxAddition::Fresh { tx, .. } = addition {
                for txin in &tx.input {
                    spent_by.insert((txin.txid, txin.vout), tx.txid);
                }
            }
        }
        spent_by
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::hashes::Hash;
    use brk_types::{FeeRate, Sats, TxOut, VSize};

    use super::*;
    use crate::{
        AddedKind, TxRemoval,
        state::TxEntry,
        test_support::{fake_bitcoin_tx, fake_entry_info, fake_tx, fake_txid, p2wpkh_script},
    };

    fn empty_state() -> RwLock<State> {
        RwLock::new(State::default())
    }

    fn seed_known(state: &RwLock<State>, txid: Txid) {
        let tx = fake_tx(0xA0, &[None], &[(p2wpkh_script(50), 5_000)]);
        let mut altered = tx;
        altered.txid = txid;
        for input in altered.input.iter_mut() {
            input.prevout = Some(TxOut::from((p2wpkh_script(51), Sats::from(1_000u64))));
        }
        let info = fake_entry_info(txid, 1_000, 100);
        let entry = TxEntry::new(&info, 100, false);
        state.write().txs.insert(altered, entry);
    }

    fn seed_graveyard(state: &RwLock<State>, txid: Txid) {
        let tx = fake_tx(0xB0, &[None], &[(p2wpkh_script(60), 5_000)]);
        let mut altered = tx;
        altered.txid = txid;
        let info = fake_entry_info(txid, 500, 100);
        let entry = TxEntry::new(&info, 100, false);
        let rate = FeeRate::from((Sats::from(500u64), VSize::from(100u64)));
        state
            .write()
            .graveyard
            .bury(altered, entry, rate, TxRemoval::Vanished);
    }

    #[test]
    fn classify_addition_skips_already_known() {
        let state = empty_state();
        let known_txid = fake_txid(0x10);
        seed_known(&state, known_txid);

        let info = fake_entry_info(known_txid, 100, 100);
        let mut new_txs: FxHashMap<Txid, bitcoin::Transaction> = FxHashMap::default();
        new_txs.insert(
            known_txid,
            fake_bitcoin_tx(0x11, &[(p2wpkh_script(7), 1_234)]),
        );

        let pulled = Preparer::prepare(&[known_txid], vec![info], new_txs, &state);
        assert!(pulled.added.is_empty(), "known tx must be filtered out");
        assert!(pulled.removed.is_empty(), "still live, nothing removed");
    }

    #[test]
    fn classify_addition_emits_revived_for_graveyard_hit() {
        let state = empty_state();
        let txid = fake_txid(0x20);
        seed_graveyard(&state, txid);

        let info = fake_entry_info(txid, 100, 100);
        let pulled = Preparer::prepare(&[txid], vec![info], FxHashMap::default(), &state);

        assert_eq!(pulled.added.len(), 1);
        assert!(matches!(pulled.added[0].kind(), AddedKind::Revived));
    }

    #[test]
    fn classify_addition_emits_fresh_with_raw_payload() {
        let state = empty_state();
        let txid = fake_txid(0x30);
        // Make the bitcoin tx hash to `txid`: we instead key `new_txs`
        // by the synthetic txid, since classify_addition keys lookup by
        // info.txid, not by tx.compute_txid().
        let info = fake_entry_info(txid, 200, 120);
        let raw = fake_bitcoin_tx(0x31, &[(p2wpkh_script(8), 2_345)]);
        let mut new_txs: FxHashMap<Txid, bitcoin::Transaction> = FxHashMap::default();
        new_txs.insert(txid, raw);

        let pulled = Preparer::prepare(&[txid], vec![info], new_txs, &state);
        assert_eq!(pulled.added.len(), 1);
        assert!(matches!(pulled.added[0].kind(), AddedKind::Fresh));
    }

    #[test]
    fn classify_addition_drops_entry_with_no_raw_and_no_graveyard() {
        let state = empty_state();
        let txid = fake_txid(0x40);
        let info = fake_entry_info(txid, 100, 100);

        let pulled = Preparer::prepare(&[txid], vec![info], FxHashMap::default(), &state);
        assert!(pulled.added.is_empty(), "no payload, no tomb -> filtered");
    }

    #[test]
    fn classify_removal_marks_replaced_when_outpoint_is_spent_by_new_tx() {
        let state = empty_state();
        // Loser: spends (parent, vout=0). We arrange the new fresh tx
        // to spend the same outpoint.
        let parent_txid = fake_txid(0x50);
        let loser_txid = fake_txid(0x51);
        let replacer_txid = fake_txid(0x52);
        {
            let prev = Some(TxOut::from((p2wpkh_script(80), Sats::from(10_000u64))));
            let mut tx = fake_tx(0x51, &[prev], &[(p2wpkh_script(81), 5_000)]);
            tx.txid = loser_txid;
            tx.input[0].txid = parent_txid;
            tx.input[0].vout = Vout::ZERO;
            let info = fake_entry_info(loser_txid, 100, 100);
            let entry = TxEntry::new(&info, 100, false);
            state.write().txs.insert(tx, entry);
        }

        let info = fake_entry_info(replacer_txid, 200, 120);
        let mut new_txs: FxHashMap<Txid, bitcoin::Transaction> = FxHashMap::default();
        let mut raw = fake_bitcoin_tx(0x52, &[(p2wpkh_script(82), 4_321)]);
        raw.input[0].previous_output = bitcoin::OutPoint {
            txid: bitcoin::Txid::from_byte_array({
                let mut b = [0u8; 32];
                b[0] = 0x50;
                b
            }),
            vout: 0,
        };
        new_txs.insert(replacer_txid, raw);

        let pulled = Preparer::prepare(&[replacer_txid], vec![info], new_txs, &state);
        assert_eq!(pulled.removed.len(), 1);
        let (_, reason) = pulled.removed[0];
        match reason {
            TxRemoval::Replaced { by } => assert_eq!(by, replacer_txid),
            TxRemoval::Vanished => panic!("expected Replaced, got Vanished"),
        }
    }

    #[test]
    fn classify_removal_marks_vanished_when_no_new_tx_spends_outpoint() {
        let state = empty_state();
        let gone_txid = fake_txid(0x60);
        {
            let prev = Some(TxOut::from((p2wpkh_script(90), Sats::from(10_000u64))));
            let mut tx = fake_tx(0x60, &[prev], &[(p2wpkh_script(91), 6_000)]);
            tx.txid = gone_txid;
            tx.input[0].txid = fake_txid(0xAA);
            let info = fake_entry_info(gone_txid, 100, 100);
            let entry = TxEntry::new(&info, 100, false);
            state.write().txs.insert(tx, entry);
        }

        // No live txids in this cycle, no replacers staged.
        let pulled = Preparer::prepare(&[], vec![], FxHashMap::default(), &state);
        assert_eq!(pulled.removed.len(), 1);
        assert!(matches!(pulled.removed[0].1, TxRemoval::Vanished));
    }

    #[test]
    fn fresh_resolves_prevout_from_same_cycle_mempool_parent() {
        // Same-cycle ordering: parent inserted first, then child whose
        // input points at parent.vout=0. We exercise the path by
        // putting the parent into the live store via a Fresh add, then
        // a second Preparer call where the child's `info` references
        // the parent's outpoint.
        let state = empty_state();
        let parent_txid = fake_txid(0x70);
        let child_txid = fake_txid(0x71);

        // Stage parent directly in the live store so resolve_prevout
        // sees it when the child is decoded.
        {
            let mut parent = fake_tx(0x70, &[], &[(p2wpkh_script(100), 7_777)]);
            parent.txid = parent_txid;
            parent.input.clear();
            let info = fake_entry_info(parent_txid, 100, 80);
            let entry = TxEntry::new(&info, 80, false);
            state.write().txs.insert(parent, entry);
        }

        let info = fake_entry_info(child_txid, 200, 120);
        let mut raw = fake_bitcoin_tx(0x70, &[(p2wpkh_script(101), 6_000)]);
        raw.input[0].previous_output = bitcoin::OutPoint {
            txid: bitcoin::Txid::from_byte_array({
                let mut b = [0u8; 32];
                b[0] = 0x70;
                b
            }),
            vout: 0,
        };
        let mut new_txs: FxHashMap<Txid, bitcoin::Transaction> = FxHashMap::default();
        new_txs.insert(child_txid, raw);

        let pulled = Preparer::prepare(&[parent_txid, child_txid], vec![info], new_txs, &state);
        let TxAddition::Fresh { tx, .. } = &pulled.added[0] else {
            panic!("expected Fresh classification");
        };
        let prevout = tx.input[0]
            .prevout
            .as_ref()
            .expect("parent in same-cycle pool must resolve");
        assert_eq!(prevout.value, Sats::from(7_777u64));
        // No removal: parent + child both in live set.
        assert!(pulled.removed.is_empty());
    }
}
