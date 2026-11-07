use std::sync::Arc;

use brk_mempool::{JsonBody, PendingView, ReadOnlyState};
pub use brk_mempool::{MempoolBlockSource, ResolvedMempoolBlockDiff};
use brk_types::{MempoolBlockHash, MempoolBlockIndex, OutPoint, TxIndex, Txid};
use serde::Serialize;
use serde_json::to_vec;
use vecdb::ReadableVec;

use crate::{Error, OptionData, Query, ReadPin, RepresentationId, Result};

pub mod rbf;

pub use rbf::ResolvedRbf;

/// The most blocks a pending view corrects a pool behind the pin across: one further behind has
/// stalled.
const MAX_POOL_LAG: usize = 6;

/// A body the pool built once, shared, and its content hash as its identity.
fn published_json((bytes, hash): JsonBody) -> Result<(Arc<[u8]>, RepresentationId)> {
    Ok((bytes, RepresentationId::Content(hash)))
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

    /// Encoded once per publication, shared.
    pub fn json(&self) -> Arc<[u8]> {
        self.mempool.txids_json().expect("published when resolved")
    }
}

impl Query {
    /// `read` of the pool as `pin` sees it ([`PendingView`]): at the pin's tip, or a few blocks
    /// behind it (the moment after blocks publish, before the pool follows) without what they
    /// settled. Nothing pending (the confirmed data answers alone) with no mempool or before its
    /// first publication; `StateUpdating` while it can't be seen from the pin (see
    /// [`Self::pending_view`]), never a confident answer without it.
    pub(crate) fn with_pending<T: Default>(
        &self,
        mempool: Option<&ReadOnlyState>,
        pin: &ReadPin,
        read: impl FnOnce(&ReadOnlyState, &Arc<PendingView>) -> brk_error::Result<T>,
    ) -> Result<T> {
        let Some(mempool) = mempool else {
            return Ok(T::default());
        };
        Ok(match self.pending_view(mempool, pin)? {
            Some(view) => read(mempool, &view)?,
            None => T::default(),
        })
    }

    /// `mempool` with its view as `pin` sees it, if it has one ([`Self::with_pending`]).
    pub(crate) fn pending(
        &self,
        mempool: Option<Arc<ReadOnlyState>>,
        pin: &ReadPin,
    ) -> Result<Option<(Arc<ReadOnlyState>, Arc<PendingView>)>> {
        let Some(mempool) = mempool else {
            return Ok(None);
        };
        Ok(self.pending_view(&mempool, pin)?.map(|view| (mempool, view)))
    }

    /// The pool as `pin` sees it: see [`Self::with_pending`]. None before its first publication.
    /// `StateUpdating` with a pool the pin can't see: on a branch the chain left (a reorg, until
    /// the pool follows), ahead of an older pin, or stalled more than [`MAX_POOL_LAG`] blocks
    /// behind. Behind the pin, made once per publication and pin tip, every read of that moment
    /// sharing it.
    pub(crate) fn pending_view(
        &self,
        mempool: &ReadOnlyState,
        pin: &ReadPin,
    ) -> Result<Option<Arc<PendingView>>> {
        let Some(pool_tip) = mempool.tip() else {
            return Ok(None);
        };
        let tip = pin.tip();
        if pool_tip == tip {
            return Ok(Some(mempool.view_at(&tip)?));
        }
        let height = match self.height_by_hash_at(&pool_tip, pin) {
            Ok(height) => height,
            Err(Error::NotFound(_)) => return Err(Error::StateUpdating),
            Err(error) => return Err(error),
        };
        let lengths = pin.lengths();
        if usize::from(lengths.height) - usize::from(height) > MAX_POOL_LAG + 1 {
            return Err(Error::StateUpdating);
        }
        // Every input of the blocks past the pool's tip: the outpoint it spends, and its spender
        // (read once for the moment, by the first read of it).
        let spends = || -> Result<Vec<_>> {
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
            Ok(spends)
        };
        Ok(Some(mempool.view_behind(tip, spends)?))
    }

    fn require_mempool(&self) -> Result<Arc<ReadOnlyState>> {
        self.mempool().ok_or(Error::MempoolNotAvailable)
    }

    /// Mempool statistics with their exact content identity, built once per publication.
    pub fn mempool_info_json(&self) -> Result<(Arc<[u8]>, RepresentationId)> {
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
    pub fn recommended_fees_json(&self) -> Result<(Arc<[u8]>, RepresentationId)> {
        published_json(self.require_mempool()?.fees_json()?)
    }

    /// mempool.space's `/api/v1/fees/precise`, as [`Self::recommended_fees_json`].
    pub fn precise_fees_json(&self) -> Result<(Arc<[u8]>, RepresentationId)> {
        published_json(self.require_mempool()?.precise_fees_json()?)
    }

    /// mempool.space's `/api/v1/fees/mempool-blocks`, as [`Self::recommended_fees_json`].
    pub fn mempool_blocks_json(&self) -> Result<(Arc<[u8]>, RepresentationId)> {
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
