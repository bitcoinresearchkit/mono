use bitcoin::hex::DisplayHex;
use brk_error::Result;
use brk_types::{Transaction, Txid};

use crate::{Query, RepresentationId, representation_id::content_hash};

use super::{ResolvedConfirmedTx, resolved::TransactionSource};

/// Raw transaction data resolved to one exact in-memory or indexed source.
pub struct ResolvedRawTransaction {
    source: RawTransactionSource,
}

enum RawTransactionSource {
    Memory { bytes: Vec<u8>, hash: u64 },
    Chain(ResolvedConfirmedTx),
}

impl ResolvedRawTransaction {
    fn memory(transaction: Transaction) -> Self {
        let bytes = transaction.encode_bytes();
        let hash = content_hash(&bytes);
        Self {
            source: RawTransactionSource::Memory { bytes, hash },
        }
    }

    pub fn identity(&self) -> RepresentationId {
        match &self.source {
            RawTransactionSource::Memory { hash, .. } => RepresentationId::Content(*hash),
            RawTransactionSource::Chain(transaction) => transaction.identity(),
        }
    }
}

impl Query {
    /// Resolve raw transaction data once before an async response handoff.
    pub fn resolve_raw_transaction(&self, txid: &Txid) -> Result<ResolvedRawTransaction> {
        Ok(match self.resolve_transaction_source(txid)? {
            TransactionSource::Memory(transaction) => ResolvedRawTransaction::memory(transaction),
            TransactionSource::Chain(transaction) => ResolvedRawTransaction {
                source: RawTransactionSource::Chain(transaction),
            },
        })
    }

    /// Read raw bytes without repeating the transaction-prefix lookup.
    pub fn transaction_raw_resolved(&self, transaction: ResolvedRawTransaction) -> Result<Vec<u8>> {
        match transaction.source {
            RawTransactionSource::Memory { bytes, .. } => Ok(bytes),
            RawTransactionSource::Chain(transaction) => {
                let (_, index, _) = self.revalidate_confirmed_tx(transaction)?;
                let bytes = self.transaction_raw_by_index(index)?;
                self.revalidate_confirmed_tx(transaction)?;
                Ok(bytes)
            }
        }
    }

    /// Hex-encode raw bytes without repeating transaction resolution.
    pub fn transaction_hex_resolved(&self, transaction: ResolvedRawTransaction) -> Result<String> {
        self.transaction_raw_resolved(transaction)
            .map(|bytes| bytes.to_lower_hex_string())
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::{
        Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness,
        absolute::LockTime, consensus, hashes::Hash, transaction::Version,
    };

    use super::*;

    #[test]
    fn content_identity_changes_with_witness_when_txid_does_not() {
        let mut transaction = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: bitcoin::Txid::from_byte_array([1; 32]),
                    vout: 0,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: Witness::from_slice(&[b"first"]),
            }],
            output: vec![TxOut {
                value: Amount::from_sat(1),
                script_pubkey: ScriptBuf::new(),
            }],
        };
        let first_txid = transaction.compute_txid();
        let first_hash = content_hash(&consensus::serialize(&transaction));

        transaction.input[0].witness = Witness::from_slice(&[b"second"]);
        assert_eq!(transaction.compute_txid(), first_txid);
        assert_ne!(
            content_hash(&consensus::serialize(&transaction)),
            first_hash
        );
    }
}
