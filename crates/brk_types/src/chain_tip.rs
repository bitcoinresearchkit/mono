#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One complete Bitcoin Core `getchaintips` record, under Core's field names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct ChainTip {
    pub height: u32,
    /// Conventional block-hash hex. Keep the RPC field's public Rust type.
    #[cfg_attr(feature = "schemars", schemars(with = "crate::BlockHash"))]
    pub hash: bitcoin::BlockHash,
    /// Zero for the active tip; otherwise the branch's length off the active chain.
    pub branchlen: u32,
    /// Core's status verbatim, including `unknown` and future values.
    pub status: String,
}
