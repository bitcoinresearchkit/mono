use brk_types::{BlockHash, FeeRate, Transaction, Txid};
use rapidhash::{RapidHashSet, fast::RandomState};
use smallvec::SmallVec;

mod eviction;
pub mod file;
pub mod linker;
mod moment;
pub mod tombstone;

pub use file::GraveyardFile;
pub use linker::Linker;
pub use moment::Moment;
pub use tombstone::TxTombstone;

use crate::{
    TxRemoval,
    steps::Confirmed,
    stores::{
        cow::{CowMap, CowQueue},
        tx_store::TxRecord,
    },
};

/// Recently-dropped txs retained for reappearance (a returning tx keeps its first sighting) and
/// post-mine analytics (RBF trees, removal reasons).
///
/// A tombstone is lone, or one version of a replacement tree: the versions `Replaced { by }`
/// links, rooted at the one no version replaced (pending, buried, or a confirmed replacer the
/// pool never saw). A lone one goes an hour after its burial; a tree stays while its root is
/// pending, and goes whole a day after it left ([`Self::evict_old`]).
///
/// Invariants: `predecessors_by_replacer[by]` lists exactly the `Replaced { by }` tombstones,
/// `replaced` counts them, `bytes` sums the bodies, and the links close no cycle (a replacer
/// with a tombstone arrives, exhumed, or is told mined within the cycle that links it). `filed`
/// holds exactly the lone tombstones and the roots, each with the time of its entry in `lone` or
/// `trees`; `replacements` holds one entry for every `Replaced` tombstone, at its burial. Each
/// queue keeps time order, and an entry `filed` no longer gives (or whose txid changed state since)
/// is stale, skipped where read and dropped once they outnumber the rest. What vanished, until
/// the blocks or a late replacer can no longer have displaced it, the writer's [`Linker`] holds
/// apart (publications never copy it), passed to what needs it. A
/// restart files each tombstone again at its burial: one that became lone, or a root seen pending,
/// after it can go up to a retention early.
#[derive(Clone, Default)]
pub struct TxGraveyard {
    revision: u64,
    tombstones: CowMap<Txid, TxTombstone, RandomState>,
    predecessors_by_replacer: CowMap<Txid, SmallVec<[Txid; 1]>, RandomState>,
    /// Lone tombstones, by when each became lone.
    lone: CowQueue<(Moment, Txid)>,
    /// Tree roots, by when each became one, was buried, or was last seen pending.
    trees: CowQueue<(Moment, Txid)>,
    /// Each lone tombstone's and root's entry time in `lone` or `trees`: an entry at another is
    /// stale.
    filed: CowMap<Txid, Moment, RandomState>,
    /// The `Replaced` burials, so RBF listings never scan the far more numerous mined and dropped
    /// txs.
    replacements: CowQueue<(Moment, Txid)>,
    /// The memory every body held takes ([`TxRecord::memory`]).
    bytes: u64,
    /// How many `Replaced` tombstones are held.
    replaced: usize,
    /// The tip whose blocks it was last told (`confirm`): the next cycle reads the blocks since,
    /// and so does one after a restart (it is saved with it).
    told: Option<BlockHash>,
    /// When every block up to `told` was last told: what was buried before, the blocks since
    /// didn't displace.
    told_at: Option<Moment>,
}

impl TxGraveyard {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn told(&self) -> Option<BlockHash> {
        self.told
    }

    /// When every block up to [`Self::told`] was last told.
    pub fn told_at(&self) -> Option<Moment> {
        self.told_at
    }

    /// `told` and `told_at` read back from disk.
    fn restore_told(&mut self, tip: BlockHash, at: Moment) {
        self.told = Some(tip);
        self.told_at = Some(at);
        self.revision = self.revision.wrapping_add(1);
    }

    /// What vanished since the blocks were last all told, each with its burial: what the blocks
    /// since can have displaced ([`Linker::after_restore`]).
    pub fn vanished_since_told(&self) -> Vec<(Txid, Moment)> {
        self.tombstones
            .iter()
            .filter(|(_, tombstone)| {
                matches!(tombstone.removal, TxRemoval::Vanished)
                    && self.told_at.is_none_or(|at| tombstone.removed_at > at)
            })
            .map(|(txid, tombstone)| (*txid, tombstone.removed_at))
            .collect()
    }

    /// Every block up to `tip` told: a change publications and saves carry.
    pub fn set_told(&mut self, tip: BlockHash, linker: &mut Linker) {
        self.told_at = Some(Moment::now());
        linker.told();
        if self.told != Some(tip) {
            self.told = Some(tip);
            self.revision = self.revision.wrapping_add(1);
        }
    }

    pub fn get(&self, txid: &Txid) -> Option<&TxTombstone> {
        self.tombstones.get(txid)
    }

    /// Tombstone iff the tx left the pool without a replacer (mined, expired, or dropped);
    /// `Replaced` tombstones are excluded.
    pub fn get_unreplaced(&self, txid: &Txid) -> Option<&TxTombstone> {
        let tomb = self.tombstones.get(txid)?;
        tomb.replaced_by().is_none().then_some(tomb)
    }

    /// `txid`'s replacer, when it is still known (`live`, or buried): one step of a walk to the
    /// newest version the pool knows (`api::rbf`'s `tree_root`).
    pub fn next_version(&self, txid: &Txid, live: impl Fn(&Txid) -> bool) -> Option<Txid> {
        self.tombstones
            .get(txid)
            .and_then(TxTombstone::replaced_by)
            .filter(|by| live(by) || self.tombstones.contains_key(by))
            .copied()
    }

    /// Tombstones marked as `Replaced { by: replacer }`. Used to walk
    /// backward through RBF history: given a tx that's still live (or
    /// in the graveyard), find every tx it displaced.
    pub fn predecessors_of<'a>(
        &'a self,
        replacer: &'a Txid,
    ) -> impl Iterator<Item = (&'a Txid, &'a TxTombstone)> {
        self.predecessors_by_replacer
            .get(replacer)
            .into_iter()
            .flatten()
            .filter_map(|txid| self.tombstones.get(txid).map(|tombstone| (txid, tombstone)))
    }

    /// Every `Replaced` tombstone, yielded as (`predecessor_txid`,
    /// `replacer_txid`) in reverse bury order (most recent replacement
    /// event first). Caller walks the replacer chain forward to find
    /// each tree's terminal replacer.
    ///
    /// `replacements` may carry stale entries (re-buries, prior exhumes). The
    /// `removed_at == t` check marks those as `None`, so callers can charge
    /// every scanned entry against their work budget, including stale entries.
    pub fn replacements_recent_first(&self) -> impl Iterator<Item = Option<(&Txid, &Txid)>> {
        self.replacements.iter().rev().map(|(t, txid)| {
            let ts = self.tombstones.get(txid)?;
            if ts.removed_at != *t {
                return None;
            }
            Some((txid, ts.replaced_by()?))
        })
    }

    /// Buries `record`: one vanished, `linker` waits for what can have displaced it (one read
    /// back from disk, [`Self::vanished_since_told`] tells).
    pub fn bury(
        &mut self,
        record: TxRecord,
        chunk_rate: FeeRate,
        removal: TxRemoval,
        linker: &mut Linker,
    ) {
        let (txid, now) = (record.entry.txid, Moment::now());
        self.insert(record, chunk_rate, removal, now);
        if matches!(removal, TxRemoval::Vanished) {
            linker.vanished(txid, now);
        }
    }

    /// A tombstone read back from disk, buried when it was (oldest first, as the queues keep
    /// time order).
    fn restore(
        &mut self,
        record: TxRecord,
        chunk_rate: FeeRate,
        removal: TxRemoval,
        removed_at: Moment,
    ) {
        self.insert(record, chunk_rate, removal, removed_at);
    }

    fn insert(
        &mut self,
        record: TxRecord,
        chunk_rate: FeeRate,
        removal: TxRemoval,
        removed_at: Moment,
    ) {
        let txid = record.entry.txid;
        self.bytes += record.memory();
        let tombstone = TxTombstone {
            record,
            chunk_rate,
            removal,
            removed_at,
        };
        if let Some(previous) = self.tombstones.insert(txid, tombstone) {
            self.bytes -= previous.record.memory();
            if let Some(by) = previous.replaced_by() {
                self.replaced -= 1;
                self.unlist(&txid, by, removed_at);
            }
        }
        self.told_removal(txid, removal, removed_at);
        self.revision = self.revision.wrapping_add(1);
    }

    /// Files `txid`'s tombstone, just told `removal` at `at`: a replaced version under its
    /// replacer, a root (it replaced others) in `trees`, any other in `lone`.
    fn told_removal(&mut self, txid: Txid, removal: TxRemoval, at: Moment) {
        if let TxRemoval::Replaced { by } = removal {
            self.replaced += 1;
            self.filed.remove(&txid);
            self.list(txid, by, at);
            self.replacements.push_back((at, txid));
        } else if self.predecessors_by_replacer.contains_key(&txid) {
            self.file_root(txid, at);
        } else {
            self.file_lone(txid, at);
        }
    }

    fn file_lone(&mut self, txid: Txid, at: Moment) {
        self.filed.insert(txid, at);
        self.lone.push_back((at, txid));
    }

    fn file_root(&mut self, txid: Txid, at: Moment) {
        self.filed.insert(txid, at);
        self.trees.push_back((at, txid));
    }

    /// Lists `txid` among `by`'s predecessors, at `at`: `by` roots a tree from then on, unless
    /// it is itself a replaced version (its tree's root has its entry).
    fn list(&mut self, txid: Txid, by: Txid, at: Moment) {
        let predecessors = self
            .predecessors_by_replacer
            .shard_mut(&by)
            .entry(by)
            .or_default();
        let first = predecessors.is_empty();
        predecessors.push(txid);
        if first && self.get(&by).is_none_or(|tombstone| tombstone.replaced_by().is_none()) {
            self.file_root(by, at);
        }
    }

    /// Takes `txid` off `by`'s predecessors, at `at`: `by`, replacing none any more, is lone from
    /// then on if buried (and replaced by none), and nothing if not.
    fn unlist(&mut self, txid: &Txid, by: &Txid, at: Moment) {
        let emptied = self
            .predecessors_by_replacer
            .get_mut(by)
            .is_some_and(|predecessors| {
                predecessors.retain(|predecessor| predecessor != txid);
                predecessors.is_empty()
            });
        if !emptied {
            return;
        }
        self.predecessors_by_replacer.remove(by);
        match self.get(by).map(|tombstone| tombstone.replaced_by().is_none()) {
            Some(true) => self.file_lone(*by, at),
            Some(false) => {}
            None => {
                self.filed.remove(by);
            }
        }
    }

    /// Drop the tombstone of a tx that came back to life (its predecessors stay: their replacer
    /// is pending again).
    pub fn exhume(&mut self, txid: &Txid) {
        self.remove(txid, Moment::now());
    }

    /// Removes `txid`'s tombstone, if any, at `at`: one that replaced others roots their tree
    /// from then on (it is pending again).
    fn remove(&mut self, txid: &Txid, at: Moment) -> Option<TxTombstone> {
        let tombstone = self.tombstones.remove(txid)?;
        self.revision = self.revision.wrapping_add(1);
        self.bytes -= tombstone.record.memory();
        if let Some(by) = tombstone.replaced_by() {
            self.replaced -= 1;
            self.unlist(txid, by, at);
        }
        if self.predecessors_by_replacer.contains_key(txid) {
            self.file_root(*txid, at);
        } else {
            self.filed.remove(txid);
        }
        Some(tombstone)
    }

    /// Removes `txid`'s tombstone and every version it replaced, recursively: a tree, or the
    /// part of one under a version.
    fn remove_tree(&mut self, txid: Txid, at: Moment) {
        let mut stack = vec![txid];
        while let Some(txid) = stack.pop() {
            if let Some(predecessors) = self.predecessors_by_replacer.get(&txid) {
                stack.extend(predecessors.iter().copied());
            }
            self.remove(&txid, at);
        }
    }

    /// The tombstone buried at `at`, unless a later burial or an exhume replaced it.
    fn current(&self, txid: &Txid, at: Moment) -> Option<&TxTombstone> {
        self.tombstones
            .get(txid)
            .filter(|tombstone| tombstone.removed_at == at)
    }

    /// What new blocks confirmed, told to the buried: one they mined reads mined from now on (a
    /// replaced version mined after all, which also ends a cycle its replacer's confirmed conflict
    /// closed, or one buried vanished); one mined by a block a reorg took off the chain that
    /// neither returned nor was mined again reads vanished, or replaced by a conflict the new
    /// blocks confirmed; a vanished one a confirmed conflict displaced reads replaced by it.
    /// Displaced: buried since the blocks were last all told ([`Linker::since_told`]: a cycle
    /// that couldn't read its blocks buried them vanished, the next cycle's read tells them
    /// here), or by a late replacer the pool listed by then and never loaded
    /// ([`Linker::displaced`], in memory until the next complete view: a restart or an unread
    /// block in between leaves such a one vanished, as a replacer never seen does). An outpoint
    /// respent after its transaction left (evicted, expired) replaced nothing. O(the blocks'
    /// transactions, and what vanished since they were last all told or waits for a late
    /// replacer).
    pub fn confirm(&mut self, blocks: &Confirmed, linker: &mut Linker) {
        let now = Moment::now();
        let mut changes: Vec<(Txid, TxRemoval)> = Vec::new();
        for txid in blocks.mined_txids() {
            if self
                .get(txid)
                .is_some_and(|tombstone| !matches!(tombstone.removal, TxRemoval::Mined))
            {
                changes.push((*txid, TxRemoval::Mined));
            }
        }
        let conflict = |tombstone: &TxTombstone| {
            let tx = &tombstone.record.tx;
            blocks
                .conflicts(&tx.txid, tx.input.iter().map(|txin| (&txin.txid, txin.vout)))
                .next()
        };
        for txid in blocks.disconnected_txids() {
            if let Some(tombstone) = self.get(txid)
                && matches!(tombstone.removal, TxRemoval::Mined)
                && !blocks.mined(txid)
            {
                let removal = conflict(tombstone)
                    .map_or(TxRemoval::Vanished, |by| TxRemoval::Replaced { by });
                changes.push((*txid, removal));
            }
        }
        let mut displaced: Vec<(Txid, Moment, Txid)> = linker
            .since_told()
            .iter()
            .filter_map(|&(txid, at)| {
                let tombstone = self.current(&txid, at)?;
                Some((txid, at, conflict(tombstone)?))
            })
            .collect();
        displaced.extend(linker.displaced(|outpoint| blocks.spender(outpoint)));
        let mut told: RapidHashSet<Txid> = changes.iter().map(|(txid, _)| *txid).collect();
        for (txid, at, by) in displaced {
            if !blocks.mined(&txid) && Self::is_vanished(&self.tombstones, &txid, at) && told.insert(txid) {
                changes.push((txid, TxRemoval::Replaced { by }));
            }
        }
        for (txid, removal) in changes {
            self.reclassify(txid, removal, now, linker);
        }
    }

    /// A tombstone told another reason, as of `now`: its retention starts again.
    fn reclassify(&mut self, txid: Txid, removal: TxRemoval, now: Moment, linker: &mut Linker) {
        let Some(tombstone) = self.tombstones.get_mut(&txid) else {
            return;
        };
        let previous = tombstone.replaced_by().copied();
        tombstone.removal = removal;
        tombstone.removed_at = now;
        if let Some(by) = previous {
            self.replaced -= 1;
            self.unlist(&txid, &by, now);
        }
        self.told_removal(txid, removal, now);
        if matches!(removal, TxRemoval::Vanished) {
            linker.vanished(txid, now);
        }
        self.revision = self.revision.wrapping_add(1);
    }

    /// An arrival the fetch cap held back, replacing what vanished meanwhile: what it conflicts
    /// with that vanished no earlier than it was first listed reads replaced by it ([`Linker`]).
    /// Called after the cycle's blocks are told (`confirm`), and only when they all could be: a
    /// confirmed conflict comes first.
    pub fn relink(&mut self, arrival: &Transaction, linker: &mut Linker) {
        let now = Moment::now();
        let tombstones = &self.tombstones;
        let losers = linker.losers(arrival, |txid, at| Self::is_vanished(tombstones, txid, at));
        for loser in losers {
            self.reclassify(loser, TxRemoval::Replaced { by: arrival.txid }, now, linker);
        }
    }

    /// This cycle's vanished, indexed for [`Self::relink`]: before it, so a same-cycle arrival
    /// finds them too.
    pub fn index_vanished(&self, linker: &mut Linker) {
        let tombstones = &self.tombstones;
        linker.index(|txid, at| {
            tombstones
                .get(txid)
                .filter(|tombstone| tombstone.removed_at == at)
                .map(|tombstone| tombstone.record.tx.input.as_slice())
        });
    }

    /// The cycle's end, with the listed transactions it couldn't load ([`Linker::end_cycle`]).
    pub fn end_cycle(&self, unloaded: &RapidHashSet<Txid>, linker: &mut Linker) {
        let tombstones = &self.tombstones;
        linker.end_cycle(unloaded, |txid, at| Self::is_vanished(tombstones, txid, at));
        debug_assert!(self.consistent(), "graveyard invariants");
    }

    /// Whether no chain of `Replaced { by }` links returns to where it started. O(tombstones).
    fn acyclic(&self) -> bool {
        let mut done: RapidHashSet<Txid> = RapidHashSet::default();
        for (start, _) in self.tombstones.iter() {
            let mut path: RapidHashSet<Txid> = RapidHashSet::default();
            let mut at = *start;
            while !done.contains(&at) {
                if !path.insert(at) {
                    return false;
                }
                match self.get(&at).and_then(TxTombstone::replaced_by) {
                    Some(by) => at = *by,
                    None => break,
                }
            }
            done.extend(path);
        }
        true
    }

    /// Whether `txid`'s tombstone is the vanished one buried at `at`.
    fn is_vanished(
        tombstones: &CowMap<Txid, TxTombstone, RandomState>,
        txid: &Txid,
        at: Moment,
    ) -> bool {
        tombstones.get(txid).is_some_and(|tombstone| {
            tombstone.removed_at == at && matches!(tombstone.removal, TxRemoval::Vanished)
        })
    }
}
