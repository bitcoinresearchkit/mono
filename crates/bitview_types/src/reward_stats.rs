use brk_types::{Height, Sats};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, Serializer};

/// Block reward statistics over a range of blocks
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RewardStats {
    /// First block in the range
    pub start_block: Height,
    /// Last block in the range
    pub end_block: Height,
    /// Total coinbase rewards (subsidy + fees) in sats
    #[serde(serialize_with = "sats_as_string")]
    #[schemars(with = "String")]
    pub total_reward: Sats,
    /// Total transaction fees in sats
    #[serde(serialize_with = "sats_as_string")]
    #[schemars(with = "String")]
    pub total_fee: Sats,
    /// Total number of transactions
    #[serde(serialize_with = "u64_as_string")]
    #[schemars(with = "String")]
    pub total_tx: u64,
}

fn sats_as_string<S>(value: &Sats, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

fn u64_as_string<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}
