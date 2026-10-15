use brk_types::Txid;
use serde_json::to_vec;

use super::CpfpSource;
use crate::{Query, RepresentationId, Result, r#impl::tx::body::ResolvedTxBody};

/// CPFP JSON resolved to one exact live or confirmed transaction source.
pub struct ResolvedCpfp {
    source: ResolvedTxBody<Vec<u8>>,
}

impl ResolvedCpfp {
    pub fn identity(&self) -> RepresentationId {
        self.source.identity()
    }
}

impl Query {
    /// Resolve CPFP JSON once before an async response handoff.
    pub fn resolve_cpfp(&self, txid: &Txid) -> Result<ResolvedCpfp> {
        let source = match self.resolve_cpfp_source(txid)? {
            CpfpSource::Memory(info) => {
                let value = to_vec(&info).unwrap();
                let hash = RepresentationId::content_hash(&value);
                ResolvedTxBody::Memory { value, hash }
            }
            CpfpSource::Chain(transaction) => ResolvedTxBody::Chain(transaction),
        };
        Ok(ResolvedCpfp { source })
    }

    /// Build JSON bytes without repeating transaction resolution.
    pub fn cpfp_json_resolved(&self, cpfp: ResolvedCpfp) -> Result<Vec<u8>> {
        match cpfp.source {
            ResolvedTxBody::Memory { value, .. } => Ok(value),
            ResolvedTxBody::Chain(transaction) => {
                let info = self.confirmed_cpfp_resolved(transaction)?;
                Ok(to_vec(&info).unwrap())
            }
        }
    }
}
