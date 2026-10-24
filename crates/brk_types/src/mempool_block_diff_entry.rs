#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Transaction;

/// One stretch of the new block in a `MempoolBlockDiff`.
///
/// Untagged on the wire so JSON type disambiguates the variants:
/// - `Retained(run)` serializes as `[start, count]`: the prior block's transactions
///   `start..start + count` (which the client cached at `since`), kept in order.
/// - `New(tx)` serializes as a transaction object: a body that was new or changed since the
///   prior block and takes this position.
// `Tx` holds the new body: owned by default, a reference or a block position when serving.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(rename = "MempoolBlockDiffEntry"))]
#[serde(untagged)]
pub enum MempoolBlockDiffEntry<Tx = Transaction> {
    Retained(RetainedRun),
    New(Tx),
}

/// Consecutive transactions kept from the prior block, `[start, count]`: its positions
/// `start..start + count`, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct RetainedRun(pub u32, pub u32);
