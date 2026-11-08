use std::{iter::Rev, slice, sync::Arc};

use brk_types::{AddressMempoolStats, Timestamp, TxidPrefix};
use smallvec::SmallVec;

use crate::{key::TxKey, stores::tx_store::TxRecord};

/// Pending transactions an address keeps in a sorted inline vector before shared leaves take over
/// (and below half of it, back): either way an insert or a removal stays cheap however many it
/// has.
const SPILL: usize = 32;
/// The most transactions one of [`Leaves`] holds: what a write copies at most.
const LEAF: usize = 64;

/// Per-address mempool record: rolling stats plus the address's pending transactions in page
/// order (`AddrTx`'s).
#[derive(Clone, Default)]
pub struct AddrEntry {
    pub stats: AddressMempoolStats,
    pub txs: AddrTxs,
}

/// A pending transaction as an address's pages order it: by first sighting, then the txid's
/// prefix (a public tie-break), then its key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AddrTx {
    pub first_seen: Timestamp,
    pub prefix: TxidPrefix,
    pub key: TxKey,
}

impl From<&TxRecord> for AddrTx {
    fn from(record: &TxRecord) -> Self {
        Self {
            first_seen: record.entry.first_seen,
            prefix: TxidPrefix::from(&record.entry.txid),
            key: record.entry.key(),
        }
    }
}

/// An address's pending transactions, ascending.
#[derive(Clone)]
pub enum AddrTxs {
    Few(SmallVec<[AddrTx; 1]>),
    Many(Leaves),
}

/// A sorted set of an address's transactions in shared leaves. While a publication shares the
/// entry, the first write copies the list of leaf pointers and each write the leaf it touches, not
/// the transactions of the whole set. Leaves aren't merged: below [`SPILL`] / 2 transactions, the
/// set goes back to the inline vector.
#[derive(Clone, Default)]
pub struct Leaves {
    /// Sorted, none empty, each holding what sorts before the next one's first.
    leaves: Vec<Arc<Vec<AddrTx>>>,
    len: usize,
}

impl Leaves {
    /// The leaf that holds `tx`, if any does: the last one starting at or before it.
    fn leaf_of(&self, tx: &AddrTx) -> usize {
        self.leaves
            .partition_point(|leaf| leaf[0] <= *tx)
            .saturating_sub(1)
    }

    /// `tx`'s leaf and its place in it, when it's here.
    fn find(&self, tx: &AddrTx) -> Option<(usize, usize)> {
        let at = self.leaf_of(tx);
        let place = self.leaves.get(at)?.binary_search(tx).ok()?;
        Some((at, place))
    }

    fn insert(&mut self, tx: AddrTx) {
        let at = self.leaf_of(&tx);
        let Some(leaf) = self.leaves.get_mut(at) else {
            self.leaves.push(Arc::new(vec![tx]));
            self.len = 1;
            return;
        };
        let Err(place) = leaf.binary_search(&tx) else {
            return;
        };
        let leaf = Arc::make_mut(leaf);
        leaf.insert(place, tx);
        self.len += 1;
        if leaf.len() > LEAF {
            let upper = leaf.split_off(LEAF / 2);
            self.leaves.insert(at + 1, Arc::new(upper));
        }
    }

    fn remove(&mut self, tx: &AddrTx) {
        let Some((at, place)) = self.find(tx) else {
            return;
        };
        let leaf = Arc::make_mut(&mut self.leaves[at]);
        leaf.remove(place);
        self.len -= 1;
        if leaf.is_empty() {
            self.leaves.remove(at);
        }
    }

    fn iter(&self) -> LeavesIter<'_> {
        Self::flatten(&self.leaves)
    }

    fn flatten(leaves: &[Arc<Vec<AddrTx>>]) -> LeavesIter<'_> {
        leaves.iter().flat_map(|leaf: &Arc<Vec<AddrTx>>| leaf.iter())
    }
}

type LeavesIter<'a> = std::iter::FlatMap<
    slice::Iter<'a, Arc<Vec<AddrTx>>>,
    slice::Iter<'a, AddrTx>,
    fn(&'a Arc<Vec<AddrTx>>) -> slice::Iter<'a, AddrTx>,
>;

impl Default for AddrTxs {
    fn default() -> Self {
        Self::Few(SmallVec::new())
    }
}

impl AddrTxs {
    pub fn len(&self) -> usize {
        match self {
            Self::Few(txs) => txs.len(),
            Self::Many(txs) => txs.len,
        }
    }

    pub fn contains(&self, tx: &AddrTx) -> bool {
        match self {
            Self::Few(txs) => txs.binary_search(tx).is_ok(),
            Self::Many(txs) => txs.find(tx).is_some(),
        }
    }

    pub fn iter(&self) -> AddrTxsIter<'_> {
        match self {
            Self::Few(txs) => AddrTxsIter::Few(txs.iter()),
            Self::Many(txs) => AddrTxsIter::Many(txs.iter()),
        }
    }

    /// Newest first, all of them or those before `cursor` (strictly); none if `cursor` isn't one.
    pub fn newest_before(&self, cursor: Option<&AddrTx>) -> Option<NewestBefore<'_>> {
        Some(match (self, cursor) {
            (Self::Few(txs), None) => NewestBefore::Few(txs.iter().rev()),
            (Self::Few(txs), Some(cursor)) => {
                NewestBefore::Few(txs[..txs.binary_search(cursor).ok()?].iter().rev())
            }
            (Self::Many(txs), None) => NewestBefore::Many([].iter().rev(), txs.iter().rev()),
            (Self::Many(txs), Some(cursor)) => {
                let (at, place) = txs.find(cursor)?;
                NewestBefore::Many(
                    txs.leaves[at][..place].iter().rev(),
                    Leaves::flatten(&txs.leaves[..at]).rev(),
                )
            }
        })
    }

    pub(crate) fn insert(&mut self, tx: AddrTx) {
        match self {
            Self::Few(txs) => {
                if let Err(at) = txs.binary_search(&tx) {
                    txs.insert(at, tx);
                }
                if txs.len() > SPILL {
                    let mut many = Leaves::default();
                    txs.drain(..).for_each(|tx| many.insert(tx));
                    *self = Self::Many(many);
                }
            }
            Self::Many(txs) => {
                txs.insert(tx);
            }
        }
    }

    pub(crate) fn remove(&mut self, tx: &AddrTx) {
        match self {
            Self::Few(txs) => {
                if let Ok(at) = txs.binary_search(tx) {
                    txs.remove(at);
                }
            }
            Self::Many(txs) => {
                txs.remove(tx);
                if txs.len < SPILL / 2 {
                    *self = Self::Few(txs.iter().copied().collect());
                }
            }
        }
    }
}

/// [`AddrTxs::iter`].
pub enum AddrTxsIter<'a> {
    Few(slice::Iter<'a, AddrTx>),
    Many(LeavesIter<'a>),
}

impl<'a> Iterator for AddrTxsIter<'a> {
    type Item = &'a AddrTx;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Few(txs) => txs.next(),
            Self::Many(txs) => txs.next(),
        }
    }
}

/// [`AddrTxs::newest_before`].
pub enum NewestBefore<'a> {
    Few(Rev<slice::Iter<'a, AddrTx>>),
    /// What's before the cursor in its leaf, then the leaves before it.
    Many(Rev<slice::Iter<'a, AddrTx>>, Rev<LeavesIter<'a>>),
}

impl<'a> Iterator for NewestBefore<'a> {
    type Item = &'a AddrTx;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Few(txs) => txs.next(),
            Self::Many(leaf, leaves) => leaf.next().or_else(|| leaves.next()),
        }
    }
}
