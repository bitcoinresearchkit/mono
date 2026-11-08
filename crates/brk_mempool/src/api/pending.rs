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

/// The reader tips past a pool's own whose views it keeps: a moment's readers share one tip, a
/// few straddle the next block.
const BEHIND_TIPS: usize = 3;

/// The pool as a reader pinned at a chain tip sees it: at that tip, as published; behind it, or on
/// a branch the reader's chain left (a reorg the pool hasn't followed yet), without what the
/// reader's blocks past the two chains' last common one settled: the transactions they mined,
/// those they conflicted (spent an input of) and those descending from a conflicted one. On a left
/// branch, without what's in limbo either ([`Self::in_limbo`]).
pub struct PendingView {
    /// The pool's tip: the view reads only the publication it was made from.
    tip: BlockHash,
    /// The reader's.
    reader: BlockHash,
    settled: FxHashSet<TxKey>,
    limbo: FxHashSet<Txid>,
}

impl PendingView {
    pub(crate) fn tip(&self) -> &BlockHash {
        &self.tip
    }

    pub(crate) fn settles_nothing(&self) -> bool {
        self.settled.is_empty()
    }

    /// Tells views apart where what they settle differs: zero at the pool's tip (nothing
    /// settled), the reader's tip hashed otherwise (one view per publication and reader tip).
    pub(crate) fn identity(&self) -> u64 {
        if self.settles_nothing() {
            return 0;
        }
        let mut hasher = json::keyed_hasher();
        self.reader.hash(&mut hasher);
        hasher.finish()
    }

    /// Whether `txid`, pooled, is still pending as the reader sees it.
    pub(crate) fn pending(&self, txid: &Txid) -> bool {
        self.pending_key(&TxKey::from(txid))
    }

    /// [`Self::pending`] by the transaction's key.
    pub(crate) fn pending_key(&self, key: &TxKey) -> bool {
        self.settled.is_empty() || !self.settled.contains(key)
    }

    /// Whether `txid` is unknown for the moment: mined only by the blocks of the pool's branch the
    /// reader's chain left (their coinbases aside), or pooled spending from one of those. Core
    /// puts the first back in its pool (unless the reader's blocks conflicted it, the second then
    /// gone too), and the pool has them as they are once it follows the reorg, a cycle later. In
    /// no view meanwhile.
    pub fn in_limbo(&self, txid: &Txid) -> bool {
        self.limbo.contains(txid)
    }
}

impl ReadOnlyState {
    /// The pool for a reader at `tip`, when it's there.
    pub fn view_at(&self, tip: &BlockHash) -> Result<Arc<PendingView>> {
        self.pool()?.at(tip)?;
        Ok(Arc::new(PendingView {
            tip: *tip,
            reader: *tip,
            settled: FxHashSet::default(),
            limbo: FxHashSet::default(),
        }))
    }

    /// The pool for a reader at `reader`, past the newest of [`Self::ancestry`] its chain holds,
    /// the pool's `left` newest blocks past that one (a reorg's branch; none when the pool only
    /// trails the reader): `spends` gives the reader's blocks past it, each input the outpoint it
    /// spends and its transaction. Made once per publication and reader tip: every read of the
    /// moment after a block shares one (those arriving meanwhile wait for it, readers at another
    /// tip don't). O(those inputs, the pooled spenders' inputs, and what they settle's outputs;
    /// past a left branch, the pool's inputs too).
    pub fn view_behind<E: From<Error>, S: IntoIterator<Item = ((Txid, Vout), Txid)>>(
        &self,
        reader: BlockHash,
        left: usize,
        spends: impl FnOnce() -> std::result::Result<S, E>,
    ) -> std::result::Result<Arc<PendingView>, E> {
        let state = self.pool()?;
        let slot = {
            let mut slots = state
                .behind
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            match slots.iter().find(|(tip, _)| *tip == reader) {
                Some((_, slot)) => slot.clone(),
                None => {
                    if slots.len() == BEHIND_TIPS {
                        slots.remove(0);
                    }
                    let slot = BehindSlot::default();
                    slots.push((reader, slot.clone()));
                    slot
                }
            }
        };
        let mut behind = slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(view) = behind.as_ref() {
            return Ok(view.clone());
        }
        let spends: RapidHashMap<(Txid, Vout), Txid> = spends()?.into_iter().collect();
        // The pooled spenders of those outpoints, each checked once against its own inputs: mined
        // when a block spends one of them by the transaction itself, conflicted when by another.
        let mut candidates: Vec<TxKey> = spends
            .keys()
            .filter_map(|&(txid, vout)| {
                state
                    .outpoint_spends
                    .get(&OutpointKey::new(TxKey::from(&txid), vout))
            })
            .collect();
        candidates.sort_unstable();
        candidates.dedup();
        let mut settled = FxHashSet::default();
        let mut conflicted = Vec::new();
        for record in candidates.iter().filter_map(|key| state.txs.record_by_key(key)) {
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
        // What spends a conflicted transaction's outputs can't confirm either.
        while let Some(record) = conflicted.pop() {
            for child in state.outpoint_spends.children(&state.txs, &record.tx) {
                if settled.insert(child.entry.key()) {
                    conflicted.push(child);
                }
            }
        }
        let limbo = if left == 0 {
            FxHashSet::default()
        } else {
            limbo(state, left, &spends, &mut settled)
        };
        let view = Arc::new(PendingView {
            tip: state.tip,
            reader,
            settled,
            limbo,
        });
        *behind = Some(view.clone());
        Ok(view)
    }
}

/// What the pool's `left` newest blocks mined that the reader's (`spends`) didn't (every
/// transaction they mined spends), then whatever `pool` holds spending from it, settled. Their
/// coinbases aren't: gone with their blocks for good, and nothing pooled spends one so young.
/// O(those blocks' transactions, and the pool's inputs when one is left).
fn limbo(
    pool: &Pool,
    left: usize,
    spends: &RapidHashMap<(Txid, Vout), Txid>,
    settled: &mut FxHashSet<TxKey>,
) -> FxHashSet<Txid> {
    let blocks = pool.ancestry.blocks();
    debug_assert!(left < blocks.len(), "a common block among the pool's");
    let mined: FxHashSet<&Txid> = spends.values().collect();
    let mut limbo: FxHashSet<Txid> = blocks[blocks.len().saturating_sub(left)..]
        .iter()
        .flat_map(|block| block.txids.iter().skip(1))
        .filter(|txid| !mined.contains(txid))
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
