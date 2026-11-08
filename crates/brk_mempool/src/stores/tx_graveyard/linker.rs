//! Tracks vanished tombstones that may still turn out replaced. Two things can reveal the replacer
//! after the fact: a block read later, and an arrival the per-cycle fetch cap held back. Burials
//! are indexed by the outpoints they spent, so a replacer arriving cycles later still finds them.
//! Only the writer uses this. Publications never copy it.

use std::mem;

use brk_types::{Transaction, TxIn, Txid, Vout};
use rapidhash::{RapidHashMap, RapidHashSet};
use smallvec::SmallVec;

use super::Moment;

/// A vanished burial awaiting a late replacer, with its burial time and cycle.
#[derive(Clone, Copy)]
struct Burial {
    txid: Txid,
    at: Moment,
    cycle: u64,
}

/// Holds only references to burials. Each use asks the graveyard whether the tombstone is still the
/// vanished one buried then.
///
/// - `since_confirmed` holds the burials since the blocks were last all applied.
/// - `unlinked` holds the burials of cycles whose view wasn't complete, plus this cycle's once
///   indexed.
/// - `pending` holds the burials not indexed yet. Indexing drains it.
/// - `unloaded_since` holds the listed transactions not loaded yet.
/// - `deferred` holds arrivals of cycles that couldn't read every block.
///
/// The end of a complete cycle with nothing deferred clears `unlinked`, `pending` and
/// `unloaded_since`.
#[derive(Clone, Default)]
pub struct Linker {
    /// What vanished since the blocks were last all applied, with each burial time.
    since_confirmed: Vec<(Txid, Moment)>,
    /// What vanished in cycles whose view wasn't complete, by each outpoint it spent.
    unlinked: RapidHashMap<(Txid, Vout), SmallVec<[Burial; 1]>>,
    /// This cycle's vanished burials and reclassifications, not indexed into `unlinked` yet.
    pending: Vec<(Txid, Moment)>,
    /// Listed transactions not loaded yet, with the cycle each was first listed in. Only a
    /// transaction listed by the cycle something vanished in can have replaced it.
    unloaded_since: RapidHashMap<Txid, u64>,
    /// Arrivals not relinked yet because their cycle couldn't read every block, with the cycle
    /// each was first listed in. A confirmed conflict takes precedence, so they wait for a cycle
    /// that reads them all.
    deferred: RapidHashMap<Txid, u64>,
    /// The current cycle's number.
    cycle: u64,
}

impl Linker {
    /// A linker after a restart: `since_confirmed` from the graveyard, no late replacer awaited.
    pub fn after_restore(since_confirmed: Vec<(Txid, Moment)>) -> Self {
        Self {
            since_confirmed,
            ..Self::default()
        }
    }

    /// For a recovery from a panic: keeps what `previous` awaited a late replacer for, which no
    /// publication holds. An entry for a burial the restored graveyard lacks is ignored when used.
    /// `since_confirmed` stays the restored graveyard's.
    pub fn keep_awaited(&mut self, previous: Self) {
        self.unlinked = previous.unlinked;
        self.unloaded_since = previous.unloaded_since;
        self.deferred = previous.deferred;
        self.cycle = previous.cycle;
    }

    /// Holds `txid`, an arrival of a cycle that couldn't read every block, for a later cycle.
    pub fn defer(&mut self, txid: Txid) {
        let listed = self.first_listed(&txid);
        self.deferred.entry(txid).or_insert(listed);
    }

    /// `txid` left the pool: its listing ended, so a later return starts a new one.
    pub fn undefer(&mut self, txid: &Txid) {
        self.deferred.remove(txid);
    }

    pub fn deferred(&self) -> Vec<Txid> {
        self.deferred.keys().copied().collect()
    }

    pub fn clear_deferred(&mut self) {
        self.deferred.clear();
    }

    /// The cycle `txid` was first listed in. One never left unloaded was first listed this cycle.
    fn first_listed(&self, txid: &Txid) -> u64 {
        self.deferred
            .get(txid)
            .or_else(|| self.unloaded_since.get(txid))
            .copied()
            .unwrap_or(self.cycle)
    }

    /// `txid` vanished at `at`, by burial or by reclassification.
    pub fn vanished(&mut self, txid: Txid, at: Moment) {
        self.since_confirmed.push((txid, at));
        self.pending.push((txid, at));
    }

    /// What vanished since the blocks were last all applied.
    pub fn since_confirmed(&self) -> &[(Txid, Moment)] {
        &self.since_confirmed
    }

    /// Every block was applied: later blocks cannot have displaced what vanished before now.
    pub fn blocks_confirmed(&mut self) {
        self.since_confirmed.clear();
    }

    /// Indexes this cycle's vanished burials by the outpoints they spent. `inputs` returns a
    /// burial's inputs while it is still that burial. Call before [`Self::losers`], so an arrival
    /// in the same cycle finds them.
    pub fn index<'a>(&mut self, inputs: impl Fn(&Txid, Moment) -> Option<&'a [TxIn]>) {
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

    /// What `arrival` replaced among the vanished, each once: those it conflicts with that vanished
    /// no earlier than it was first listed, and are still the burial indexed (`vanished_at`). A
    /// transaction listed only later, spending an outpoint the vanished one freed, replaced
    /// nothing.
    pub fn losers(
        &self,
        arrival: &Transaction,
        vanished_at: impl Fn(&Txid, Moment) -> bool,
    ) -> SmallVec<[Txid; 1]> {
        let mut losers: SmallVec<[Txid; 1]> = SmallVec::new();
        if self.unlinked.is_empty() {
            return losers;
        }
        let listed = self.first_listed(&arrival.txid);
        for txin in &arrival.input {
            let Some(burials) = self.unlinked.get(&(txin.txid, txin.vout)) else {
                continue;
            };
            for burial in burials {
                // Several of its inputs can lead to one loser: report it once.
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

    /// What a confirmed spend displaced among the burials waiting here. The rule is
    /// [`Self::losers`]'s: the spender must have been listed, and not loaded, by the cycle the
    /// burial vanished in. Returns `(loser, its burial time, spender)`, still to check against the
    /// tombstone.
    pub fn displaced(
        &self,
        spender: impl Fn(&(Txid, Vout)) -> Option<Txid>,
    ) -> Vec<(Txid, Moment, Txid)> {
        if self.unloaded_since.is_empty() {
            return Vec::new();
        }
        let mut displaced = Vec::new();
        for (outpoint, burials) in &self.unlinked {
            let Some(by) = spender(outpoint) else {
                continue;
            };
            let Some(&listed) = self.unloaded_since.get(&by) else {
                continue;
            };
            for burial in burials {
                if burial.txid != by && listed <= burial.cycle {
                    displaced.push((burial.txid, burial.at, by));
                }
            }
        }
        displaced
    }

    /// Ends a cycle. `unloaded` holds the listed transactions it couldn't load. If none, no
    /// replacer is still to arrive and everything awaiting one is dropped. Otherwise burials no
    /// longer vanished (`vanished_at`) are dropped, and the unloaded keep the cycle they were first
    /// listed in.
    pub fn end_cycle(
        &mut self,
        unloaded: &RapidHashSet<Txid>,
        vanished_at: impl Fn(&Txid, Moment) -> bool,
    ) {
        if unloaded.is_empty() && self.deferred.is_empty() {
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
