use brk_types::Txid;
use serde_json::to_vec;

use super::CpfpSource;
use crate::{Query, RepresentationId, Result, r#impl::tx::body::ResolvedTxBody};

/// CPFP JSON resolved to one exact live or confirmed transaction source.
pub struct ResolvedCpfp {
    source: ResolvedTxBody,
}

impl ResolvedCpfp {
    pub fn identity(&self) -> RepresentationId {
        self.source.identity()
    }
}

impl Query {
    /// Resolve CPFP JSON once before an async response handoff.
    pub fn resolve_cpfp(&self, txid: &Txid) -> Result<ResolvedCpfp> {
        let source = match self
            .resolve_cpfp_source(txid)
            .map_err(|error| self.transaction_error(error))?
        {
            CpfpSource::Memory(info) => ResolvedTxBody::memory(to_vec(&info).unwrap()),
            CpfpSource::Chain(transaction) => ResolvedTxBody::Chain(transaction),
        };
        Ok(ResolvedCpfp { source })
    }

    /// Build JSON bytes without repeating transaction resolution.
    pub fn cpfp_json_resolved(&self, cpfp: ResolvedCpfp) -> Result<Vec<u8>> {
        match cpfp.source {
            ResolvedTxBody::Memory { bytes, .. } => Ok(bytes),
            ResolvedTxBody::Chain(transaction) => {
                let info = self.confirmed_cpfp_resolved(transaction)?;
                Ok(to_vec(&info).unwrap())
            }
        }
    }
}
