use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Height, Sats};

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
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

fn u64_as_string<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_encoded_totals_have_string_schemas() {
        let stats = RewardStats {
            start_block: Height::new(1),
            end_block: Height::new(2),
            total_reward: Sats::new(3),
            total_fee: Sats::new(4),
            total_tx: 5,
        };
        let value = serde_json::to_value(stats).unwrap();
        assert_eq!(value["totalReward"], "3");
        assert_eq!(value["totalFee"], "4");
        assert_eq!(value["totalTx"], "5");

        let schema = serde_json::to_value(schemars::schema_for!(RewardStats)).unwrap();
        for field in ["totalReward", "totalFee", "totalTx"] {
            assert_eq!(schema["properties"][field]["type"], "string");
        }
    }
}
