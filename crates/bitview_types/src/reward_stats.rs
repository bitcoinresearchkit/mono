use std::fmt::Display;

use brk_types::{Height, Sats};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

/// Block reward statistics over a range of blocks
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RewardStats {
    /// First block in the range
    pub start_block: Height,
    /// Last block in the range
    pub end_block: Height,
    /// Total coinbase rewards (subsidy + fees) in sats
    #[serde(with = "decimal_string")]
    #[schemars(with = "String")]
    pub total_reward: Sats,
    /// Total transaction fees in sats
    #[serde(with = "decimal_string")]
    #[schemars(with = "String")]
    pub total_fee: Sats,
    #[serde(with = "decimal_string")]
    #[schemars(with = "String")]
    pub total_tx: u64,
}

/// An unsigned integer written as a decimal string.
mod decimal_string {
    use super::*;

    pub fn serialize<T: Display, S: Serializer>(
        value: &T,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T: From<u64>, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<T, D::Error> {
        String::deserialize(deserializer)?
            .parse::<u64>()
            .map(T::from)
            .map_err(D::Error::custom)
    }
}
