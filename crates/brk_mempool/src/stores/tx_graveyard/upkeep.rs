//! What only the writer keeps about the graveyard, beside what publications share: when each
//! tombstone is due to go ([`Retention`]), and what vanished while something can still have
//! displaced it ([`Linker`]).

use std::{collections::BTreeMap, time::Duration};

use brk_types::Txid;
use rustc_hash::FxHashMap;

use super::{Linker, Moment, TxGraveyard, TxTombstone};

/// How long a lone tombstone stays.
const RETENTION: Duration = Duration::from_hours(1);
/// How long a replacement tree stays once its root left the pool (mempool.space's): while that
/// one is pending, the whole tree does.
const TREE_RETENTION: Duration = Duration::from_hours(24);

/// The writer's bookkeeping for one graveyard, passed to whatever changes it.
#[derive(Default)]
pub struct Upkeep {
    pub(super) linker: Linker,
    pub(super) retention: Retention,
}

impl Upkeep {
    /// One for `graveyard` as read back (a restart, or the writer's recovery from a
    /// publication): each lone tombstone and root due a retention after its burial (a root
    /// never buried, after its newest version's), and what vanished since the blocks were last
    /// all told awaiting them.
    pub fn of(graveyard: &TxGraveyard) -> Self {
        let mut retention = Retention::default();
        for (txid, tombstone) in graveyard.tombstones.iter() {
            if let Some(kind) = graveyard.kind(txid) {
                retention.file(*txid, kind, tombstone.removed_at);
            }
        }
        for (root, predecessors) in graveyard.predecessors_by_replacer.iter() {
            if graveyard.get(root).is_none() {
                let newest = predecessors
                    .iter()
                    .filter_map(|txid| graveyard.get(txid).map(|tombstone| tombstone.removed_at))
                    .max()
                    .unwrap_or_else(Moment::now);
                retention.file(*root, Kind::Root, newest);
            }
        }
        Self {
            linker: Linker::after_restore(graveyard.vanished_since_told()),
            retention,
        }
    }
}

/// What a tombstone is to its retention. One `Replaced` is a version of its tree: it goes with
/// the tree, never on its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    /// Replaced by none, replacing none: it goes [`RETENTION`] after it became so.
    Lone,
    /// Replaced by none, replacing others (or a pending or unseen replacer, without a tombstone
    /// of its own): its tree goes whole [`TREE_RETENTION`] after it became one, was buried, or
    /// was last seen pending.
    Root,
}

impl Kind {
    fn retention(self) -> Duration {
        match self {
            Self::Lone => RETENTION,
            Self::Root => TREE_RETENTION,
        }
    }
}

/// Every lone tombstone and tree root with when it is due: exactly those, each once.
#[derive(Default)]
pub(super) struct Retention {
    filed: FxHashMap<Txid, (Kind, Moment)>,
    /// The same by due moment, the lone and the roots apart.
    lone: BTreeMap<(Moment, [u8; 32]), Txid>,
    roots: BTreeMap<(Moment, [u8; 32]), Txid>,
}

impl Retention {
    /// `txid` is of `kind` from `at` on, due a retention later (filed before, it is moved).
    pub fn file(&mut self, txid: Txid, kind: Kind, at: Moment) {
        self.unfile(&txid);
        let due = at + kind.retention();
        self.filed.insert(txid, (kind, due));
        self.queue(kind).insert((due, *txid), txid);
    }

    /// `txid` is neither lone nor a root any more.
    pub fn unfile(&mut self, txid: &Txid) {
        if let Some((kind, due)) = self.filed.remove(txid) {
            self.queue(kind).remove(&(due, **txid));
        }
    }

    /// The first of `kind` to come due, and when.
    pub fn first(&self, kind: Kind) -> Option<(Moment, Txid)> {
        let queue = match kind {
            Kind::Lone => &self.lone,
            Kind::Root => &self.roots,
        };
        queue.first_key_value().map(|((due, _), txid)| (*due, *txid))
    }

    pub fn count(&self, kind: Kind) -> usize {
        match kind {
            Kind::Lone => self.lone.len(),
            Kind::Root => self.roots.len(),
        }
    }

    pub fn kind(&self, txid: &Txid) -> Option<Kind> {
        self.filed.get(txid).map(|(kind, _)| *kind)
    }

    pub fn len(&self) -> usize {
        self.filed.len()
    }

    fn queue(&mut self, kind: Kind) -> &mut BTreeMap<(Moment, [u8; 32]), Txid> {
        match kind {
            Kind::Lone => &mut self.lone,
            Kind::Root => &mut self.roots,
        }
    }
}

impl TxGraveyard {
    /// What `txid` is to its retention, as the graveyard's links tell: none for a replaced
    /// version, or for one neither buried nor replacing any.
    pub(super) fn kind(&self, txid: &Txid) -> Option<Kind> {
        let replacing = self.predecessors_by_replacer.contains_key(txid);
        match self.get(txid).map(TxTombstone::replaced_by) {
            Some(Some(_)) => None,
            Some(None) if replacing => Some(Kind::Root),
            Some(None) => Some(Kind::Lone),
            None => replacing.then_some(Kind::Root),
        }
    }
}
