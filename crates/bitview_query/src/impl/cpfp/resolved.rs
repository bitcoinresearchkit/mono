use brk_types::{Cpfp, Txid};
use serde_json::to_vec;

use crate::{Error, Query, RepresentationId, Result, r#impl::tx::body::ResolvedTxBody};

/// CPFP JSON resolved to one exact live or confirmed transaction source.
pub struct ResolvedCpfp {
    source: ResolvedTxBody<Vec<u8>>,
}

impl ResolvedCpfp {
    pub fn identity(&self) -> RepresentationId {
        self.source.identity()
    }

    pub fn is_settled(&self) -> bool {
        self.source.is_settled()
    }
}

impl Query {
    /// Resolve CPFP JSON once before an async response handoff. An unknown transaction answers
    /// `{"ancestors":[]}` as mempool.space does, live: it may yet arrive.
    pub fn resolve_cpfp(&self, txid: &Txid) -> Result<ResolvedCpfp> {
        let live = |value: &Cpfp| {
            let value = to_vec(value).unwrap();
            let hash = RepresentationId::content_hash(&value);
            (value, hash)
        };
        let source = match self.resolve_tx_body(txid, |mempool, tip| {
            Ok(mempool
                .cpfp_info(txid, tip)?
                .map(|info| live(&Cpfp::Mempool(info))))
        }) {
            Err(Error::UnknownTxid) => {
                let (value, hash) = live(&Cpfp::EMPTY);
                ResolvedTxBody::Memory { value, hash }
            }
            source => source?,
        };
        Ok(ResolvedCpfp { source })
    }

    /// Build JSON bytes without repeating transaction resolution.
    pub fn cpfp_json_resolved(&self, cpfp: ResolvedCpfp) -> Result<Vec<u8>> {
        match cpfp.source {
            ResolvedTxBody::Memory { value, .. } => Ok(value),
            ResolvedTxBody::Chain(transaction) => {
                let cpfp = self.confirmed_cpfp_resolved(transaction)?;
                Ok(to_vec(&cpfp).unwrap())
            }
        }
    }
}
