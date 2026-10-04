#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{BlockHash, Height, Timestamp};

/// Transaction confirmation status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct TxStatus {
    /// Whether the transaction is confirmed
    #[cfg_attr(feature = "schemars", schemars(example = true))]
    pub confirmed: bool,

    /// Block height (only present if confirmed)
    #[cfg_attr(feature = "schemars", schemars(example = Some(916656)))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_height: Option<Height>,

    /// Block hash (only present if confirmed)
    #[cfg_attr(feature = "schemars", schemars(example = Some("000000000000000000012711f7e0d13e586752a42c66e25faf75f159b3d04911".to_string())))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<BlockHash>,

    /// Block timestamp (only present if confirmed)
    #[cfg_attr(feature = "schemars", schemars(example = Some(1759000868)))]
    #[serde(skip_serializing_if = "Option::is_none")]
    block_time: Option<Timestamp>,
}

impl TxStatus {
    pub const UNCONFIRMED: Self = Self {
        confirmed: false,
        block_hash: None,
        block_height: None,
        block_time: None,
    };

    pub fn confirmed(height: Height, block_hash: BlockHash, block_time: Timestamp) -> Self {
        Self {
            confirmed: true,
            block_height: Some(height),
            block_hash: Some(block_hash),
            block_time: Some(block_time),
        }
    }
}
