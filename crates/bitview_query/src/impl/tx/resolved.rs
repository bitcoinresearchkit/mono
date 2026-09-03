use brk_error::{Error, Result};
use brk_types::{Transaction, Txid};

use crate::Query;

use super::ResolvedConfirmedTx;

pub(super) enum TransactionSource {
    Memory(Transaction),
    Chain(ResolvedConfirmedTx),
}

impl Query {
    /// Resolve one exact live, confirmed, or recently vanished transaction.
    pub(super) fn resolve_transaction_source(&self, txid: &Txid) -> Result<TransactionSource> {
        let mempool = self.mempool();
        if let Some(transaction) = mempool.and_then(|m| m.with_tx(txid, Transaction::clone)) {
            return Ok(TransactionSource::Memory(transaction));
        }

        match self.resolve_confirmed_tx(txid) {
            Ok(transaction) => Ok(TransactionSource::Chain(transaction)),
            Err(Error::UnknownTxid) => mempool
                .and_then(|m| m.with_vanished_tx(txid, Transaction::clone))
                .map(TransactionSource::Memory)
                .ok_or(Error::UnknownTxid),
            Err(error) => Err(error),
        }
    }
}
