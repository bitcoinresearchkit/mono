#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Transaction;

/// One slot of the new template in a `BlockTemplateDiff`.
///
/// Untagged on the wire so JSON type disambiguates the variants:
/// - `Retained(idx)` serializes as a bare integer - index into the
///   transactions of the prior template (which the client cached at
///   `since`).
/// - `New(tx)` serializes as a transaction object - a body that was
///   new or changed since the prior template and must replace this position.
// `Tx` holds the new body: owned by default, a reference or a template position when serving.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(rename = "BlockTemplateDiffEntry"))]
#[serde(untagged)]
pub enum BlockTemplateDiffEntry<Tx = Transaction> {
    Retained(u32),
    New(Tx),
}
