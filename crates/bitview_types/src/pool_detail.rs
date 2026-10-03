use std::borrow::Cow;

use brk_types::{Pool, PoolSlug, Sats};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Detailed pool information with statistics across time periods
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PoolDetail {
    /// Pool information
    pub pool: PoolDetailInfo,

    /// Block counts for different time periods
    #[serde(rename = "blockCount")]
    pub block_count: PoolBlockCounts,

    /// Pool's share of total blocks for different time periods
    #[serde(rename = "blockShare")]
    pub block_share: PoolBlockShares,

    /// Estimated hashrate based on blocks mined (H/s)
    #[serde(rename = "estimatedHashrate")]
    #[schemars(example = 200_000_000_000_000_000_000_u128)]
    pub estimated_hashrate: u128,

    /// Self-reported hashrate (if available, H/s)
    #[serde(rename = "reportedHashrate")]
    pub reported_hashrate: Option<u128>,

    /// Total reward earned by this pool (sats, all time; None for minor pools)
    #[serde(rename = "totalReward")]
    pub total_reward: Option<Sats>,
}

/// Pool information for detail view
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PoolDetailInfo {
    /// Pool identifier
    #[schemars(example = 111)]
    id: u8,

    /// Pool name
    #[schemars(example = &"Foundry USA")]
    name: Cow<'static, str>,

    /// Pool website URL
    #[schemars(example = &"https://foundrydigital.com/")]
    link: Cow<'static, str>,

    /// Known payout addresses
    addresses: Vec<Cow<'static, str>>,

    /// Coinbase tag patterns (regexes)
    regexes: Vec<Cow<'static, str>>,

    /// URL-friendly pool identifier
    slug: PoolSlug,

    /// Unique pool identifier
    #[schemars(example = 44)]
    unique_id: u8,
}

impl From<&'static Pool> for PoolDetailInfo {
    fn from(pool: &'static Pool) -> Self {
        Self {
            id: pool.mempool_id(),
            name: Cow::Borrowed(pool.name),
            link: Cow::Borrowed(pool.link()),
            addresses: pool.addrs().iter().map(|&s| Cow::Borrowed(s)).collect(),
            regexes: pool.tags().iter().map(|&s| Cow::Borrowed(s)).collect(),
            slug: pool.slug(),
            unique_id: pool.mempool_unique_id(),
        }
    }
}

/// Block counts for different time periods
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PoolBlockCounts {
    /// Total blocks mined (all time)
    #[schemars(example = 75000)]
    pub all: u64,

    /// Blocks mined in last 24 hours
    #[serde(rename = "24h")]
    #[schemars(example = 42)]
    pub day: u64,

    /// Blocks mined in last week
    #[serde(rename = "1w")]
    #[schemars(example = 280)]
    pub week: u64,
}

/// Pool's share of total blocks for different time periods
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PoolBlockShares {
    /// Share of all blocks (0.0 - 1.0)
    #[schemars(example = 0.28)]
    pub all: f64,

    /// Share of blocks in last 24 hours (0.0 - 1.0)
    #[serde(rename = "24h")]
    #[schemars(example = 0.30)]
    pub day: f64,

    /// Share of blocks in last week (0.0 - 1.0)
    #[serde(rename = "1w")]
    #[schemars(example = 0.29)]
    pub week: f64,
}
