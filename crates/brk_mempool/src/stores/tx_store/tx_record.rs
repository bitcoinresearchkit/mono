use std::{
    hash::{DefaultHasher, Hasher},
    io::{self, Write},
    sync::Arc,
};

use brk_types::{FeeRate, SigOps, Transaction, TxIn, TxOut, Vin, Weight};
use serde_json::to_writer;

use crate::{json, state::TxEntry};

/// Live transaction body and its mempool entry, kept together so one
/// key lookup returns everything readers need.
#[derive(Clone)]
pub struct TxRecord {
    pub tx: Arc<Transaction>,
    pub entry: TxEntry,
    /// The keyed hash of the body's public JSON ([`json::keyed_hasher`]), refreshed whenever the
    /// body changes: templates hash these instead of reformatting every body.
    pub digest: u64,
}

impl TxRecord {
    pub fn new(tx: Arc<Transaction>, entry: TxEntry) -> Self {
        let digest = Self::digest_of(&tx);
        Self { tx, entry, digest }
    }

    /// Its sigop cost as Core's policy sees it: counted from the body once every prevout is
    /// resolved; before, the count misses their P2SH and witness parts, so the most Core's vsize
    /// allows stands in (a block's sigop budget never undercounts; the policy weight it gives is
    /// within 3 WU of the true one).
    pub fn policy_sigops(&self) -> SigOps {
        if !self.tx.is_resolved() {
            SigOps::bound(self.entry.vsize)
        } else {
            self.tx.total_sigop_cost
        }
    }

    /// What Core measures feerates over: the weight, sigop-adjusted.
    pub fn policy_weight(&self) -> Weight {
        self.policy_sigops().adjust_weight(self.entry.weight)
    }

    /// Its feerate alone, over the policy weight's exact quarter: the chunk rate of a
    /// transaction with no in-pool relatives.
    pub fn own_rate(&self) -> FeeRate {
        FeeRate::from((self.entry.fee, self.policy_weight()))
    }

    /// Fills missing prevouts (its body copied first if a publication shares it), its sigops and
    /// digest following.
    pub fn fill(&mut self, fills: Vec<(Vin, TxOut)>) {
        let tx = Arc::make_mut(&mut self.tx);
        for (vin, prevout) in fills {
            let txin = &mut tx.input[usize::from(vin)];
            debug_assert!(txin.prevout.is_none(), "fills only fill missing prevouts");
            txin.prevout = Some(prevout);
        }
        tx.refresh_sigops();
        self.digest = Self::digest_of(tx);
    }

    /// The memory it takes, about: the record, the transaction, its inputs and outputs, their
    /// scripts, witnesses and filled prevouts (what a serialized size undercounts two to four
    /// times).
    pub fn memory(&self) -> u64 {
        let tx = &self.tx;
        let inputs: usize = tx
            .input
            .iter()
            .map(|txin| {
                size_of::<TxIn>()
                    + txin.script_sig.len()
                    + txin.witness.size()
                    + txin
                        .prevout
                        .as_ref()
                        .map_or(0, |prevout| prevout.script_pubkey.len())
            })
            .sum();
        let outputs: usize = tx
            .output
            .iter()
            .map(|txout| size_of::<TxOut>() + txout.script_pubkey.len())
            .sum();
        (size_of::<Self>() + size_of::<Transaction>() + inputs + outputs) as u64
    }

    pub fn digest_of(tx: &Transaction) -> u64 {
        let mut hasher = HashWriter(json::keyed_hasher());
        to_writer(&mut hasher, tx).expect("transaction fields serialize");
        hasher.0.finish()
    }
}

/// Hashes written bytes without buffering them.
struct HashWriter(DefaultHasher);

impl Write for HashWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.write(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
