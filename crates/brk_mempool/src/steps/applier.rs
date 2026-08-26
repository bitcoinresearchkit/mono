use brk_types::{Transaction, TxidPrefix};
use parking_lot::RwLock;

use crate::{
    Snapshot, TxRemoval,
    cycle::{AddrTransitions, CycleDiff, TxAdded, TxRemoved},
    state::{State, TxEntry},
    steps::preparer::{TxAddition, TxsPulled},
};

/// Applies a prepared diff to in-memory mempool state under one write
/// guard. Body proceeds: bury removed → publish added → evict. Events
/// are pushed into the caller-supplied [`CycleDiff`] accumulator.
pub struct Applier;

impl Applier {
    /// `prev_snapshot` supplies the previous cycle's snapshot. Burial
    /// reads each tomb's `chunk_rate` from it (always-fresh,
    /// package-aware via local linearization). The fallback to
    /// `entry.fee_rate()` is unreachable in steady state - every burial
    /// target was alive at the previous tick, so the snapshot has it.
    pub fn apply(
        lock: &RwLock<State>,
        prev_snapshot: &Snapshot,
        pulled: TxsPulled,
        diff: &mut CycleDiff,
    ) {
        let TxsPulled {
            live_len,
            added,
            removed,
        } = pulled;
        let mut state = lock.write();
        let additional = live_len.saturating_sub(state.txs.len());
        state.txs.reserve(additional);
        Self::bury_removals(
            &mut state,
            prev_snapshot,
            &mut diff.addrs,
            &mut diff.removed,
            removed,
        );
        Self::publish_additions(&mut state, &mut diff.addrs, &mut diff.added, added);
        state.graveyard.evict_old();
    }

    fn bury_removals(
        state: &mut State,
        snapshot: &Snapshot,
        transitions: &mut AddrTransitions,
        events: &mut Vec<TxRemoved>,
        removed: Vec<(TxidPrefix, TxRemoval)>,
    ) {
        events.reserve(removed.len());
        for (prefix, reason) in removed {
            if let Some(ev) = Self::bury_one(state, snapshot, transitions, &prefix, reason) {
                events.push(ev);
            }
        }
    }

    fn bury_one(
        state: &mut State,
        prev_snapshot: &Snapshot,
        transitions: &mut AddrTransitions,
        prefix: &TxidPrefix,
        reason: TxRemoval,
    ) -> Option<TxRemoved> {
        let record = state.txs.remove_by_prefix(prefix)?;
        let chunk_rate = prev_snapshot
            .chunk_rate_for(prefix)
            .unwrap_or_else(|| record.entry.fee_rate());
        let txid = record.entry.txid;
        state.info.remove(&record.tx, record.entry.fee);
        state.addrs.remove_tx(transitions, &record.tx);
        state.outpoint_spends.remove_spends(&record.tx, *prefix);
        state
            .graveyard
            .bury(record.tx, record.entry, chunk_rate, reason);
        Some(TxRemoved {
            txid,
            reason,
            chunk_rate,
        })
    }

    fn publish_additions(
        state: &mut State,
        transitions: &mut AddrTransitions,
        events: &mut Vec<TxAdded>,
        added: Vec<TxAddition>,
    ) {
        events.reserve(added.len());
        for addition in added {
            let kind = addition.kind();
            if let Some((tx, entry)) = Self::resolve_addition(state, addition) {
                events.push(TxAdded {
                    txid: entry.txid,
                    fee: entry.fee,
                    vsize: entry.vsize,
                    fee_rate: entry.fee_rate(),
                    first_seen: entry.first_seen,
                    kind,
                });
                Self::publish_one(state, transitions, tx, entry);
            }
        }
    }

    fn resolve_addition(state: &mut State, addition: TxAddition) -> Option<(Transaction, TxEntry)> {
        match addition {
            TxAddition::Fresh { tx, entry } => Some((tx, entry)),
            TxAddition::Revived { entry } => {
                let tomb = state.graveyard.exhume(&entry.txid)?;
                Some((tomb.tx, entry))
            }
        }
    }

    fn publish_one(
        state: &mut State,
        transitions: &mut AddrTransitions,
        tx: Transaction,
        entry: TxEntry,
    ) {
        let prefix = entry.txid_prefix();
        state.info.add(&tx, entry.fee);
        state.addrs.add_tx(transitions, &tx);
        state.outpoint_spends.insert_spends(&tx, prefix);
        state.txs.insert(tx, entry);
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{FeeRate, Sats, TxOut, Txid, VSize};

    use super::*;
    use crate::{
        AddedKind,
        cycle::CycleDiff,
        steps::preparer::{TxAddition, TxsPulled},
        test_support::{fake_entry_info, fake_tx, p2wpkh_script},
    };

    fn fresh_addition(seed: u8, fee: u64, vsize: u64) -> (TxAddition, Txid) {
        let prev = Some(TxOut::from((p2wpkh_script(seed), Sats::from(2_500u64))));
        let tx = fake_tx(seed, &[prev], &[(p2wpkh_script(seed + 1), 1_234)]);
        let txid = tx.txid;
        let info = fake_entry_info(txid, fee, vsize);
        let entry = TxEntry::new(&info, vsize, false);
        (TxAddition::Fresh { tx, entry }, txid)
    }

    fn fresh_pulled(addition: TxAddition) -> TxsPulled {
        TxsPulled {
            live_len: 1,
            added: vec![addition],
            removed: vec![],
        }
    }

    #[test]
    fn publish_one_inserts_into_all_stores() {
        let lock = RwLock::new(State::default());
        let snapshot = Snapshot::default();
        let mut diff = CycleDiff::default();
        let (addition, txid) = fresh_addition(0xC0, 200, 100);

        Applier::apply(&lock, &snapshot, fresh_pulled(addition), &mut diff);

        let state = lock.read();
        assert!(state.txs.contains(&txid));
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].txid, txid);
    }

    #[test]
    fn revived_path_exhumes_body_from_graveyard() {
        let lock = RwLock::new(State::default());
        let snapshot = Snapshot::default();
        let (addition, txid) = fresh_addition(0xC1, 300, 100);
        let TxAddition::Fresh { tx, entry } = addition else {
            unreachable!();
        };
        // Pre-load the graveyard with this tx, then submit a Revived
        // addition that re-publishes it without a raw body.
        let rate = FeeRate::from((entry.fee, entry.vsize));
        lock.write()
            .graveyard
            .bury(tx, entry.clone(), rate, TxRemoval::Vanished);

        let mut diff = CycleDiff::default();
        Applier::apply(
            &lock,
            &snapshot,
            fresh_pulled(TxAddition::Revived { entry }),
            &mut diff,
        );

        let state = lock.read();
        assert!(state.txs.contains(&txid), "revived tx republished");
        assert!(state.graveyard.get(&txid).is_none(), "tomb consumed");
        assert_eq!(diff.added.len(), 1);
        assert!(matches!(diff.added[0].kind, AddedKind::Revived));
    }

    #[test]
    fn revived_with_empty_graveyard_is_dropped() {
        let lock = RwLock::new(State::default());
        let snapshot = Snapshot::default();
        let info = fake_entry_info(Txid::COINBASE, 100, 100);
        let entry = TxEntry::new(&info, 100, false);

        let mut diff = CycleDiff::default();
        Applier::apply(
            &lock,
            &snapshot,
            fresh_pulled(TxAddition::Revived { entry }),
            &mut diff,
        );

        let state = lock.read();
        assert!(!state.txs.contains(&Txid::COINBASE));
        assert!(diff.added.is_empty(), "no body, no event");
    }

    #[test]
    fn bury_preserves_chunk_rate_from_snapshot() {
        let lock = RwLock::new(State::default());
        let (addition, txid) = fresh_addition(0xC2, 100, 100);

        // Publish first to plant the tx, with a fee-rate that differs
        // from the snapshot's stub rate so we can tell them apart.
        Applier::apply(
            &lock,
            &Snapshot::default(),
            fresh_pulled(addition),
            &mut CycleDiff::default(),
        );
        let isolated_rate = FeeRate::from((Sats::from(100u64), VSize::from(100u64)));

        let cpfp_rate = FeeRate::from((Sats::from(500u64), VSize::from(100u64)));
        let prefix = TxidPrefix::from(&txid);
        let snapshot = Snapshot::for_test_with_chunk_rates(&[(prefix, cpfp_rate, txid)]);

        let mut diff = CycleDiff::default();
        Applier::apply(
            &lock,
            &snapshot,
            TxsPulled {
                live_len: 0,
                added: vec![],
                removed: vec![(prefix, TxRemoval::Vanished)],
            },
            &mut diff,
        );

        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.removed[0].chunk_rate, cpfp_rate);
        assert_ne!(diff.removed[0].chunk_rate, isolated_rate);
        let state = lock.read();
        assert_eq!(state.graveyard.get(&txid).unwrap().chunk_rate, cpfp_rate);
    }

    #[test]
    fn bury_falls_back_to_isolated_rate_when_snapshot_misses() {
        let lock = RwLock::new(State::default());
        let (addition, txid) = fresh_addition(0xC3, 700, 100);
        Applier::apply(
            &lock,
            &Snapshot::default(),
            fresh_pulled(addition),
            &mut CycleDiff::default(),
        );

        let isolated_rate = FeeRate::from((Sats::from(700u64), VSize::from(100u64)));
        let prefix = TxidPrefix::from(&txid);

        let mut diff = CycleDiff::default();
        Applier::apply(
            &lock,
            &Snapshot::default(),
            TxsPulled {
                live_len: 0,
                added: vec![],
                removed: vec![(prefix, TxRemoval::Vanished)],
            },
            &mut diff,
        );

        assert_eq!(diff.removed[0].chunk_rate, isolated_rate);
    }
}
