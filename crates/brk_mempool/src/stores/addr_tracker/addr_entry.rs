use std::{
    collections::{BTreeSet, btree_set},
    iter::Rev,
    slice,
};

use brk_types::{AddressMempoolStats, Timestamp, TxidPrefix};
use smallvec::SmallVec;

use crate::{key::TxKey, stores::tx_store::TxRecord};

/// Pending transactions an address keeps in a sorted inline vector before an ordered tree takes
/// over (and below half of it, back): either way an insert or a removal stays cheap however many
/// it has.
const SPILL: usize = 32;

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
    Many(BTreeSet<AddrTx>),
}

impl Default for AddrTxs {
    fn default() -> Self {
        Self::Few(SmallVec::new())
    }
}

impl AddrTxs {
    pub fn len(&self) -> usize {
        match self {
            Self::Few(txs) => txs.len(),
            Self::Many(txs) => txs.len(),
        }
    }

    pub fn contains(&self, tx: &AddrTx) -> bool {
        match self {
            Self::Few(txs) => txs.binary_search(tx).is_ok(),
            Self::Many(txs) => txs.contains(tx),
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
            (Self::Many(txs), None) => NewestBefore::Many(txs.range(..).rev()),
            (Self::Many(txs), Some(cursor)) => {
                if !txs.contains(cursor) {
                    return None;
                }
                NewestBefore::Many(txs.range(..*cursor).rev())
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
                    *self = Self::Many(txs.drain(..).collect());
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
                if txs.len() < SPILL / 2 {
                    *self = Self::Few(txs.iter().copied().collect());
                }
            }
        }
    }
}

/// [`AddrTxs::iter`].
pub enum AddrTxsIter<'a> {
    Few(slice::Iter<'a, AddrTx>),
    Many(btree_set::Iter<'a, AddrTx>),
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
    Many(Rev<btree_set::Range<'a, AddrTx>>),
}

impl<'a> Iterator for NewestBefore<'a> {
    type Item = &'a AddrTx;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Few(txs) => txs.next(),
            Self::Many(txs) => txs.next(),
        }
    }
}
