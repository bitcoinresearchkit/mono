#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{BlockTemplateDiffEntry, MempoolBlock, NextBlockHash, Txid};

/// Delta between the current `getblocktemplate` projection and a prior
/// one identified by `since`. Returned by
/// `GET /api/v1/mempool/block-template/diff/{hash}`.
///
/// `removed` lists txids no longer present. A changed body can be emitted as
/// `New` without removing its txid; absence of a retained index alone does not
/// imply removal.
// `Order` holds the entries: owned by default, a view of the published bodies when serving.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(rename = "BlockTemplateDiff"))]
#[serde(rename_all = "camelCase")]
pub struct BlockTemplateDiff<Order = Vec<BlockTemplateDiffEntry>> {
    /// Current next-block hash. Use as `since` on the next diff call.
    pub hash: NextBlockHash,

    /// Echoed prior hash the diff was computed against.
    pub since: NextBlockHash,

    /// Stats of the new template; they can change while every transaction is retained.
    pub stats: MempoolBlock,

    /// New template in order. Each entry is either an index into the
    /// prior template's transactions or a full transaction body.
    pub order: Order,

    /// Txids that left the projected next block since `since`
    /// (confirmed, evicted, replaced, or pushed past block 0).
    pub removed: Vec<Txid>,
}
