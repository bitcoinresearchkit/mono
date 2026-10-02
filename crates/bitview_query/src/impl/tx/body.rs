use crate::RepresentationId;

use super::ResolvedConfirmedTx;

/// Frozen live bytes or a confirmed transaction to revalidate before loading.
/// Public response wrappers keep raw, JSON, and CPFP bodies distinct.
pub(crate) enum ResolvedTxBody {
    Memory { bytes: Vec<u8>, hash: u64 },
    Chain(ResolvedConfirmedTx),
}

impl ResolvedTxBody {
    pub fn memory(bytes: Vec<u8>) -> Self {
        let hash = RepresentationId::content_hash(&bytes);
        Self::Memory { bytes, hash }
    }

    pub fn identity(&self) -> RepresentationId {
        match self {
            Self::Memory { hash, .. } => RepresentationId::Content(*hash),
            Self::Chain(transaction) => transaction.identity(),
        }
    }
}
