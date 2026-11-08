use brk_mempool::{PendingView, ReadOnlyState};
use brk_types::Txid;

use super::ResolvedConfirmedTx;
use crate::{Query, RepresentationId, Result};

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
    /// and its content hash: the chain answers first, the pool for what is still pending as the
    /// pin sees it, when the chain doesn't hold it or is rewriting past it ([`Self::pool_answers`]):
    /// `StateUpdating` before the pool's first publication, and for what a reorg the pool hasn't
    /// followed yet left in limbo ([`Self::unanswered`]).
    pub(crate) fn resolve_tx_body<T>(
        &self,
        txid: &Txid,
        live: impl FnOnce(&ReadOnlyState, &PendingView) -> brk_error::Result<Option<(T, u64)>>,
    ) -> Result<ResolvedTxBody<T>> {
        let (mempool, pin) = self.mempool_then_pin()?;
        let read = self.read_indexer_at(pin);
        match read.resolve_confirmed_tx(txid) {
            Ok(transaction) => Ok(ResolvedTxBody::Chain(transaction)),
            Err(error) if Self::pool_answers(&error, read.pin()) => {
                let Some(mempool) = mempool else {
                    return Err(error);
                };
                let Some(view) = self.pending_view(&mempool, read.pin())? else {
                    return Err(error);
                };
                match live(&mempool, &view)? {
                    Some((value, hash)) => Ok(ResolvedTxBody::Memory { value, hash }),
                    None => Err(Self::unanswered(error, &view, txid)),
                }
            }
            Err(error) => Err(error),
        }
    }
}
