use std::{
    hash::Hasher,
    io::{self, Write},
    sync::Arc,
};

use brk_types::Transaction;
use rustc_hash::FxHasher;
use serde_json::to_writer;

use crate::state::TxEntry;

/// Live transaction body and its mempool entry, kept together so one
/// prefix lookup returns everything readers need.
#[derive(Clone)]
pub struct TxRecord {
    pub tx: Arc<Transaction>,
    pub entry: TxEntry,
    /// Hash of the body's public JSON, refreshed whenever the body changes: templates hash
    /// these instead of reformatting every body.
    pub digest: u64,
}

impl TxRecord {
    pub fn new(tx: Arc<Transaction>, entry: TxEntry) -> Self {
        let digest = Self::digest_of(&tx);
        Self { tx, entry, digest }
    }

    pub fn digest_of(tx: &Transaction) -> u64 {
        let mut hasher = HashWriter(FxHasher::default());
        to_writer(&mut hasher, tx).expect("transaction fields serialize");
        hasher.0.finish()
    }
}

/// Hashes written bytes without buffering them.
struct HashWriter(FxHasher);

impl Write for HashWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.write(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
