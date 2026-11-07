//! Late replacers: what vanished while the view wasn't complete (the per-cycle fetch cap held
//! listed transactions back), indexed by the outpoints it spent, so a replacer arriving cycles
//! later still links it.

use std::{mem, time::Instant};

use brk_types::{Transaction, TxIn, Txid, Vout};
use rapidhash::{RapidHashMap, RapidHashSet};
use smallvec::SmallVec;

/// A vanished burial waiting for a late replacer: its tombstone's time, and its cycle.
#[derive(Clone, Copy)]
struct Burial {
    txid: Txid,
    at: Instant,
    cycle: u64,
}

/// Holds no tombstone: every question about one (still the vanished burial it indexed?) goes to
/// the graveyard, through `vanished_at`.
///
/// Invariants: `unlinked` holds the burials of cycles whose view wasn't complete, and this
/// cycle's once indexed; `pending` the burials not indexed yet (indexing drains it, an incomplete
/// cycle's end keeps it); `unloaded_since` the listed transactions not loaded yet. A complete
/// cycle's end clears all three.
#[derive(Clone, Default)]
pub(super) struct Linker {
    /// What vanished in cycles whose view wasn't complete, by the outpoints each spent.
    unlinked: RapidHashMap<(Txid, Vout), SmallVec<[Burial; 1]>>,
    /// This cycle's vanished burials (and reclassifications), for `unlinked` unless the cycle's
    /// view is complete.
    pending: Vec<(Txid, Instant)>,
    /// Listed transactions not loaded yet, by the cycle each was first listed in: only one listed
    /// by the cycle something vanished in can be what replaced it.
    unloaded_since: RapidHashMap<Txid, u64>,
    /// The current cycle's number.
    cycle: u64,
}

impl Linker {
    /// `txid` vanished at `at` (buried so, or told so).
    pub fn vanished(&mut self, txid: Txid, at: Instant) {
        self.pending.push((txid, at));
    }

    /// This cycle's vanished, indexed by the outpoints they spent (`inputs`: a burial's, while it
    /// is still that one): before [`Self::losers`], so a same-cycle arrival finds them too.
    pub fn index<'a>(&mut self, inputs: impl Fn(&Txid, Instant) -> Option<&'a [TxIn]>) {
        for (txid, at) in mem::take(&mut self.pending) {
            let Some(inputs) = inputs(&txid, at) else {
                continue;
            };
            for txin in inputs {
                self.unlinked
                    .entry((txin.txid, txin.vout))
                    .or_default()
                    .push(Burial {
                        txid,
                        at,
                        cycle: self.cycle,
                    });
            }
        }
    }

    /// What `arrival` replaced among the vanished: those it conflicts with that vanished no
    /// earlier than it was first listed (one listed only later, spending an outpoint the vanished
    /// one freed, replaced nothing), each once, still the burial indexed (`vanished_at`).
    pub fn losers(
        &self,
        arrival: &Transaction,
        vanished_at: impl Fn(&Txid, Instant) -> bool,
    ) -> SmallVec<[Txid; 1]> {
        let mut losers: SmallVec<[Txid; 1]> = SmallVec::new();
        if self.unlinked.is_empty() {
            return losers;
        }
        // One never left unloaded was first listed this cycle.
        let listed = self
            .unloaded_since
            .get(&arrival.txid)
            .copied()
            .unwrap_or(self.cycle);
        for txin in &arrival.input {
            let Some(burials) = self.unlinked.get(&(txin.txid, txin.vout)) else {
                continue;
            };
            for burial in burials {
                // Several of its inputs can lead to one loser: told once.
                if listed <= burial.cycle
                    && burial.txid != arrival.txid
                    && !losers.contains(&burial.txid)
                    && vanished_at(&burial.txid, burial.at)
                {
                    losers.push(burial.txid);
                }
            }
        }
        losers
    }

    /// The cycle's end, with the listed transactions it couldn't load. None: no replacer of what
    /// vanished is still to arrive, all is let go. Otherwise what's no longer vanished
    /// (`vanished_at`) lets go, and the unloaded keep when they were first listed.
    pub fn end_cycle(
        &mut self,
        unloaded: &RapidHashSet<Txid>,
        vanished_at: impl Fn(&Txid, Instant) -> bool,
    ) {
        if unloaded.is_empty() {
            self.unlinked.clear();
            self.pending.clear();
            self.unloaded_since.clear();
        } else {
            self.unlinked.retain(|_, burials| {
                burials.retain(|burial| vanished_at(&burial.txid, burial.at));
                !burials.is_empty()
            });
            let previous = mem::take(&mut self.unloaded_since);
            self.unloaded_since = unloaded
                .iter()
                .map(|txid| (*txid, previous.get(txid).copied().unwrap_or(self.cycle)))
                .collect();
        }
        self.cycle += 1;
    }
}
