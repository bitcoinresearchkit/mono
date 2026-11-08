//! What only the writer keeps about the graveyard: when each tombstone is due ([`Retention`]), and
//! which vanished ones may still turn out replaced ([`Linker`]). Publications never copy it.

use std::{collections::BTreeMap, time::Duration};

use brk_types::Txid;
use rustc_hash::FxHashMap;

use super::{Linker, Moment, TxGraveyard, TxTombstone};

/// How long a lone tombstone stays.
const RETENTION: Duration = Duration::from_hours(1);
/// How long a tree stays after its root left the pool, as on mempool.space. While the root is
/// pending, the whole tree stays.
const TREE_RETENTION: Duration = Duration::from_hours(24);

/// The writer's bookkeeping for one graveyard. Every change to the graveyard takes it.
#[derive(Default)]
pub struct Upkeep {
    pub(super) linker: Linker,
    pub(super) retention: Retention,
}

impl Upkeep {
    /// The bookkeeping for a graveyard read back, after a restart or a recovery from a publication.
    /// Each lone tombstone and root is due one retention after its burial. A root with no tombstone
    /// counts from its newest version's burial.
    pub fn of(graveyard: &TxGraveyard) -> Self {
        let mut retention = Retention::default();
        for (txid, tombstone) in graveyard.tombstones.iter() {
            if let Some(kind) = graveyard.kind(txid) {
                retention.schedule(*txid, kind, tombstone.removed_at);
            }
        }
        for (root, predecessors) in graveyard.predecessors_by_replacer.iter() {
            if graveyard.get(root).is_none() {
                let newest = predecessors
                    .iter()
                    .filter_map(|txid| graveyard.get(txid).map(|tombstone| tombstone.removed_at))
                    .max()
                    .unwrap_or_else(Moment::now);
                retention.schedule(*root, Kind::Root, newest);
            }
        }
        Self {
            linker: Linker::after_restore(graveyard.vanished_since_confirmed()),
            retention,
        }
    }

    /// For a recovery from a panic: keeps what `previous` awaited a late replacer for
    /// ([`Linker::keep_awaited`]).
    pub fn keep_awaited(&mut self, previous: Self) {
        self.linker.keep_awaited(previous.linker);
    }
}

/// A tombstone's kind, for retention. The third kind, a version, is never scheduled: it goes with
/// its tree. The module docs have the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    /// Replaced by none and replacing none. Goes [`RETENTION`] after becoming lone.
    Lone,
    /// Replaced by none and replacing others, with or without a tombstone. Its whole tree goes
    /// [`TREE_RETENTION`] after it became a root, was buried, or was last seen pending.
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

/// When each lone tombstone and root is due. Holds exactly those, each once.
#[derive(Default)]
pub(super) struct Retention {
    scheduled: FxHashMap<Txid, (Kind, Moment)>,
    /// The same entries ordered by due time, one queue per kind.
    lone: BTreeMap<(Moment, [u8; 32]), Txid>,
    roots: BTreeMap<(Moment, [u8; 32]), Txid>,
}

impl Retention {
    /// Schedules `txid` as `kind`, due one retention after `at`. Moves it if it was scheduled.
    pub fn schedule(&mut self, txid: Txid, kind: Kind, at: Moment) {
        self.unschedule(&txid);
        let due = at + kind.retention();
        self.scheduled.insert(txid, (kind, due));
        self.queue(kind).insert((due, *txid), txid);
    }

    /// `txid` is no longer lone or a root.
    pub fn unschedule(&mut self, txid: &Txid) {
        if let Some((kind, due)) = self.scheduled.remove(txid) {
            self.queue(kind).remove(&(due, **txid));
        }
    }

    /// The next of `kind` to come due, and when.
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
        self.scheduled.get(txid).map(|(kind, _)| *kind)
    }

    pub fn len(&self) -> usize {
        self.scheduled.len()
    }

    fn queue(&mut self, kind: Kind) -> &mut BTreeMap<(Moment, [u8; 32]), Txid> {
        match kind {
            Kind::Lone => &mut self.lone,
            Kind::Root => &mut self.roots,
        }
    }
}

impl TxGraveyard {
    /// `txid`'s kind according to the graveyard's links. `None` for a version, and for a
    /// transaction neither buried nor replacing any.
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
