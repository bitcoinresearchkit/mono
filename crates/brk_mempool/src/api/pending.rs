use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use brk_error::{Error, Result};
use brk_types::{BlockHash, Txid, Vout};
use rapidhash::RapidHashMap;
use rustc_hash::FxHashSet;

use crate::{
    ReadOnlyState, json,
    state::{BehindSlot, Pool},
    key::{OutpointKey, TxKey},
};

/// How many reader tips past the pool's own tip keep a view. Readers of one moment share a tip, and
/// a few straddle the next block.
const BEHIND_TIPS: usize = 3;

/// A published pool as seen by a reader pinned at a chain tip.
///
/// At the pool's own tip the view hides nothing. Otherwise the reader's chain differs from the
/// pool's in one of two ways:
///
/// - The reader is ahead: its newer blocks mined or conflicted some pooled transactions. Those,
///   and everything spending from a conflicted one, are `settled` and hidden.
/// - A reorg moved the reader's chain off the pool's branch. The same applies from the newest
///   block both chains share, and what only the abandoned branch mined is in `limbo`
///   ([`Self::in_limbo`]).
pub struct PendingView {
    /// The tip of the publication this view was made from. The view reads only that publication.
    tip: BlockHash,
    /// The tip the reader is pinned at.
    reader: BlockHash,
    /// Pooled transactions hidden from this reader.
    settled: FxHashSet<TxKey>,
    /// Transactions whose fate this reader can't know yet.
    limbo: FxHashSet<Txid>,
}

impl PendingView {
    pub(crate) fn tip(&self) -> &BlockHash {
        &self.tip
    }

    pub(crate) fn settles_nothing(&self) -> bool {
        self.settled.is_empty()
    }

    /// Distinguishes views that hide different transactions: zero when nothing is hidden, otherwise
    /// a hash of the reader's tip. A publication has one view per reader tip.
    pub(crate) fn identity(&self) -> u64 {
        if self.settles_nothing() {
            return 0;
        }
        let mut hasher = json::keyed_hasher();
        self.reader.hash(&mut hasher);
        hasher.finish()
    }

    /// Whether a pooled transaction is still pending for this reader.
    pub(crate) fn pending(&self, txid: &Txid) -> bool {
        self.pending_key(&TxKey::from(txid))
    }

    /// [`Self::pending`] by the transaction's key.
    pub(crate) fn pending_key(&self, key: &TxKey) -> bool {
        self.settled.is_empty() || !self.settled.contains(key)
    }

    /// Whether this reader can't know `txid`'s fate yet, because a reorg isn't followed yet.
    ///
    /// True for a non-coinbase transaction only the abandoned branch mined, and for a pooled
    /// transaction spending from one. Core returns the first kind to its pool unless the reader's
    /// blocks conflict it. The pool knows which once it follows the reorg, a cycle later. Until
    /// then these transactions appear in no view, and a lookup of one answers `StateUpdating`.
    pub fn in_limbo(&self, txid: &Txid) -> bool {
        self.limbo.contains(txid)
    }
}

impl ReadOnlyState {
    /// The view for a reader pinned at the pool's own tip. Errors for any other tip.
    pub fn view_at(&self, tip: &BlockHash) -> Result<Arc<PendingView>> {
        self.pool()?.at(tip)?;
        Ok(Arc::new(PendingView {
            tip: *tip,
            reader: *tip,
            settled: FxHashSet::default(),
            limbo: FxHashSet::default(),
        }))
    }

    /// The view for a reader whose tip isn't the pool's.
    ///
    /// `left` counts the pool's newest blocks that the reader's chain doesn't hold: zero when the
    /// pool only trails the reader, more after a reorg. `spends` yields every input of the reader's
    /// blocks past the newest block both chains share, as the outpoint spent and the spending
    /// transaction.
    ///
    /// Each publication builds one view per reader tip. Readers of that tip wait for the first
    /// build and share it; readers of other tips don't wait. Costs O(those inputs, plus the inputs
    /// and outputs of the transactions hidden), and O(the pool's inputs) when `left` isn't zero.
    pub fn view_behind<E: From<Error>, S: IntoIterator<Item = ((Txid, Vout), Txid)>>(
        &self,
        reader: BlockHash,
        left: usize,
        spends: impl FnOnce() -> std::result::Result<S, E>,
    ) -> std::result::Result<Arc<PendingView>, E> {
        let pool = self.pool()?;
        let slot = slot_for(pool, reader);
        let mut built = slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(view) = built.as_ref() {
            return Ok(view.clone());
        }
        let spends: Spends = spends()?.into_iter().collect();
        let mut settled = settled_by(pool, &spends);
        let limbo = if left == 0 {
            FxHashSet::default()
        } else {
            limbo(pool, left, &spends, &mut settled)
        };
        let view = Arc::new(PendingView {
            tip: pool.tip,
            reader,
            settled,
            limbo,
        });
        *built = Some(view.clone());
        Ok(view)
    }
}

/// The inputs of the reader's blocks: each outpoint spent, with its spending transaction.
type Spends = RapidHashMap<(Txid, Vout), Txid>;

/// `reader`'s slot in `pool`, created if missing. The oldest slot goes when all are taken.
fn slot_for(pool: &Pool, reader: BlockHash) -> BehindSlot {
    let mut slots = pool
        .behind
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some((_, slot)) = slots.iter().find(|(tip, _)| *tip == reader) {
        return slot.clone();
    }
    if slots.len() == BEHIND_TIPS {
        slots.remove(0);
    }
    let slot = BehindSlot::default();
    slots.push((reader, slot.clone()));
    slot
}

/// The pooled transactions `spends` settles.
///
/// A pooled transaction spending one of those outpoints was either mined (the spender is itself)
/// or conflicted (the spender is another). Whatever spends from a conflicted one can't confirm
/// either, so its descendants are settled too. A mined transaction's children stay pending.
fn settled_by(pool: &Pool, spends: &Spends) -> FxHashSet<TxKey> {
    let mut candidates: Vec<TxKey> = spends
        .keys()
        .filter_map(|&(txid, vout)| {
            pool.outpoint_spends
                .get(&OutpointKey::new(TxKey::from(&txid), vout))
        })
        .collect();
    candidates.sort_unstable();
    candidates.dedup();
    let mut settled = FxHashSet::default();
    let mut conflicted = Vec::new();
    for record in candidates.iter().filter_map(|key| pool.txs.record_by_key(key)) {
        let tx = &record.tx;
        let Some(spender) = tx
            .input
            .iter()
            .find_map(|input| spends.get(&(input.txid, input.vout)))
        else {
            continue;
        };
        settled.insert(record.entry.key());
        if *spender != tx.txid {
            conflicted.push(record);
        }
    }
    while let Some(record) = conflicted.pop() {
        for child in pool.outpoint_spends.children(&pool.txs, &record.tx) {
            if settled.insert(child.entry.key()) {
                conflicted.push(child);
            }
        }
    }
    settled
}

/// The transactions in limbo for a reader whose chain left the pool's `left` newest blocks. Pooled
/// ones are added to `settled`, which hides them.
///
/// Limbo starts with what those blocks mined and the reader's blocks didn't. Every transaction a
/// block mines spends an input, so `spends` names all the reader's non-coinbase ones. Coinbases are
/// skipped: an abandoned block's coinbase is gone for good, and nothing pooled can spend one so
/// young. Limbo then grows by every pooled transaction spending from it, transitively.
fn limbo(
    pool: &Pool,
    left: usize,
    spends: &Spends,
    settled: &mut FxHashSet<TxKey>,
) -> FxHashSet<Txid> {
    let blocks = pool.ancestry.blocks();
    debug_assert!(left < blocks.len(), "the common block is one of the pool's");
    let reader_mined: FxHashSet<&Txid> = spends.values().collect();
    let mut limbo: FxHashSet<Txid> = blocks[blocks.len().saturating_sub(left)..]
        .iter()
        .flat_map(|block| block.txids.iter().skip(1))
        .filter(|txid| !reader_mined.contains(txid))
        .copied()
        .collect();
    if limbo.is_empty() {
        return limbo;
    }
    let mut orphans: Vec<_> = pool
        .txs
        .records()
        .map(|(_, record)| record)
        .filter(|record| record.tx.input.iter().any(|input| limbo.contains(&input.txid)))
        .collect();
    while let Some(record) = orphans.pop() {
        if limbo.insert(record.tx.txid) {
            settled.insert(record.entry.key());
            orphans.extend(pool.outpoint_spends.children(&pool.txs, &record.tx));
        }
    }
    limbo
}
