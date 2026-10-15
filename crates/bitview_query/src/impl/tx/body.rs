use crate::RepresentationId;

use super::ResolvedConfirmedTx;

/// A live value with its content hash, or a confirmed transaction to revalidate before loading.
/// Public response wrappers keep raw, JSON, and CPFP bodies distinct; a live value is encoded
/// only once a response needs its bytes.
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
}
