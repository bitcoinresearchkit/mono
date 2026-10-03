#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Sats, Txid, Weight};

/// A transaction in a CPFP relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CpfpEntry {
    pub txid: Txid,
    pub weight: Weight,
    pub fee: Sats,
}
