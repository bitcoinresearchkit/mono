#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{MempoolBlock, MempoolBlockHash, MempoolTxSummary};

/// A projected mempool block with its transactions in block order: block 0 is Bitcoin Core's
/// `getblocktemplate` selection, the later ones packed after it as Core's block assembler packs,
/// ranked by signed modified fees within Core's default budgets, with paid-fee statistics. Returned by
/// `GET /api/mempool-blocks/{index}`.
// `Txs` holds the transactions: owned by default, a view of the published bodies when serving.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(rename = "MempoolBlockContent"))]
#[serde(rename_all = "camelCase")]
pub struct MempoolBlockContent<Txs = Vec<MempoolTxSummary>> {
    /// Pass as `since` to `GET /api/mempool-blocks/{index}/diff/{since}` to follow changes.
    pub hash: MempoolBlockHash,

    pub stats: MempoolBlock,

    /// The transactions in block order (block 0's: `getblocktemplate`'s), each in its light
    /// shape: the whole body is at `GET /api/tx/{txid}`.
    pub transactions: Txs,
}
