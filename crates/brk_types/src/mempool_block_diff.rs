#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{MempoolBlock, MempoolBlockDiffEntry, MempoolBlockHash, Txid};

/// A projected mempool block's changes since a prior copy identified by `since`. Returned by
/// `GET /api/mempool-blocks/{index}/diff/{since}`.
///
/// `order` alone is the new block, and a `New` entry replaces what was known of its txid (a
/// transaction whose body changed comes back as `New`, not in `removed`). `removed` lists the
/// txids that left the block since `since`: evict what was cached for them. When the server
/// doesn't hold the copy at `since` (aged out, another server's, or from before a restart), the
/// diff is from no copy: `since` is `"0"`, every entry is `New` and `removed` is empty, so replace
/// the copy and drop what was cached for txids not in `order`.
// `Order` holds the entries: owned by default, a view of the published bodies when serving.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(rename = "MempoolBlockDiff"))]
#[serde(rename_all = "camelCase")]
pub struct MempoolBlockDiff<Order = Vec<MempoolBlockDiffEntry>> {
    /// The block's current hash. Use as `since` on the next diff call.
    pub hash: MempoolBlockHash,

    /// The prior hash the diff was computed against: the `since` asked for, or `"0"` when the
    /// server doesn't hold that copy (then `order` replaces it).
    pub since: MempoolBlockHash,

    /// Stats of the new block; they can change while every transaction is retained.
    pub stats: MempoolBlock,

    /// The new block in order. Each entry is either a run of the prior copy's transactions,
    /// `[start, count]`, or a transaction in its light shape.
    pub order: Order,

    /// Txids that left the projected block since `since` (confirmed, evicted, replaced, or moved
    /// to another block).
    pub removed: Vec<Txid>,
}
