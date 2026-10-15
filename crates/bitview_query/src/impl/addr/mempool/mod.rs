use brk_mempool::PublishedTx;
use brk_types::{Address, BlockHash};
use serde::Serializer;

use crate::{
    Error, Query, RepresentationId, Result,
    r#impl::{addr::parse_addr, mempool::anchored},
};

/// One address's mempool page, captured from one publication.
pub struct ResolvedAddrMempoolTxs(Vec<PublishedTx>);

impl ResolvedAddrMempoolTxs {
    pub fn identity(&self) -> RepresentationId {
        page_identity(None, &self.0)
    }

    pub fn json(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        serde_json::Serializer::new(&mut bytes)
            .collect_seq(self.0.iter().map(|published| published.tx.as_ref()))
            .expect("transactions serialize");
        bytes
    }
}

/// The identity of a page of live bodies after the confirmed page anchored at `anchor`: their
/// digests, without encoding the bodies.
pub(crate) fn page_identity(anchor: Option<BlockHash>, txs: &[PublishedTx]) -> RepresentationId {
    let mut bytes = Vec::with_capacity(33 + txs.len() * 8);
    match anchor {
        Some(anchor) => {
            bytes.push(1);
            bytes.extend_from_slice(anchor.as_slice());
        }
        None => bytes.push(0),
    }
    for published in txs {
        bytes.extend_from_slice(&published.digest.to_le_bytes());
    }
    RepresentationId::content(&bytes)
}

impl Query {
    /// Capture shared transaction bodies only when mempool and indexed chain
    /// refer to the same completed publication.
    pub fn resolve_addr_mempool_txs(
        &self,
        addr: &Address,
        limit: usize,
    ) -> Result<ResolvedAddrMempoolTxs> {
        let bytes = parse_addr(addr)?;
        let mempool = self.mempool().ok_or(Error::MempoolNotAvailable)?;
        let pin = self.pin_safe_lengths()?;
        Ok(ResolvedAddrMempoolTxs(anchored(mempool.addr_txs(
            &bytes,
            limit,
            &pin.tip(),
        ))?))
    }
}
