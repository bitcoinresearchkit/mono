#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{MempoolBlock, MempoolBlockDiffEntry, MempoolBlockHash, Txid};

/// A projected mempool block's changes since a prior copy identified by `since`. Returned by
/// `GET /api/mempool-blocks/{index}/diff/{since}`.
///
/// `removed` lists txids no longer present. A changed body can be emitted as `New` without
/// removing its txid; absence from every retained run alone does not imply removal.
// `Order` holds the entries: owned by default, a view of the published bodies when serving.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(rename = "MempoolBlockDiff"))]
#[serde(rename_all = "camelCase")]
pub struct MempoolBlockDiff<Order = Vec<MempoolBlockDiffEntry>> {
    /// The block's current hash. Use as `since` on the next diff call.
    pub hash: MempoolBlockHash,

    /// Echoed prior hash the diff was computed against.
    pub since: MempoolBlockHash,

    /// Stats of the new block; they can change while every transaction is retained.
    pub stats: MempoolBlock,

    /// The new block in order. Each entry is either a run of the prior copy's transactions,
    /// `[start, count]`, or a full transaction body.
    pub order: Order,

    /// Txids that left the projected next block since `since`
    /// (confirmed, evicted, replaced, or pushed past block 0).
    pub removed: Vec<Txid>,
}
