//! Process-keyed 8-byte ids for the live stores.
//!
//! Txids are attacker-chosen: grinding two transactions to share a raw 8-byte
//! txid prefix costs ~2^32 SHA-256d, which would let one live record overwrite
//! another. Keys are instead a SipHash of the full txid under a random key
//! drawn once per process, so collisions cannot be ground offline.

use std::{
    hash::{BuildHasher, RandomState},
    sync::LazyLock,
};

use brk_types::{Txid, Vout};
use derive_more::Deref;

static SEED: LazyLock<RandomState> = LazyLock::new(RandomState::new);

#[derive(Debug, Deref, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TxKey(u64);

impl From<&Txid> for TxKey {
    #[inline]
    fn from(txid: &Txid) -> Self {
        Self(SEED.hash_one(txid))
    }
}

/// `(TxKey, Vout)` outpoint id. Callers verify the full outpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutpointKey(TxKey, Vout);

impl OutpointKey {
    #[inline]
    pub fn new(tx: TxKey, vout: Vout) -> Self {
        Self(tx, vout)
    }
}
