use std::sync::Arc;

use brk_mempool::{JsonBody, PendingView, ReadOnlyState};
pub use brk_mempool::{MempoolBlockSource, ResolvedMempoolBlockDiff};
use brk_types::{MempoolBlockHash, MempoolBlockIndex, OutPoint, TxIndex, Txid};
use serde::{Serialize, Serializer};
use serde_json::to_vec;
use vecdb::ReadableVec;

use crate::{Error, OptionData, Query, ReadPin, RepresentationId, Result};

pub mod rbf;

pub use rbf::ResolvedRbf;

/// The most blocks a pending view corrects a pool behind the pin across: one further behind has
/// stalled.
const MAX_POOL_LAG: usize = 6;

/// A lookup of the pool at its own tip, or none before its first publication: the confirmed data
/// answers alone meanwhile.
pub(crate) fn anchored<T: Default>(view: brk_error::Result<T>) -> Result<T> {
    match view {
        Err(brk_error::Error::StateUpdating) => Ok(T::default()),
        view => Ok(view?),
    }
}

/// A body the pool built once, and its content hash as its identity.
fn published_json((bytes, hash): JsonBody) -> Result<(Vec<u8>, RepresentationId)> {
    Ok((bytes.to_vec(), RepresentationId::Content(hash)))
}

fn serialize_json<T: Serialize>(value: &T) -> (Vec<u8>, RepresentationId) {
    let bytes = to_vec(value).unwrap();
    let identity = RepresentationId::content(&bytes);
    (bytes, identity)
}

/// One publication's txid array, serialized only once a response needs it.
pub struct MempoolTxids {
    mempool: Arc<ReadOnlyState>,
    hash: u64,
}

impl MempoolTxids {
    /// The array's order-sensitive validator.
    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn json(&self) -> Vec<u8> {
        let txids = self.mempool.txids().expect("published when resolved");
        let mut bytes = Vec::new();
        serde_json::Serializer::new(&mut bytes)
            .collect_seq(txids)
            .expect("txids serialize");
        bytes
    }
}

impl Query {
    /// `read` of the pool as `pin` sees it ([`PendingView`]): at the pin's tip, or a few blocks
    /// behind it (the moment after blocks publish, before the pool follows) without what they
    /// settled. Nothing pending (the confirmed data answers alone) with no mempool, before its
    /// first publication, or with one stalled or on a branch the chain left.
    pub(crate) fn with_pending<T: Default>(
        &self,
        mempool: Option<&ReadOnlyState>,
        pin: &ReadPin,
        read: impl FnOnce(&ReadOnlyState, &PendingView) -> brk_error::Result<T>,
    ) -> Result<T> {
        let Some(mempool) = mempool else {
            return Ok(T::default());
        };
        Ok(match self.pending_view(mempool, pin)? {
            Some(view) => read(mempool, &view)?,
            None => T::default(),
        })
    }

    /// The pool as `pin` sees it, if it can: see [`Self::with_pending`].
    pub(crate) fn pending_view(
        &self,
        mempool: &ReadOnlyState,
        pin: &ReadPin,
    ) -> Result<Option<PendingView>> {
        let Some(pool_tip) = mempool.tip() else {
            return Ok(None);
        };
        let tip = pin.tip();
        if pool_tip == tip {
            return Ok(Some(mempool.view_at(&tip)?));
        }
        let height = match self.height_by_hash_at(&pool_tip, pin) {
            Ok(height) => height,
            Err(Error::NotFound(_)) => return Ok(None),
            Err(error) => return Err(error),
        };
        let lengths = pin.lengths();
        if usize::from(lengths.height) - usize::from(height) > MAX_POOL_LAG + 1 {
            return Ok(None);
        }
        // Every input of the blocks past the pool's tip: the outpoint it spends, and its spender.
        let vecs = self.indexer().vecs();
        let first = vecs
            .inputs
            .first_txin_index
            .collect_one(height.incremented())
            .data()?;
        let mut outpoint = vecs.inputs.outpoint.cursor();
        let mut spender = vecs.inputs.tx_index.cursor();
        let txid = vecs.transactions.txid.reader();
        let inputs = usize::from(first)..usize::from(lengths.txin_index);
        let mut spends = Vec::with_capacity(inputs.len());
        for input in inputs {
            let spent: OutPoint = outpoint.get(input).data()?;
            if spent.is_coinbase() {
                continue;
            }
            let tx: TxIndex = spender.get(input).data()?;
            spends.push(((txid.get(spent.tx_index()), spent.vout()), txid.get(tx)));
        }
        Ok(Some(mempool.view_behind(spends)?))
    }

    fn require_mempool(&self) -> Result<Arc<ReadOnlyState>> {
        self.mempool().ok_or(Error::MempoolNotAvailable)
    }

    /// Mempool statistics with their exact content identity, built once per publication.
    pub fn mempool_info_json(&self) -> Result<(Vec<u8>, RepresentationId)> {
        published_json(self.require_mempool()?.info_json()?)
    }

    /// The pool's txids from one publication: their validator now, the array on demand.
    pub fn mempool_txids(&self) -> Result<MempoolTxids> {
        let mempool = self.require_mempool()?;
        let hash = mempool.txids_hash()?;
        Ok(MempoolTxids { mempool, hash })
    }

    /// mempool.space's `/api/v1/fees/recommended` (whole sat/vB) with its content identity, built
    /// once per projection.
    pub fn recommended_fees_json(&self) -> Result<(Vec<u8>, RepresentationId)> {
        published_json(self.require_mempool()?.fees_json()?)
    }

    /// mempool.space's `/api/v1/fees/precise`, as [`Self::recommended_fees_json`].
    pub fn precise_fees_json(&self) -> Result<(Vec<u8>, RepresentationId)> {
        published_json(self.require_mempool()?.precise_fees_json()?)
    }

    /// mempool.space's `/api/v1/fees/mempool-blocks`, as [`Self::recommended_fees_json`].
    pub fn mempool_blocks_json(&self) -> Result<(Vec<u8>, RepresentationId)> {
        published_json(self.require_mempool()?.mempool_blocks_json()?)
    }

    /// Serialize recent transactions with their exact content identity.
    pub fn mempool_recent_json(&self) -> Result<(Vec<u8>, RepresentationId)> {
        let recent = self.require_mempool()?.recent_txs()?;
        Ok(serialize_json(&recent))
    }

    /// Transaction times and an exact, order-sensitive result hash from one
    /// mempool state snapshot.
    pub fn transaction_times_with_hash(&self, txids: &[Txid]) -> Result<(Vec<u64>, u64)> {
        Ok(self.require_mempool()?.transaction_times_with_hash(txids)?)
    }

    /// Capture a published block for cheap validation and deferred construction.
    pub fn resolve_mempool_block(&self, index: MempoolBlockIndex) -> Result<MempoolBlockSource> {
        served_block(&*self.require_mempool()?, index)
    }

    /// Validate history before conditional handling and capture both diff inputs.
    pub fn resolve_mempool_block_diff(
        &self,
        index: MempoolBlockIndex,
        since: MempoolBlockHash,
    ) -> Result<ResolvedMempoolBlockDiff> {
        let mempool = self.require_mempool()?;
        let source = served_block(&mempool, index)?;
        mempool
            .resolve_mempool_block_diff(source, since)
            .ok_or_else(|| Error::NotFound(format!("unknown since hash: {since}")))
    }
}

/// Block `index` of the published projection, or why it isn't served.
fn served_block(mempool: &ReadOnlyState, index: MempoolBlockIndex) -> Result<MempoolBlockSource> {
    if let Some(source) = mempool.mempool_block_source(index.get() as usize)? {
        return Ok(source);
    }
    let projected = mempool.projected_blocks()?;
    Err(Error::NotFound(if (index.get() as usize) < projected {
        format!(
            "Mempool block {index} holds what's left past block {}, more than a block: only its \
             stats are served, by GET /api/v1/fees/mempool-blocks",
            index.get() - 1
        )
    } else {
        format!(
            "Mempool block {index} isn't projected: blocks 0 to {} are",
            projected - 1
        )
    }))
}
