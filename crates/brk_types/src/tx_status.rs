use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{BlockHash, Height, Timestamp};

/// Transaction confirmation status
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TxStatus {
    /// Whether the transaction is confirmed
    #[schemars(example = true)]
    pub confirmed: bool,

    /// Block height (only present if confirmed)
    #[schemars(example = Some(916656))]
    pub block_height: Option<Height>,

    /// Block hash (only present if confirmed)
    #[schemars(example = Some("000000000000000000012711f7e0d13e586752a42c66e25faf75f159b3d04911".to_string()))]
    pub block_hash: Option<BlockHash>,

    /// Block timestamp (only present if confirmed)
    #[schemars(example = Some(1759000868))]
    pub block_time: Option<Timestamp>,
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

    pub fn is_deeply_confirmed(&self, current_height: Height) -> bool {
        self.confirmed
            && self
                .block_height
                .is_some_and(|height| height.is_deeply_confirmed(current_height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deeply_confirmed_requires_more_than_six_blocks() {
        let mut status = TxStatus {
            confirmed: true,
            block_height: Some(Height::new(100)),
            block_hash: None,
            block_time: None,
        };

        assert!(!status.is_deeply_confirmed(Height::new(106)));
        assert!(status.is_deeply_confirmed(Height::new(107)));

        status.confirmed = false;
        assert!(!status.is_deeply_confirmed(Height::new(107)));
    }
}
