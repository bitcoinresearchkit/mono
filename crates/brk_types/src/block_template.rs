#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{MempoolBlock, NextBlockHash, Transaction};

/// Projected next-block contents from Bitcoin Core's `getblocktemplate`
/// (block 0 of the snapshot). Returned by
/// `GET /api/v1/mempool/block-template`.
// `Txs` holds the bodies: owned by default, a view of the published ones when serving.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(rename = "BlockTemplate"))]
#[serde(rename_all = "camelCase")]
pub struct BlockTemplate<Txs = Vec<Transaction>> {
    /// Pass to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas.
    pub hash: NextBlockHash,

    pub stats: MempoolBlock,

    /// Full transaction bodies in `getblocktemplate` order.
    pub transactions: Txs,
}
