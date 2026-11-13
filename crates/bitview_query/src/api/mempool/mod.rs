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

/// The most blocks a pending view corrects a pool behind the pin across, about an hour's: the pool
/// normally trails the chain by a cycle (a block), and one further behind has stalled and tells
/// nothing of what's pending now. It bounds the view's build too: every input of those blocks.
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
    /// The array's validator: the pool's membership revision, unique to this process.
    pub fn hash(&self) -> u64 {
        self.hash
    }

    /// Encoded once per publication, shared.
    pub fn json(&self) -> Arc<[u8]> {
        self.mempool.txids_json().expect("published when resolved")
    }
}

impl Query {
    /// Runs `read` on the pool as `pin` sees it ([`Self::pending_view`]). With no view to read,
    /// answers `T::default()`, which is nothing pending. Call this only after the chain alone
    /// hasn't answered.
    pub(crate) fn with_pending<T: Default>(
        &self,
        mempool: Option<&ReadOnlyState>,
        pin: &ReadPin,
        read: impl FnOnce(&ReadOnlyState, &Arc<PendingView>) -> brk_error::Result<T>,
    ) -> Result<T> {
        Ok(match self.pending_view(mempool, pin)? {
            Some((mempool, view)) => read(mempool, &view)?,
            None => T::default(),
        })
    }

    /// The pool as `pin` sees it, with the publication the view reads.
    ///
    /// The pool usually sits at the pin's tip. It trails for a moment after each block, and after
    /// a reorg it can sit on the branch the chain left, or ahead of the pin. Past its own tip the
    /// view hides what the pin's blocks settled, counted from the newest block both chains hold.
    /// See [`PendingView`].
    ///
    /// - `Ok(None)`: nothing pending can be told. Either there is no mempool, or the pool is more
    ///   than [`MAX_POOL_LAG`] blocks behind and has stalled.
    /// - `StateUpdating`: the pool hasn't published yet, or none of the blocks it remembers is on
    ///   the pin's chain (a reorg deeper than its memory).
    ///
    /// A view past the tip is built once per publication and pin tip, and shared.
    pub(crate) fn pending_view<'a>(
        &self,
        mempool: Option<&'a ReadOnlyState>,
        pin: &ReadPin,
    ) -> Result<Option<(&'a ReadOnlyState, Arc<PendingView>)>> {
        let Some(mempool) = mempool else {
            return Ok(None);
        };
        let Some(pool_tip) = mempool.tip() else {
            return Err(Error::updating());
        };
        let tip = pin.tip();
        if pool_tip == tip {
            return Ok(Some((mempool, mempool.view_at(&tip)?)));
        }
        // Walk the pool's blocks newest first, to the first the pin's chain holds. `left` counts
        // those passed. A block the chain lacks is unknown to it or past a lowered publication.
        let mut common = None;
        for (left, hash) in mempool.ancestry().rev().enumerate() {
            match self.height_by_hash_at(hash, pin) {
                Ok(height) => {
                    common = Some((height, left));
                    break;
                }
                Err(Error::NotFound(_)) => {}
                Err(Error::StateUpdating(_)) if pin.is_lowered() => {}
                Err(error) => return Err(error),
            }
        }
        let Some((height, left)) = common else {
            return Err(Error::updating());
        };
        let lengths = pin.lengths();
        if usize::from(lengths.height) - usize::from(height) > MAX_POOL_LAG + 1 {
            return Ok(None);
        }
        // Every input of the pin's blocks past the common one, as the outpoint and its spender.
        // Only the first reader of this publication and tip runs this.
        let spends = || -> Result<Vec<_>> {
            let vecs = self.indexer().vecs();
            let after = height.incremented();
            let first = if after < lengths.height {
                vecs.inputs.first_txin_index.collect_one(after).data()?
            } else {
                lengths.txin_index
            };
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
        Ok(Some((mempool, mempool.view_behind(tip, left, spends)?)))
    }

    /// Whether the pool may answer a transaction lookup the chain answered with `error`.
    ///
    /// True when the chain doesn't hold the transaction, or when its row sits past a lowered
    /// publication (a rollback is rewriting it). Either way it isn't on the pin's chain, so a
    /// transaction pending in the view is pending for this reader.
    pub(crate) fn pool_answers(error: &Error, pin: &ReadPin) -> bool {
        match error {
            Error::UnknownTxid => true,
            Error::StateUpdating(_) => pin.is_lowered(),
            _ => false,
        }
    }

    /// The error for a lookup neither the chain nor the pool answered: the chain's `error`, or
    /// `StateUpdating` for a transaction in limbo ([`PendingView::in_limbo`]).
    pub(crate) fn unanswered(error: Error, view: &PendingView, txid: &Txid) -> Error {
        if view.in_limbo(txid) {
            Error::updating()
        } else {
            error
        }
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

    /// The recently added transactions with their exact content identity, built once per
    /// publication.
    pub fn mempool_recent_json(&self) -> Result<(Arc<[u8]>, RepresentationId)> {
        published_json(self.require_mempool()?.recent_json()?)
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
