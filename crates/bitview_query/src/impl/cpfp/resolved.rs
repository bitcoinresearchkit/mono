use brk_error::Result;
use brk_types::{CpfpInfo, Txid};

use crate::{Query, RepresentationId, representation_id::content_hash};

use crate::r#impl::tx::ResolvedConfirmedTx;

/// CPFP JSON resolved to one exact live or confirmed transaction source.
pub struct ResolvedCpfp {
    source: ResolvedCpfpSource,
}

enum ResolvedCpfpSource {
    Memory { bytes: Vec<u8>, hash: u64 },
    Chain(ResolvedConfirmedTx),
}

pub(super) enum CpfpSource {
    Memory(CpfpInfo),
    Chain(ResolvedConfirmedTx),
}

impl ResolvedCpfp {
    fn memory(info: CpfpInfo) -> Self {
        let bytes = serde_json::to_vec(&info).unwrap();
        let hash = content_hash(&bytes);
        Self {
            source: ResolvedCpfpSource::Memory { bytes, hash },
        }
    }

    pub fn identity(&self) -> RepresentationId {
        match &self.source {
            ResolvedCpfpSource::Memory { hash, .. } => RepresentationId::Content(*hash),
            ResolvedCpfpSource::Chain(transaction) => transaction.identity(),
        }
    }
}

impl Query {
    pub(super) fn resolve_cpfp_source(&self, txid: &Txid) -> Result<CpfpSource> {
        if let Some(info) = self.mempool().and_then(|m| m.cpfp_info(txid)) {
            return Ok(CpfpSource::Memory(info));
        }
        self.resolve_confirmed_tx(txid).map(CpfpSource::Chain)
    }

    /// Resolve CPFP JSON once before an async response handoff.
    pub fn resolve_cpfp(&self, txid: &Txid) -> Result<ResolvedCpfp> {
        Ok(match self.resolve_cpfp_source(txid)? {
            CpfpSource::Memory(info) => ResolvedCpfp::memory(info),
            CpfpSource::Chain(transaction) => ResolvedCpfp {
                source: ResolvedCpfpSource::Chain(transaction),
            },
        })
    }

    /// Build JSON bytes without repeating transaction resolution.
    pub fn cpfp_json_resolved(&self, cpfp: ResolvedCpfp) -> Result<Vec<u8>> {
        match cpfp.source {
            ResolvedCpfpSource::Memory { bytes, .. } => Ok(bytes),
            ResolvedCpfpSource::Chain(transaction) => {
                let info = self.confirmed_cpfp_resolved(transaction)?;
                Ok(serde_json::to_vec(&info).unwrap())
            }
        }
    }
}
