use brk_mempool::ReadOnlyState;
use brk_types::{BlockHash, Txid};

use super::ResolvedConfirmedTx;
use crate::{Error, Query, RepresentationId, Result};

/// A mempool value with its content hash, or a confirmed transaction to revalidate before
/// loading. Transaction values are encoded on demand; CPFP values are already JSON.
pub(crate) enum ResolvedTxBody<T> {
    Memory { value: T, hash: u64 },
    Chain(ResolvedConfirmedTx),
}

impl<T> ResolvedTxBody<T> {
    pub fn identity(&self) -> RepresentationId {
        match self {
            Self::Memory { hash, .. } => RepresentationId::Content(*hash),
            Self::Chain(transaction) => transaction.identity(),
        }
    }

    pub fn is_settled(&self) -> bool {
        match self {
            Self::Memory { .. } => false,
            Self::Chain(transaction) => transaction.is_settled(),
        }
    }
}

impl Query {
    /// Resolve `txid` to its confirmed transaction or, through `live`, to the mempool's value
    /// and its content hash: the chain answers first, the pool only for what is still pending.
    pub(crate) fn resolve_tx_body<T>(
        &self,
        txid: &Txid,
        live: impl FnOnce(&ReadOnlyState, &BlockHash) -> brk_error::Result<Option<(T, u64)>>,
    ) -> Result<ResolvedTxBody<T>> {
        let (mempool, pin) = self.mempool_then_pin()?;
        let read = self.read_indexer_at(pin);
        match read.resolve_confirmed_tx(txid) {
            Ok(transaction) => Ok(ResolvedTxBody::Chain(transaction)),
            Err(Error::UnknownTxid) => {
                let (mempool, tip) = Self::lookup_mempool(mempool).ok_or(Error::UnknownTxid)?;
                let (value, hash) = live(&mempool, &tip)?.ok_or(Error::UnknownTxid)?;
                Ok(ResolvedTxBody::Memory { value, hash })
            }
            Err(error) => Err(error),
        }
    }
}
